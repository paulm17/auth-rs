use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TiktokUserInfoResponse {
  pub data: TiktokUserData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TiktokUserData {
  pub user: TiktokUser,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TiktokUser {
  #[serde(rename = "open_id")]
  pub open_id: String,
  #[serde(rename = "union_id")]
  pub union_id: Option<String>,
  pub nickname: Option<String>,
  pub avatar: Option<String>,
  pub biography: Option<String>,
  #[serde(rename = "is_verified")]
  pub is_verified: Option<bool>,
  #[serde(rename = "follower_count")]
  pub follower_count: Option<i32>,
  #[serde(rename = "following_count")]
  pub following_count: Option<i32>,
}

pub struct TiktokProvider;

impl OAuthProfileProvider for TiktokProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Tiktok
  }

  fn profile_url(&self) -> &'static str {
    "https://open-api.tiktok.com/user/info/"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: TiktokUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let nickname = user.nickname.clone().ok_or_else(|| missing_field_error("Email"))?;
    let email = format!("{}@tiktok.com", nickname.to_lowercase()); 
    let name = user.nickname.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}