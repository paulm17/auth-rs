use std::sync::Arc;
use argon2::{
  password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
  Argon2,
};
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use chrono::Utc;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde_json::json;
use crate::{
  model::ResetPasswordSchema, schema::{email_confirmation, user, EmailConfirmation, UserPasswordUpdate}, utils::update_confirm_code, AppState
};

pub async fn reset_password_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<ResetPasswordSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  let code = body.code.to_owned();

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

    let timestamp = Utc::now().naive_utc();
    let statement = diesel::update(user::table)
    .filter(user::id.eq(confirmation.user_id))
    .set(&UserPasswordUpdate {
        password: hashed_password.into(),
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

  let _ = update_confirm_code(axum::extract::State(data), confirmation.id.to_string(), "completed".to_string()).await;

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