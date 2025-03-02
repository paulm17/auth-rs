use std::sync::Arc;

use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use axum_extra::extract::{
  cookie::{Cookie, SameSite},
  CookieJar,
};
use anyhow::Result;
use serde_json::json;
use crate::{token::blacklist_token, AppState};

pub async fn logout_handler(
  cookie_jar: CookieJar,
  State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let access_cookie = Cookie::build(("access_token", ""))
    .path("/")
    .max_age(time::Duration::minutes(-1))
    .same_site(SameSite::None)
    .http_only(false);
  let refresh_cookie = Cookie::build(("refresh_token", ""))
    .path("/")
    .max_age(time::Duration::minutes(-1))
    .same_site(SameSite::None)
    .http_only(true);
  let logged_in_cookie = Cookie::build(("logged_in", "true"))
    .path("/")
    .max_age(time::Duration::minutes(-1))
    .same_site(SameSite::None)
    .http_only(false);

  let access_token = cookie_jar
    .get("access_token")
    .map(|cookie| cookie.value().to_string())
    .ok_or_else(|| {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "could not delete access token"
      });
      (StatusCode::FORBIDDEN, Json(error_response))
    })?;  

  if let Ok(false) = blacklist_token(axum::extract::State(data.clone()), &data.paseto.access_key, &access_token).await {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Failed to blacklist access token"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let refresh_token = cookie_jar
    .get("refresh_token")
    .map(|cookie| cookie.value().to_string())
    .ok_or_else(|| {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "could not delete refresh token"
      });
      (StatusCode::FORBIDDEN, Json(error_response))
    })?;

  if let Ok(false) = blacklist_token(axum::extract::State(data.clone()), &data.paseto.refresh_key, &refresh_token).await {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Failed to blacklist refresh token"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let mut headers = HeaderMap::new();
  headers.append(
    header::SET_COOKIE,
    access_cookie.to_string().parse().unwrap(),
  );
  headers.append(
    header::SET_COOKIE,
    refresh_cookie.to_string().parse().unwrap(),
  );
  headers.append(
    header::SET_COOKIE,
    logged_in_cookie.to_string().parse().unwrap(),
  );

  let mut response = Response::new(json!({"status": "success"}).to_string());
  response.headers_mut().extend(headers);

  Ok(response)
}