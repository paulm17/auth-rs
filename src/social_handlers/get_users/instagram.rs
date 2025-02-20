use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstagramUser {
  pub id: String,
  pub username: Option<String>,
  pub full_name: Option<String>,
  pub profile_picture_url: Option<String>,
  pub bio: Option<String>,
  pub website: Option<String>,
  pub is_business_account: bool,
  pub is_private: Option<bool>,
  pub media_count: Option<u32>,
  pub follower_count: Option<u32>,
  pub following_count: Option<u32>,
  pub account_type: Option<String>,
  pub created_at: Option<DateTime<Utc>>,
  pub scopes: Vec<InstagramScope>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum InstagramScope {
  UserProfile,
  UserMedia,
  //
  Other(String),
}

pub struct InstagramProvider;

impl OAuthProfileProvider for InstagramProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Instagram
  }

  fn profile_url(&self) -> &'static str {
    "https://graph.instagram.com/me?fields=id,username,email"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: InstagramUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let username = user.username.ok_or_else(|| missing_field_error("Email"))?;
    let email = format!("{}@instagram.com", username.to_lowercase()); 
    let name = user.full_name.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}