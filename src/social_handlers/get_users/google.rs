use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleUser {
  pub email: Option<String>,
  pub email_verified: Option<bool>,
  pub family_name: Option<String>,
  pub given_name: Option<String>,
  #[serde(default)]
  pub locale: Option<String>,
  pub picture: Option<String>,
}

pub struct GoogleProvider;

impl OAuthProfileProvider for GoogleProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Google
  }

  fn profile_url(&self) -> &'static str {
    "https://www.googleapis.com/oauth2/v3/userinfo"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: GoogleUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let email = user.given_name.ok_or_else(|| missing_field_error("Email"))?;
    let name = user.email.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}