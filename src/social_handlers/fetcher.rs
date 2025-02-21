use axum::{
  http::{header, StatusCode},
  Json,
};
use anyhow::Result;
use serde_json::json;

use super::model::OAuthProvider;

pub trait OAuthProfileProvider {
  fn provider(&self) -> &'static OAuthProvider;
  
  // Provider-specific URL
  fn profile_url(&self) -> &'static str;
  
  // Provider-specific headers
  fn additional_headers(&self) -> Vec<(String, String)> {
    Vec::new() // Default empty implementation
  }

  // Add this new method for providers that need additional requests
  async fn fetch_additional_data(
    &self, 
    _client: &reqwest::Client, 
    _access_token: &str
  ) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    Ok("".into()) // Default implementation returns null
  }
  
  // Provider-specific parsing
  fn extract_user_info(&self, response_json: serde_json::Value, bytes: &[u8]) 
    -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)>;
}

pub async fn handle_oauth_provider<P: OAuthProfileProvider>(
  provider: &P,
  access_token: &str,
) -> Result<(String, String, serde_json::Value), (StatusCode, Json<serde_json::Value>)> {
  let mut request = reqwest::Client::new()
    .get(provider.profile_url())
    .header(header::AUTHORIZATION, format!("Bearer {}", access_token));
  
  // Add provider-specific headers
  for (key, value) in provider.additional_headers() {
    request = request.header(key, value);
  }
  
  let response = request.send().await.map_err(handle_error)?;
  let bytes = response.bytes().await.map_err(handle_error)?;
  
  // Parse the response text from bytes for raw JSON
  let response_text = String::from_utf8(bytes.clone().to_vec()).map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({"status": "error", "message": format!("Failed to parse response text: {}", e)}))
    )
  })?;

  let response_json: serde_json::Value = serde_json::from_str(&response_text).map_err(|e| {
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(json!({"status": "error", "message": format!("Failed to parse JSON: {}", e)}))
    )
  })?;  

  // Extract user info using provider-specific implementation
  let (mut email, name, obj) = provider.extract_user_info(response_json, &bytes)?;

  // Fetch additional data if needed
  if provider.provider() == &OAuthProvider::Github {
    email = provider.fetch_additional_data(&reqwest::Client::new(), access_token).await.unwrap();
  }
  
  Ok((email, name, obj))
}

pub fn handle_error(e: reqwest::Error) -> (StatusCode, Json<serde_json::Value>) {
  let error_response = json!({
    "status": "fail",
    "message": format!("Failed to fetch user: {}", e),
  });
  (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
}

pub fn missing_field_error(field: &str) -> (StatusCode, Json<serde_json::Value>) {
  let error_response = json!({
    "status": "fail",
    "message": format!("{} not provided", field),
  });
  (StatusCode::BAD_REQUEST, Json(error_response))
}