use crate::social_handlers::{fetcher::{missing_field_error, OAuthProfileProvider}, model::OAuthProvider};
use axum::{
  http::StatusCode,
  Json,
};
use anyhow::Result;
use serde_json::json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MicrosoftUser {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "businessPhones")]
    pub business_phones: Option<Vec<String>>,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[serde(rename = "officeLocation")]
    pub office_location: Option<String>,
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: Option<String>,
}

pub struct MicrosoftProvider;

impl OAuthProfileProvider for MicrosoftProvider {
  fn provider(&self) -> &'static OAuthProvider {
    &OAuthProvider::Microsoft
  }

  fn profile_url(&self) -> &'static str {
    "https://graph.microsoft.com/v1.0/me"
  }
    
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
    let user: MicrosoftUser = serde_json::from_slice(bytes).map_err(|_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({
        "status": "error",
        "message": "Failed to parse response"
      }))
    ))?;
    
    let email = user.mail.ok_or_else(|| missing_field_error("Email"))?;
    let name = user.display_name.ok_or_else(|| missing_field_error("Name"))?;
    
    Ok((email, name, response_json))
  }
}