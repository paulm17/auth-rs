use std::sync::Arc;
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde_json::json;
use crate::{
  model::CheckCodeSchema, schema::{email_confirmation, EmailConfirmation}, AppState
};

pub async fn check_code_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<CheckCodeSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

  let confirmation_exists = email_confirmation::table
    .filter(email_confirmation::code.eq(body.code.to_owned()))
    .filter(email_confirmation::flow.eq("created"))
    .first::<EmailConfirmation>(&mut conn)
    .optional();

  let flow = if let Ok(Some(confirmation)) = confirmation_exists {
    confirmation.flow
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Code is invalid or has expired"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  if flow != "seen" {
    let error_response = serde_json::json!({
      "is_valid": false
    });
    return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
  }

  let mut response = Response::new(
    json!({"is_valid": true})
      .to_string(),
  );

  let mut headers = HeaderMap::new();
  headers.append(
    header::CONTENT_TYPE,
    "application/json".parse().unwrap(),
  );

  response.headers_mut().extend(headers);

  Ok(response)
}