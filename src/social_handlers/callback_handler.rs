use std::sync::Arc;
use axum::{
  extract::{Query, State},
  http::{header, StatusCode},
  response::{IntoResponse, Redirect},
  Json,
};
use anyhow::Result;
use axum_extra::extract::cookie::{Cookie, SameSite};
use serde::{Deserialize, Serialize};
use ulid::Ulid;
use chrono::Utc;
use oauth2::{
  basic::BasicClient, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, PkceCodeVerifier, RedirectUrl, TokenResponse, TokenUrl
};
use convex::FunctionResult::Value;
use crate::{
  utils::{generate_token, parse_duration}, AppState
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
    // Verify CSRF token
  let mut client = data.convex.clone();
  let result = client
    .query(
      "socialOauth:getCSRF",
      maplit::btreemap! {
          "csrf".into() => params.state.clone().into(),
      },
    )
    .await;

  let (csrf, provider_id, redirect_to, pkce_verifier) = match &result {
    Ok(Value(convex::Value::Object(obj))) => {
      let csrf = obj.get("csrf")
        .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
        })
        .unwrap_or("");

      let provider_id = obj.get("providerId")
        .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
        })
        .unwrap_or("");

      let redirect_to = obj.get("redirectTo")
        .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
        })
        .unwrap_or("");

      let pkce_verifier = obj.get("pkceVerifier")
        .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
        })
        .unwrap_or("");

      (csrf, provider_id, redirect_to, pkce_verifier)
    }
    _ => ("", "", "", ""),
  };

  let provider_exists = client.query("socialProviders:getProviderById", maplit::btreemap!{
    "id".into() => provider_id.into(),
  }).await;

  let provider_name = match &provider_exists {
    Ok(Value(convex::Value::Object(obj))) => obj.get("name")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format_args!("Provider not found {:?}", e)
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
    _ => "",
  };

  if csrf != params.state {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Invalid state parameter",
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Get provider-specific client and fetch user info
  let provider = OAuthProvider::from_str(provider_name).ok_or_else(|| {
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
      .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
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

  // Check if user exists
  let mut client = data.convex.clone();
  let user_exists = client
    .query(
      "users:getUserbyEmail",
      maplit::btreemap! {
        "email".into() => email.clone().into(),
      },
    )
    .await;

  let mut user_id = match &user_exists {
    Ok(Value(convex::Value::Object(obj))) => obj
      .get("id")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.to_string()),
        _ => None,
      })
      .unwrap_or_else(String::new),
    _ => String::new(),
  };

  if user_id.is_empty() {
    // Create new user
    let timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
    user_id = Ulid::new().to_string();
    let result = client
      .mutation(
      "users:insertUserbyEmail",
      maplit::btreemap! {
        "id".into() => user_id.clone().into(),
        "name".into() => name.into(),
        "email".into() => email.into(),
        "verified".into() => true.into(),
        "role".into() => "user".into(),
        "createdAt".into() => timestamp_float.into(),
        },
      )
      .await;

    if format!("{:?}", result).contains("Server Error") {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "New user could not be saved to database",
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
  }

  // Insert identity    
  let last_signin_at_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let result = client.mutation("identities:insertIdentity", maplit::btreemap!{
    "id".into() => Ulid::new().to_string().into(),
    "userId".into() => user_id.clone().into(),
    "providerId".into() => provider_id.into(),
    "identityData".into() => obj.to_string().into(),
    "lastSignInAt".into() => last_signin_at_float.into(),
    "createdAt".into() => timestamp_float.into(),
  }).await;

  if format!("{:?}", result).contains("Server Error") {
    println!("{:?}", result);

    let error_message = "new identity could not saved to database";
    let error_response = serde_json::json!({
      "status": "fail",
      "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Generate tokens
  let access_token_details = generate_token(
    user_id.clone(),
    data.env.access_token_max_age,
    data.rsa.access_tokens.private_key.to_owned(),
  )?;

  let refresh_token_details = generate_token(
    user_id.clone(),
    data.env.refresh_token_max_age,
    data.rsa.refresh_tokens.private_key.to_owned(),
  )?;

  // Save tokens
  let access_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let _ = client.mutation("tokens:insertToken", maplit::btreemap!{
    "id".into() => access_token_details.token_uuid.to_string().into(),
    "userId".into() => user_id.clone().into(),
    "token".into() => access_token_details.token.clone().unwrap().into(),
    "expires".into() => (access_token_details.expires_in.clone().unwrap() as f64).into(),
    "createdAt".into() => access_token_timestamp_float.into(),
  }).await;

  let refresh_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let _ = client.mutation("tokens:insertToken", maplit::btreemap!{
    "id".into() => refresh_token_details.token_uuid.to_string().into(),
    "userId".into() => user_id.clone().into(),
    "token".into() => refresh_token_details.token.clone().unwrap().into(),
    "expires".into() => (refresh_token_details.expires_in.clone().unwrap() as f64).into(),
    "createdAt".into() => refresh_token_timestamp_float.into(),
  }).await;

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

  let redirect = Redirect::temporary(&redirect_to);
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