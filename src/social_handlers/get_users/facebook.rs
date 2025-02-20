use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FacebookUser {
  pub id: String,  // Facebook uses string IDs
  pub name: Option<String>,
  pub email: Option<String>,
  pub picture: Option<PictureData>,
  pub first_name: Option<String>,
  pub last_name: Option<String>,
  pub middle_name: Option<String>,
  pub birthday: Option<String>,
  pub gender: Option<String>,
  pub location: Option<Location>,
  pub hometown: Option<Location>,
  pub link: Option<String>,
  pub age_range: Option<AgeRange>,
  pub friends: Option<ConnectionData>,
  pub photos: Option<ConnectionData>,
  pub posts: Option<ConnectionData>,
  pub created_time: Option<String>,
  pub updated_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PictureData {
  pub url: String,
  pub width: Option<i32>,
  pub height: Option<i32>,
  pub is_silhouette: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Location {
  pub id: String,
  pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgeRange {
  pub min: i32,
  pub max: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionData {
  pub total_count: Option<i32>,
  pub paging: Option<PagingData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PagingData {
  pub cursors: Cursors,
  pub next: Option<String>,
  pub previous: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cursors {
  pub before: String,
  pub after: String,
}

pub struct FacebookProvider;

impl OAuthProfileProvider for FacebookProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Facebook
  }
  
  fn profile_url(&self) -> &'static str {
    "https://graph.facebook.com/me?fields=name,email"
  }
  
  fn additional_headers(&self) -> Vec<(String, String)> {
    vec![
      ("User-Agent".to_string(), "your-app-name/1.0".to_string())
    ]
  }
  
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: FacebookUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let name = user.name
      .ok_or_else(|| missing_field_error("Name"))?;
    let email = format!("{}@facebook.com", name.to_lowercase());
    
    Ok((email, name, response_json))
  }
}