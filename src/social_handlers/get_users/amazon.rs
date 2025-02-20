use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AmazonUser {
  pub user_id: String,
  pub name: Option<String>,
  pub email: Option<String>,
  pub postal_code: Option<String>,
  pub profile: Option<String>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
  pub avatar_url: Option<String>,
  pub locale: Option<String>,
  pub country: Option<String>,
  #[serde(rename = "type")]
  pub user_type: Option<String>,
  pub verified: Option<bool>,
  pub phone_number: Option<String>,
  pub address: Option<String>,
  pub birthdate: Option<String>,
  pub timezone: Option<String>,
  pub account_status: Option<String>,
  pub last_login: Option<String>,
  pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferences {
  pub language: Option<String>,
  pub notification_enabled: Option<bool>,
  pub marketing_emails: Option<bool>,
}

// Your existing AmazonScope enum remains unchanged
#[derive(Debug, Serialize, Deserialize)]
pub enum AmazonScope {
  #[serde(rename = "profile")]
  Profile,
  #[serde(rename = "profile:user_id")]
  ProfileUserId,
  #[serde(rename = "postal_code")]
  PostalCode,
  Other(String),
}

pub struct AmazonProvider;

impl OAuthProfileProvider for AmazonProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Amazon
  }

  fn profile_url(&self) -> &'static str {
    "https://api.amazon.com/user/profile"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: AmazonUser = serde_json::from_slice(bytes).map_err(|_| (
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