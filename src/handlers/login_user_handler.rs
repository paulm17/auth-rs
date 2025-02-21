use std::sync::Arc;
use argon2::{
  password_hash::{PasswordHash, PasswordVerifier},
  Argon2,
};
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use axum_extra::extract::cookie::{Cookie, SameSite};
use anyhow::Result;
use serde_json::json;
use convex::FunctionResult::Value;
use chrono::Utc;
use crate::{
  model::LoginUserSchema, utils::{generate_token, parse_duration}, AppState
};

pub async fn login_user_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let email = body.email.to_owned().to_ascii_lowercase();
  let mut client = data.convex.clone();
  let user_exists = client.query("users:getUserbyEmail", maplit::btreemap!{
    "email".into() => email.into()
  }).await;

  let user_id = match &user_exists {
    Ok(Value(convex::Value::Object(obj))) => obj.get("id")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  let password_hash = match &user_exists {
    Ok(Value(convex::Value::Object(obj))) => obj.get("password")
      .and_then(|v| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };
  
  let is_valid = match PasswordHash::new(password_hash) {
    Ok(parsed_hash) => Argon2::default()
      .verify_password(body.password.as_bytes(), &parsed_hash)
      .map_or(false, |_| true),
    Err(_) => false,
  };    

  if !is_valid {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Invalid email or password"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }    

  let access_token_details = generate_token(
    user_id.to_string(),
    data.env.access_token_max_age,
    data.rsa.access_tokens.private_key.to_owned()
  )?;

  let refresh_token_details = generate_token(
    user_id.to_string(),
    data.env.refresh_token_max_age,
    data.rsa.refresh_tokens.private_key.to_owned(),
  )?;

  let access_cookie = Cookie::build(
    ("access_token",
    access_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.access_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::None)
    .http_only(false);

  let refresh_cookie = Cookie::build(
    ("refresh_token",
    refresh_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.refresh_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::None)
    .http_only(true);

  let access_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let result = client.mutation("tokens:insertToken", maplit::btreemap!{
    "id".into() => access_token_details.token_uuid.to_string().into(),
    "userId".into() => user_id.into(),
    "token".into() => access_token_details.token.clone().unwrap().into(),
    "expires".into() => (access_token_details.expires_in.clone().unwrap() as f64).into(),
    "createdAt".into() => access_token_timestamp_float.into(),
  }).await;
  
  if format!("{:?}", result).contains("Server Error") {
    let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", result);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let refresh_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  let result = client.mutation("tokens:insertToken", maplit::btreemap!{
    "id".into() => refresh_token_details.token_uuid.to_string().into(),
    "userId".into() => user_id.into(),
    "token".into() => refresh_token_details.token.clone().unwrap().into(),
    "expires".into() => (refresh_token_details.expires_in.clone().unwrap() as f64).into(),
    "createdAt".into() => refresh_token_timestamp_float.into(),
  }).await;

  if format!("{:?}", result).contains("Server Error") {
    let error_message = format!("Failed to save refresh token: validation error\nDetails: {:?}", result);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let mut response = Response::new(
    json!({"status": "success", "access_token": access_token_details.token.unwrap()})
        .to_string(),
  );
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
    header::CONTENT_TYPE,
    "application/json".parse().unwrap(),
  );

  response.headers_mut().extend(headers);

  Ok(response)
}
