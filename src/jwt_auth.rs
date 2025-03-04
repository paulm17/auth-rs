use std::sync::Arc;

use axum::{
  extract::State,
  http::{header, Request, StatusCode},
  middleware::Next,
  response::IntoResponse,
  Json, body::Body,
};

use axum_extra::extract::cookie::CookieJar;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde::Serialize;

use crate::{schema::{user, User}, token, AppState};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub status: &'static str,
  pub message: String,
}

#[derive(Clone)]
pub struct JWTAuthMiddleware {
  pub user: User,
  #[allow(dead_code)]
  pub access_token_uuid: uuid::Uuid,
}

pub async fn auth(
  cookie_jar: CookieJar,
  State(data): State<Arc<AppState>>,
  mut req: Request<Body>,
  next: Next,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
  let access_token = cookie_jar
    .get("access_token")
    .map(|cookie| cookie.value().to_string())
    .or_else(|| {
      req.headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| {
            if auth_value.starts_with("Bearer ") {
                Some(auth_value[7..].to_owned())
            } else {
                None
            }
        })
    });

  let access_token = access_token.ok_or_else(|| {
    let error_response = ErrorResponse {
      status: "fail",
      message: "You are not logged in, please provide token".to_string(),
    };
    (StatusCode::UNAUTHORIZED, Json(error_response))
  })?;  

  let access_token_details =
    match token::verify_paseto_token(&data.env.auth_key, &access_token) {
      Ok(token_details) => token_details,
      Err(e) => {
        let error_response = ErrorResponse {
          status: "fail",
          message: format!("verify_paseto_token {:?}", e),
        };
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
      }
    };

  let access_token_uuid = uuid::Uuid::parse_str(&access_token_details.token_uuid.to_string())
    .map_err(|_| {
      let error_response = ErrorResponse {
        status: "fail",
        message: "Invalid token".to_string(),
      };
      (StatusCode::UNAUTHORIZED, Json(error_response))
    })?;

  let user_id = access_token_details.user_id.to_string();

  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  
  let user_result = user::table
    .filter(user::id.eq(user_id.to_string()))
    .first::<User>(&mut conn)
    .optional();

  let user = match user_result {
    Ok(Some(user)) => user,
    Ok(None) => {
      let error_response = ErrorResponse {
        status: "fail",
        message: "User not found".to_string(),
      };
      return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }
    Err(_) => {
      let error_response = ErrorResponse {
        status: "fail",
        message: "Error fetching user from database".to_string(),
      };
      return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
    }
  };

  req.extensions_mut().insert(JWTAuthMiddleware {
    user,
    access_token_uuid,
  });

  Ok(next.run(req).await)
}
