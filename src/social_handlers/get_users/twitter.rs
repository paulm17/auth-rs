use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterMeResponse {
  pub data: TwitterUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitterUser {
  pub id: String,
  pub name: Option<String>,
  pub username: Option<String>,
  pub protected: Option<bool>,
  pub verified: Option<bool>,
  #[serde(rename = "created_at")]
  pub created_at: Option<String>,
  pub description: Option<String>,
  #[serde(rename = "profile_image_url")]
  pub profile_image_url: Option<String>,
  pub location: Option<String>,
}

pub struct TwitterProvider;

impl OAuthProfileProvider for TwitterProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Twitter
  }

  fn profile_url(&self) -> &'static str {
    "https://api.twitter.com/2/users/me"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: TwitterUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let username = user.username.ok_or_else(|| missing_field_error("Email"))?;
    let email = format!("{}@twitter.com", username.to_lowercase()); 
    let name = user.name.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}