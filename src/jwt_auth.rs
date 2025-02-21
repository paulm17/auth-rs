use std::sync::Arc;

use axum::{
  extract::State,
  http::{header, Request, StatusCode},
  middleware::Next,
  response::IntoResponse,
  Json, body::Body,
};

use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use convex::FunctionResult::Value;

use crate::{model::User, token, AppState};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub status: &'static str,
  pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JWTAuthMiddleware {
  pub user: User,
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
    match token::verify_jwt_token(data.rsa.access_tokens.public_key.to_owned(), &access_token) {
      Ok(token_details) => token_details,
      Err(e) => {
        let error_response = ErrorResponse {
          status: "fail",
          message: format!("{:?}", e),
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

  let mut client = data.convex.clone();
  let user_result = client.query("users:getUserbyId", maplit::btreemap!{
    "id".into() => user_id.to_string().into()
  }).await;

  let user = match &user_result {
    Ok(Value(convex::Value::Object(obj))) => {
      User {
        id: obj.get("id")
          .and_then(|v: &convex::Value| match v {
            convex::Value::String(s) => Some(s.into()),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user id".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        email: obj.get("email")
          .and_then(|v: &convex::Value| match v {
            convex::Value::String(s) => Some(s.into()),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user email".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        name: obj.get("name")
          .and_then(|v: &convex::Value| match v {
            convex::Value::String(s) => Some(s.into()),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user name".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        role: obj.get("role")
          .and_then(|v: &convex::Value| match v {
            convex::Value::String(s) => Some(s.into()),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user role".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        photo: obj.get("photo")
          .and_then(|v: &convex::Value| match v {
            convex::Value::String(s) => Some(s.to_string()),
            _ => None
          })
          .unwrap_or_else(|| "".to_string()),
        verified: obj.get("verified")
          .and_then(|v: &convex::Value| match v {
            convex::Value::Boolean(s) => Some(s.clone()),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user verified".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        created_at: obj.get("createdAt")
          .and_then(|v: &convex::Value| match v {
            convex::Value::Float64(ts) => {
              // Convert timestamp to DateTime<Utc>
              let secs = (*ts as i64) / 1000;
              let nsecs = ((*ts as i64) % 1000) * 1_000_000;
              Some(Some(DateTime::from_timestamp(secs, nsecs as u32).unwrap_or_default()))
            }
            // Keep the string parsing as fallback
            convex::Value::String(s) => Some(DateTime::parse_from_rfc3339(s.as_str())
              .ok()
              .map(|dt| dt.with_timezone(&Utc))),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user created_at".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        updated_at: obj.get("createdAt")
          .and_then(|v: &convex::Value| match v {
            convex::Value::Float64(ts) => {
              // Convert timestamp to DateTime<Utc>
              let secs = (*ts as i64) / 1000;
              let nsecs = ((*ts as i64) % 1000) * 1_000_000;
              Some(Some(DateTime::from_timestamp(secs, nsecs as u32).unwrap_or_default()))
            }
            // Keep the string parsing as fallback
            convex::Value::String(s) => Some(DateTime::parse_from_rfc3339(s.as_str())
                .ok()
                .map(|dt| dt.with_timezone(&Utc))),
            _ => None
          })
          .ok_or_else(|| {
            let error_response = ErrorResponse {
              status: "fail",
              message: "Invalid user created_at".to_string(),
            };
            (StatusCode::BAD_REQUEST, Json(error_response))
          })?,
        }
    },
    _ => {
      let error_response = ErrorResponse {
        status: "fail",
        message: "Error fetching user from database".to_string(),
      };
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
  };

  req.extensions_mut().insert(JWTAuthMiddleware {
    user,
    access_token_uuid,
  });
  Ok(next.run(req).await)
}
