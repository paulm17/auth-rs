use std::sync::Arc;
use argon2::{
  password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
  Argon2,
};
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use serde_json::json;
use convex::FunctionResult::Value;
use crate::{
  model::ResetPasswordSchema, utils::update_confirm_code, AppState
};

pub async fn reset_password_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<ResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut client = data.convex.clone();
  let result = client.query("emailConfirmation:getConfirmationByCode", maplit::btreemap!{
    "code".into() => body.code.to_owned().into(),
    "flow".into() => "seen".into(),
  }).await;

  let user_id = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("userId")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  if user_id == "" {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Code is invalid or has expired"
    });
    return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
  }

  let code_id = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("_id")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  let token_result = client.query("users:getUserbyId", maplit::btreemap!{
    "id".into() => user_id.into()
  }).await;

  let _id = match &token_result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("_id")
      .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  let salt = SaltString::generate(&mut OsRng);
  let hashed_password = Argon2::default()
    .hash_password(body.password.as_bytes(), &salt)
    .map_err(|e| {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Error while hashing password: {}", e),
      });
      (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
    .map(|hash| hash.to_string())?;

  let result = client.mutation("users:resetPassword", maplit::btreemap!{
    "id".into() => _id.into(),
    "password".into() => hashed_password.into(),
  }).await;

  if format!("{:?}", result).contains("Server Error") {
    println!("{:?}", result);

    let error_message = "Failed to reset password";
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  update_confirm_code(axum::extract::State(data), code_id.to_string(), "completed".to_string()).await;

  let mut response = Response::new(
    json!({"status": "ok"})
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