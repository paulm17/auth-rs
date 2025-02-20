use std::sync::Arc;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
  };
use axum::{
  extract::State, http::StatusCode, response::IntoResponse, Json
};
use anyhow::Result;
use convex::FunctionResult::Value;
use convex::Value::Array;
use ulid::Ulid;
use chrono::Utc;
use crate::{
  model::RegisterUserSchema, AppState
};

pub async fn register_user_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let email = body.email.to_owned().to_ascii_lowercase();
  let mut client = data.convex.clone();
  let user_exists = client.query("users:getUserbyEmail", maplit::btreemap!{
    "email".into() => email.into(),
  }).await;

  let exists = match user_exists {
    Ok(Value(Array(arr))) => !arr.is_empty(),
    _ => false,
  };

  if exists {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "User with that email already exists",
    });
    return Err((StatusCode::CONFLICT, Json(error_response)));
  }

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

  let timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let result = client.mutation("users:insertUserbyEmail", maplit::btreemap!{
    "id".into() => Ulid::new().to_string().into(),
    "name".into() => body.name.into(),
    "email".into() => body.email.into(),
    "password".into() => hashed_password.into(),
    "verified".into() => false.into(),
    "role".into() => "user".into(),
    "createdAt".into() => timestamp_float.into(),
  }).await;

  match result {
    Ok(Value(convex::Value::String(id))) => Ok(Json(serde_json::json!({
      "status": "success",
      "message": "User registered successfully",
      "data": {
          "user_id": id
      }
    }))),
    _ => {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "Failed to create user",
      });
      Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }
  }
}
