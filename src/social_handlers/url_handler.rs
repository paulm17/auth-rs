use std::sync::Arc;
use axum::{
    extract::State,
    http::{header, HeaderMap, Response, StatusCode},
    response::IntoResponse,
    Json,
};
use anyhow::Result;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde_json::json;
use ulid::Ulid;
use chrono::{Duration, Utc};
use crate::{model::OAuthSchema, schema::{social_auth, social_provider, SocialAuth, SocialProvider}, AppState};
use oauth2::{
  basic::BasicClient,
  AuthUrl,
  ClientId,
  ClientSecret,
  CsrfToken,
  PkceCodeChallenge,
  RedirectUrl,
  Scope,
  TokenUrl,
};

use super::model::OAuthProvider;

impl OAuthProvider {
  pub fn from_str(provider: &str) -> Option<Self> {
    match provider.to_lowercase().as_str() {
      "amazon" => Some(Self::Amazon),
      "facebook" => Some(Self::Facebook),
      "github" => Some(Self::Github),
      "google" => Some(Self::Google),
      "instagram" => Some(Self::Instagram),
      "linkedin" => Some(Self::LinkedIn),
      "microsoft" => Some(Self::Microsoft),
      "reddit" => Some(Self::Reddit),
      "tiktok" => Some(Self::Tiktok),
      "twitch" => Some(Self::Twitch),
      "twitter" => Some(Self::Twitter),
      _ => None,
    }
  }

  pub fn auth_url(&self) -> &'static str {
    match self {
      Self::Amazon => "https://www.amazon.com/ap/oa",
      Self::Facebook => "https://www.facebook.com/v12.0/dialog/oauth",
      Self::Github => "https://github.com/login/oauth/authorize",
      Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
      Self::Instagram => "https://api.instagram.com/oauth/authorize",
      Self::LinkedIn => "https://www.linkedin.com/oauth/v2/authorization",
      Self::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/authorize",
      Self::Reddit => "https://www.reddit.com/api/v1/authorize",
      Self::Tiktok => "https://open.tiktokapis.com/v2/oauth/authorize",
      Self::Twitch => "https://id.twitch.tv/oauth2/authorize",
      Self::Twitter => "https://x.com/i/oauth2/authorize",
    }
  }

  pub fn token_url(&self) -> &'static str {
    match self {
      Self::Amazon => "https://api.amazon.com/auth/o2/token",
      Self::Facebook => "https://graph.facebook.com/v12.0/oauth/access_token",
      Self::Github => "https://github.com/login/oauth/access_token",
      Self::Google => "https://www.googleapis.com/oauth2/v3/token",
      Self::Instagram => "https://api.instagram.com/oauth/access_token",
      Self::LinkedIn => "https://www.linkedin.com/oauth/v2/accessToken",
      Self::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
      Self::Reddit => "https://www.reddit.com/api/v1/access_token",
      Self::Tiktok => "https://open.tiktokapis.com/v2/oauth/token",
      Self::Twitch => "https://id.twitch.tv/oauth2/token",
      Self::Twitter => "https://api.x.com/2/oauth2/token",
    }
  }

  pub fn client_id(&self, app_state: &AppState) -> String {
    match self {
      Self::Amazon => app_state.env.amazon_client_id.clone(),
      Self::Facebook => app_state.env.facebook_client_id.clone(),
      Self::Github => app_state.env.github_client_id.clone(),
      Self::Google => app_state.env.google_client_id.clone(),
      Self::Instagram => app_state.env.instagram_client_id.clone(),
      Self::LinkedIn => app_state.env.linkedin_client_id.clone(),
      Self::Microsoft => app_state.env.microsoft_client_id.clone(),
      Self::Reddit => app_state.env.reddit_client_id.clone(),
      Self::Tiktok => app_state.env.tiktok_client_id.clone(),
      Self::Twitch => app_state.env.twitch_client_id.clone(),
      Self::Twitter => app_state.env.twitter_client_id.clone(),
    }
  }

  pub fn client_secret(&self, app_state: &AppState) -> String {
    match self {
      Self::Amazon => app_state.env.amazon_client_secret.clone(),
      Self::Facebook => app_state.env.facebook_client_secret.clone(),
      Self::Github => app_state.env.github_client_secret.clone(),
      Self::Google => app_state.env.google_client_secret.clone(),
      Self::Instagram => app_state.env.instagram_client_secret.clone(),
      Self::LinkedIn => app_state.env.linkedin_client_secret.clone(),
      Self::Microsoft => app_state.env.microsoft_client_secret.clone(),
      Self::Reddit => app_state.env.reddit_client_secret.clone(),
      Self::Tiktok => app_state.env.tiktok_client_secret.clone(),
      Self::Twitch => app_state.env.twitch_client_secret.clone(),
      Self::Twitter => app_state.env.twitter_client_secret.clone(),
    }
  }

  pub fn redirect_url(&self, app_state: &AppState) -> String {
    let base_url = &app_state.env.server_url;
    match self {
      Self::Amazon => format!("{}{}", base_url, app_state.env.amazon_redirect_url),
      Self::Facebook => format!("{}{}", gen_https_base_url(base_url.to_string()), app_state.env.facebook_redirect_url),
      Self::Github => format!("{}{}", base_url, app_state.env.github_redirect_url),
      Self::Google => format!("{}{}", base_url, app_state.env.google_redirect_url),
      Self::Instagram => format!("{}{}", base_url, app_state.env.instagram_redirect_url),
      Self::LinkedIn => format!("{}{}", base_url, app_state.env.linkedin_redirect_url),
      Self::Microsoft => format!("{}{}", base_url, app_state.env.microsoft_redirect_url),
      Self::Reddit => format!("{}{}", base_url, app_state.env.reddit_redirect_url),
      Self::Tiktok => format!("{}{}", base_url, app_state.env.tiktok_redirect_url),
      Self::Twitch => format!("{}{}", gen_https_base_url(base_url.to_string()), app_state.env.twitch_redirect_url),
      Self::Twitter => format!("{}{}", base_url, app_state.env.twitter_redirect_url),
    }
  }
}

