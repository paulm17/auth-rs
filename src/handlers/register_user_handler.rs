use std::sync::Arc;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
  };
use axum::{
  extract::State, http::StatusCode, response::IntoResponse, Json
};
use anyhow::Result;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use ulid::Ulid;
use chrono::Utc;
use crate::{
  model::RegisterUserSchema, schema::{user, User}, AppState
};

pub async fn register_user_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<RegisterUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let email = body.email.to_owned().to_ascii_lowercase();
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

  let user_exists = user::table
    .filter(user::email.eq(email.clone()))
    .first::<User>(&mut conn)
    .optional();

  match user_exists {
    Ok(Some(_)) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "User already exists",
      });
      return Err((StatusCode::CONFLICT, Json(error_response)));
    },
    Ok(None) => {
    },
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Failure: {}", e),
      });
      return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    },
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
  let result = diesel::insert_into(user::table)
    .values(&User {
        id: Ulid::new().to_string(),
        name: body.name,
        email: body.email,
        password: Some(hashed_password),
        verified: false,
        created_at: timestamp,
        updated_at: None,
        deleted_at: None
    })
    .get_result::<User>(&mut conn);

  match result {
    Ok(inserted_user) => {
      Ok(Json(serde_json::json!({
        "status": "success",
        "message": "User registered successfully",
        "data": {
          "user_id": inserted_user.id
        }
    })))
    },
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("Failed to create user: {}", e),
      });
      Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    }
  }
}

