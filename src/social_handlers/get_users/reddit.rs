use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct RedditUser {
  pub id: Option<String>,
  pub name: Option<String>,
  #[serde(rename = "icon_img")]
  pub icon_img: Option<String>,
  pub created: Option<f64>,
  pub verified: Option<bool>,
  #[serde(rename = "over_18")]
  pub over_18: Option<bool>,
  #[serde(rename = "has_mail")]
  pub has_mail: Option<bool>,
  #[serde(rename = "inbox_count")]
  pub inbox_count: Option<i32>,
}

pub struct RedditProvider;

impl OAuthProfileProvider for RedditProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Reddit
  }

  fn profile_url(&self) -> &'static str {
    "https://oauth.reddit.com/api/v1/me"
  }
  
  fn additional_headers(&self) -> Vec<(String, String)> {
    vec![
      ("User-Agent".to_string(), "your-app-name/1.0".to_string())
    ]
  }
  
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: RedditUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let name = user.name
      .ok_or_else(|| missing_field_error("Name"))?;
    let email = format!("{}@reddit.com", name.to_lowercase());
    
    Ok((email, name, response_json))
  }
}