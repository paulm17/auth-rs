use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkedInUser {
    pub sub: Option<String>,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<Locale>,
    pub email: Option<String>,
    pub email_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Locale {
    pub country: String,
    pub language: String,
}

pub struct LinkedInProvider;

impl OAuthProfileProvider for LinkedInProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::LinkedIn
  }

  fn profile_url(&self) -> &'static str {
    "https://api.linkedin.com/v2/userinfo"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: LinkedInUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let email = user.email.ok_or_else(|| missing_field_error("Email"))?;
    let name = user.name.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}