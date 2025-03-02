use std::sync::Arc;
use axum::{
  extract::{Query, State}, http::StatusCode, response::{IntoResponse, Redirect}, Json
};
use anyhow::Result;
use chrono::{DateTime, TimeZone, Utc};
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use crate::{
  model::VerifyCodeSchema, schema::{email_confirmation, EmailConfirmation}, utils::update_confirm_code, AppState
};

pub async fn verify_code_handler(
  State(data): State<Arc<AppState>>,
  Query(body): Query<VerifyCodeSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  let code = body.code.to_owned();

  // verify code exists and has not been used
  let confirmation_exists = email_confirmation::table
    .filter(email_confirmation::code.eq(code.clone().to_owned()))
    .filter(email_confirmation::flow.eq("created"))
    .first::<EmailConfirmation>(&mut conn)
    .optional();

  let confirmation = if let Ok(Some(confirmation)) = confirmation_exists {
    confirmation
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "verification code does not exist"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  let current_time = Utc::now();
  let expiry_time: DateTime<Utc> = Utc.from_utc_datetime(&confirmation.expires);

  if current_time > expiry_time {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Code is invalid or has expired"
    });
    return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
  }

  let _ = update_confirm_code(axum::extract::State(data), confirmation.id.to_string(), "seen".to_string()).await;

  Ok(Redirect::temporary(&format!("{}?code={}", confirmation.redirect_to.unwrap(), confirmation.code)))
}