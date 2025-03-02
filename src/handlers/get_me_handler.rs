use axum::{
  response::IntoResponse, Extension, Json
};
use anyhow::Result;
use reqwest::StatusCode;
use crate::{
  jwt_auth::JWTAuthMiddleware, response::FilteredUser
};


pub async fn get_me_handler(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
  ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let user = &jwtauth.user;
    let json_response = serde_json::json!({
      "user": FilteredUser {
        id: user.id.to_string(),
        email: user.email.to_owned(),
        name: user.name.to_owned(),
        verified: user.verified,
        role: "".into(),
        photo: "".into(),
        createdAt: user.created_at,
        updatedAt: user.updated_at,
      }
    });
  
    Ok(Json(json_response))
  }