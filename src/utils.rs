use std::sync::Arc;
use axum::{extract::State, http::StatusCode, Json};
use crate::{token::{generate_jwt_token, TokenDetails}, AppState};

pub fn parse_duration(duration_str: &str) -> Result<i64, Box<dyn std::error::Error>> {
    let len = duration_str.len();
    let (value, unit) = duration_str.split_at(len - 1);
    let value: i64 = value.parse()?;
    
    match unit {
        "s" => Ok(value),
        "m" => Ok(value * 60),
        "h" => Ok(value * 3600),
        "d" => Ok(value * 86400),
        _ => Err("Invalid duration unit. Use s, m, h, or d".into())
    }
}

pub fn generate_token(
    user_id: String,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<serde_json::Value>)> {
  generate_jwt_token(user_id, max_age, private_key).map_err(|e| {
    let error_response = serde_json::json!({
        "status": "error",
        "message": format!("error generating token: {}", e),
    });
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
  })
}

pub async fn update_confirm_code(
  State(data): State<Arc<AppState>>,
  id: String,
  flow: String
) {
  let mut client = data.convex.clone();
  
  let _ = client.mutation("emailConfirmation:updateCode", maplit::btreemap!{
    "id".into() => id.into(),
    "flow".into() => flow.into()
  }).await;
}

