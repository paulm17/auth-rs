use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchUserResponse {
  pub data: Vec<TwitchUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TwitchUser {
  pub id: String,
  pub login: Option<String>,
  #[serde(rename = "display_name")]
  pub display_name: Option<String>,
  #[serde(rename = "type")]
  pub user_type: Option<String>,
  #[serde(rename = "broadcaster_type")]
  pub broadcaster_type: Option<String>,
  pub description: Option<String>,
  #[serde(rename = "profile_image_url")]
  pub profile_image_url: Option<String>,
  #[serde(rename = "offline_image_url")]
  pub offline_image_url: Option<String>,
  #[serde(rename = "view_count")]
  pub view_count: Option<i32>,
  pub email: Option<String>,
  #[serde(rename = "created_at")]
  pub created_at: Option<String>,
}

pub struct TwitchProvider;

impl OAuthProfileProvider for TwitchProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Twitch
  }

  fn profile_url(&self) -> &'static str {
    "https://api.twitch.tv/helix/users"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: TwitchUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let email = user.email.ok_or_else(|| missing_field_error("Email"))?;
    let name = user.display_name.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}