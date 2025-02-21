use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use reqwest::header;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
  pub login: String,
  pub id: i64,
  pub node_id: String,
  pub avatar_url: String,
  pub gravatar_id: String,
  pub url: String,
  pub html_url: String,
  pub followers_url: String,
  pub following_url: String,
  pub gists_url: String,
  pub starred_url: String,
  pub subscriptions_url: String,
  pub organizations_url: String,
  pub repos_url: String,
  pub events_url: String,
  pub received_events_url: String,
  #[serde(rename = "type")]
  pub user_type: String,
  pub user_view_type: String,
  pub site_admin: bool,
  pub name: Option<String>,
  pub company: Option<String>,
  pub blog: String,
  pub location: Option<String>,
  pub email: Option<String>,
  pub hireable: Option<bool>,
  pub bio: Option<String>,
  pub twitter_username: Option<String>,
  pub notification_email: Option<String>,
  pub public_repos: i32,
  pub public_gists: i32,
  pub followers: i32,
  pub following: i32,
  pub created_at: String,
  pub updated_at: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

pub struct GithubProvider;

impl OAuthProfileProvider for GithubProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Github
  }

  fn profile_url(&self) -> &'static str {
    "https://api.github.com/user"
  }
  
  fn additional_headers(&self) -> Vec<(String, String)> {
    vec![
      ("User-Agent".to_string(), "your-app-name/1.0".to_string())
    ]
  }

  async fn fetch_additional_data(
    &self,
    client: &reqwest::Client,
    access_token: &str
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let github_emails = client
      .get("https://api.github.com/user/emails")
      .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
      .header(header::USER_AGENT, "Your-App-Name")
      .send()
      .await
      .map_err(|e| (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(json!({
              "status": "error",
              "message": format!("Failed to fetch GitHub emails: {}", e)
          }))
      ))?
      .json::<Vec<GitHubEmail>>()
      .await
      .map_err(|e| (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(json!({
              "status": "error",
              "message": format!("Failed to parse GitHub emails: {}", e)
          }))
      ))?;

    let email = github_emails
      .iter()
      .find(|e| e.primary && e.verified)
      .ok_or_else(|| missing_field_error("Email"))?
      .email
      .clone();

    Ok(email)
  }

  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: GitHubUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;      
    
    let name = user.name
      .ok_or_else(|| missing_field_error("Name"))?;
    
    Ok(("".into(), name, response_json))
  }
}