fn gen_https_base_url(base_url: String) -> String {
  base_url.replace("http://", "https://").replace(":8000", ":8443")
}

pub async fn url_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<OAuthSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

  let provider = match OAuthProvider::from_str(&body.provider.clone()) {
    Some(p) => p,
    None => {
      let error_response = json!({
        "status": "fail",
        "message": "Invalid provider"
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
  };
  
  let scopes = body.scopes.to_owned();
  let callback_url = body.callback_url.to_owned();

  let provider_exists = social_provider::table
    .filter(social_provider::name.eq(body.provider))
    .first::<SocialProvider>(&mut conn)
    .optional();

  let provider_id = if let Ok(Some(provider)) = provider_exists {
    provider.id
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Provider not found"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  let client_id = ClientId::new(provider.client_id(&data));
  let client_secret = ClientSecret::new(provider.client_secret(&data));
  let auth_url = AuthUrl::new(provider.auth_url().to_string())
    .expect("Invalid authorization endpoint URL");
  let token_url = TokenUrl::new(provider.token_url().to_string())
    .expect("Invalid token endpoint URL");

  let client = BasicClient::new(client_id)
    .set_client_secret(client_secret)
    .set_auth_uri(auth_url)
    .set_token_uri(token_url)
    .set_redirect_uri(
        RedirectUrl::new(provider.redirect_url(&data))
      .expect("Invalid redirect URL"),
    );

  let auth_builder = client.authorize_url(CsrfToken::new_random);    
  let mut pkce_verifier = String::new();

  let auth_builder = if provider == OAuthProvider::Twitter {
    let (pkce_challenge, verifier) = PkceCodeChallenge::new_random_sha256();
      
    pkce_verifier = verifier.secret().to_string();
    auth_builder.set_pkce_challenge(pkce_challenge)
  } else {
    auth_builder
  };
  
  let auth_builder = scopes.split(',').fold(auth_builder, |builder, scope| {
    builder.add_scope(Scope::new(scope.to_string()))
  });

  let (auth_url, csrf_token) = auth_builder.url();

  // Store CSRF token in database for verification
  let expires = (Utc::now() + Duration::days(10)).naive_utc();
  let timestamp = Utc::now().naive_utc();
  let statement = diesel::insert_into(social_auth::table)
    .values(&SocialAuth {
      id: Ulid::new().to_string(),
      provider_id: provider_id.into(),
      csrf: csrf_token.secret().to_string().into(),
      pkce_verifier: pkce_verifier.into(),
      redirect_to: callback_url.into(),
      expires,
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);
  
  if let Err(e) = statement {
    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("forgot password code not saved to database: validation error\nDetails: {:?}", e)
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let mut response = Response::new(json!({ "url": auth_url.as_str().to_string() }).to_string());

  let mut headers = HeaderMap::new();
  headers.append(header::CONTENT_TYPE, "application/json".parse().unwrap());
  response.headers_mut().extend(headers);

  Ok(response)
}

