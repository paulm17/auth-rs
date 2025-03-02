use std::sync::Arc;
use axum::{
  extract::{Query, State},
  http::{header, StatusCode},
  response::{IntoResponse, Redirect},
  Json,
};
use anyhow::Result;
use axum_extra::extract::cookie::{Cookie, SameSite};
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use chrono::{DateTime, Utc};
use oauth2::{
  basic::BasicClient, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl, TokenResponse, TokenUrl
};
use crate::{
  schema::{identities, social_auth, social_provider, tokens, user, Identity, SocialAuth, SocialProvider, Token, User}, token::generate_paseto_token, utils::parse_duration, AppState
};

use super::{fetcher::handle_oauth_provider, get_users::{amazon::AmazonProvider, facebook::FacebookProvider, github::GithubProvider, google::GoogleProvider, instagram::InstagramProvider, linkedin::LinkedInProvider, microsoft::MicrosoftProvider, reddit::RedditProvider, tiktok::TiktokProvider, twitch::TwitchProvider, twitter::TwitterProvider}, model::OAuthProvider};

// Generic OAuth callback parameters
#[derive(Debug, Deserialize, Serialize)]
pub struct OAuthCallbackParams {
  code: String,
  state: String,
}

pub async fn callback_handler(
  State(data): State<Arc<AppState>>,
  Query(params): Query<OAuthCallbackParams>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  
  // Verify CSRF token
  let social_oauth_exists = social_auth::table
    .filter(social_auth::csrf.eq(params.state.clone()))
    .first::<SocialAuth>(&mut conn)
    .optional();

  let social_oauth = if let Ok(Some(social_oauth)) = social_oauth_exists {
    social_oauth
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "CSRF token not found"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  let provider_exists = social_provider::table
    .filter(social_provider::id.eq(social_oauth.provider_id.clone()))
    .first::<SocialProvider>(&mut conn)
    .optional();

  let provider_name = if let Ok(Some(provider)) = provider_exists {
    provider.name
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Provider not found"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  if social_oauth.csrf != params.state {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Invalid state parameter",
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Get provider-specific client and fetch user info
  let provider = OAuthProvider::from_str(&provider_name).ok_or_else(|| {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Provider not found",
    });
    (StatusCode::BAD_REQUEST, Json(error_response))
  })?;

  let auth_url = AuthUrl::new(provider.auth_url().to_string())
    .expect("Invalid authorization endpoint URL");
  let token_url = TokenUrl::new(provider.token_url().to_string())
    .expect("Invalid token endpoint URL");
  let redirect_url = RedirectUrl::new(provider.redirect_url(&data))
    .expect("Invalid redirect URL");

  let client = BasicClient::new(ClientId::new(provider.client_id(&data)))
    .set_client_secret(ClientSecret::new(provider.client_secret(&data)))
    .set_auth_uri(auth_url)
    .set_token_uri(token_url)
    .set_redirect_uri(redirect_url);

  let client = match provider {
    OAuthProvider::LinkedIn => client.set_auth_type(AuthType::RequestBody),
    OAuthProvider::Twitch => client.set_auth_type(AuthType::RequestBody),
    _ => client,
  };

  let token_response = if provider == OAuthProvider::Twitter {    
    client
      .exchange_code(AuthorizationCode::new(params.code))
      .set_pkce_verifier(PkceCodeVerifier::new(social_oauth.pkce_verifier.to_string()))
    } else {
      client.exchange_code(AuthorizationCode::new(params.code))
    };

    let http_client = reqwest::Client::builder()
    .user_agent("Testing Oauth App/1.0")
    .build()
    .expect("Failed to create HTTP client");

  let token_result = token_response.request_async(&http_client)
    .await.map_err(|e| {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": format!("Failed to parse token: {}", e),
    });
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
  })?;

  // println!("{:?}", token_result);

  // let token_result = token_response.request_async(&http_client)
  //   .await.map_err(|e| match e {
  //     RequestTokenError::ServerResponse(response) => {
  //       format!("Server error: {:?}", response.error_description())
  //     }
  //     RequestTokenError::Request(request_err) => {
  //       format!("Request error: {}", request_err)   
  //     }
  //     RequestTokenError::Parse(parse_err, raw_response) => {
  //       // Convert raw_response bytes to string if possible

  //       let response_str = String::from_utf8_lossy(&raw_response);
  //       format!("Parse error: {}. Response: {}", parse_err, response_str)
  //     }
  //     other => format!("Other error: {:?}", other)
  // });

  let access_token = token_result.access_token().secret();

  // Fetch user info based on provider
  let (email, name, obj) = match provider {
    OAuthProvider::Amazon => handle_oauth_provider(&AmazonProvider, access_token).await?,
    OAuthProvider::Facebook => handle_oauth_provider(&FacebookProvider, access_token).await?,
    OAuthProvider::Github => handle_oauth_provider(&GithubProvider, access_token).await?,
    OAuthProvider::Google => handle_oauth_provider(&GoogleProvider, access_token).await?,
    OAuthProvider::Instagram => handle_oauth_provider(&InstagramProvider, access_token).await?,
    OAuthProvider::LinkedIn => handle_oauth_provider(&LinkedInProvider, access_token).await?,
    OAuthProvider::Microsoft => handle_oauth_provider(&MicrosoftProvider, access_token).await?,
    OAuthProvider::Reddit => handle_oauth_provider(&RedditProvider, access_token).await?,
    OAuthProvider::Tiktok => handle_oauth_provider(&TiktokProvider, access_token).await?,
    OAuthProvider::Twitch => handle_oauth_provider(&TwitchProvider, access_token).await?,
    OAuthProvider::Twitter => handle_oauth_provider(&TwitterProvider, access_token).await?,
  };

  let user_exists = user::table
    .filter(user::email.eq(email.clone()))
    .first::<User>(&mut conn)
    .optional();

    let user_id = if let Ok(Some(user)) = user_exists {
      user.id
    } else {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "Invalid email or password"
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    };

  if user_id.is_empty() {
    // Create new user
    let timestamp = Utc::now().naive_utc();
    let statement = diesel::insert_into(user::table)
      .values(&User {
          id: user_id.clone(),
          name: name.into(),
          email: email.into(),
          password: None,
          verified: false,
          created_at: timestamp,
          updated_at: None,
          deleted_at: None
      })
      .get_result::<User>(&mut conn);

    if let Err(e) = statement {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("New user could not be saved to database: {}", e),
      });
      
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
  }

  // Insert identity    
  let timestamp = Utc::now().naive_utc();
  let last_signin_at = Utc::now().naive_utc();
  let statement = diesel::insert_into(identities::table)
    .values(&Identity {
      id: Ulid::new().to_string().into(),
      user_id: user_id.clone().into(),
      provider_id: social_oauth.provider_id.into(),
      identity_data: obj.into(),
      last_signin_at: last_signin_at.into(),
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);

  if let Err(e) = statement {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": format!("New identity could not be saved to database: {}", e),
    });
    
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Generate tokens
  let access_token_details = generate_paseto_token(
    user_id.clone(),
    data.env.access_token_max_age,
    &data.paseto.access_key,
  ).unwrap();

  let refresh_token_details = generate_paseto_token(
    user_id.clone(),
    data.env.refresh_token_max_age,
    &data.paseto.refresh_key,
  ).unwrap();

  // Save tokens
  // Access token
  let expires = DateTime::<Utc>::from_timestamp(access_token_details.expires_in.unwrap(), 0)
    .map(|dt| dt.naive_utc())
    .unwrap_or_else(|| {
      // Handle invalid timestamps
      DateTime::<Utc>::from_timestamp(0, 0)
        .unwrap()
        .naive_utc()
    });
  let timestamp = Utc::now().naive_utc();
  let statement = diesel::insert_into(tokens::table)
    .values(&Token {
      id: Ulid::new().to_string(),
      user_id: user_id.clone().into(),
      expires,
      blacklisted: false,
      token: access_token_details.token.clone().unwrap(),
      token_uuid: access_token_details.token_uuid.to_string(),
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);
  
  if let Err(e) = statement {
    let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", e);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Refresh Token
  let expires = DateTime::<Utc>::from_timestamp(access_token_details.expires_in.unwrap(), 0)
    .map(|dt| dt.naive_utc())
    .unwrap_or_else(|| {
      // Handle invalid timestamps
      DateTime::<Utc>::from_timestamp(0, 0)
        .unwrap()
        .naive_utc()
    });
  let timestamp = Utc::now().naive_utc();
  let statement = diesel::insert_into(tokens::table)
    .values(&Token {
      id: Ulid::new().to_string(),
      user_id: user_id.into(),
      expires,
      blacklisted: false,
      token: refresh_token_details.token.clone().unwrap(),
      token_uuid: refresh_token_details.token_uuid.to_string(),
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);
  
  if let Err(e) = statement {
    let error_message = format!("Failed to save refresh token: validation error\nDetails: {:?}", e);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Set cookies
  let access_cookie = Cookie::build(
    ("access_token", access_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.access_token_expires_in).unwrap_or(900)))
    .same_site(SameSite::None)
    .http_only(false);

  let refresh_cookie = Cookie::build(
    ("refresh_token", refresh_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.refresh_token_expires_in).unwrap_or(900)))
    .same_site(SameSite::None)
    .http_only(true);

  let redirect = Redirect::temporary(&social_oauth.redirect_to);
  let mut response = redirect.into_response();

  response.headers_mut().append(
    header::SET_COOKIE,
    access_cookie.to_string().parse().unwrap(),
  );
  response.headers_mut().append(
    header::SET_COOKIE,
    refresh_cookie.to_string().parse().unwrap(),
  );

  Ok(response)
}