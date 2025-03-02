use std::sync::Arc;
use axum::{extract::State, response::IntoResponse, Json};
use chrono::Utc;
use diesel::{ExpressionMethods, RunQueryDsl};
use reqwest::StatusCode;
use crate::{schema::{email_confirmation, EmailConfirmationFlowUpdate}, AppState};

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

pub async fn update_confirm_code(
  State(data): State<Arc<AppState>>,
  id: String,
  flow: String
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

  let timestamp = Utc::now().naive_utc();
  let statement = diesel::update(email_confirmation::table)
    .filter(email_confirmation::id.eq(id))
    .set(&EmailConfirmationFlowUpdate {
        flow: flow.into(),
        updated_at: timestamp.into(),
    })
    .execute(&mut conn);

  if let Err(e) = statement {
    let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Failed to reset password: validation error\nDetails: {:?}", e)
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  Ok(())
}

