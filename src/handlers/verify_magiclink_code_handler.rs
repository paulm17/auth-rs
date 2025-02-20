use std::sync::Arc;
use axum::{
  extract::{Query, State}, http::{header, StatusCode}, response::{IntoResponse, Redirect}, Json
};
use anyhow::Result;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::Utc;
use convex::FunctionResult::Value;
use crate::{
  model::VerifyMagicLinkSchema, utils::{generate_token, parse_duration, update_confirm_code}, AppState
};

pub async fn verify_magiclink_code_handler(
  State(data): State<Arc<AppState>>,
  Query(body): Query<VerifyMagicLinkSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let redirect_to = body.redirect_to.to_owned();
  let mut client = data.convex.clone();
  let result = client.query("emailConfirmation:getConfirmationByCode", maplit::btreemap!{
    "code".into() => body.code.to_owned().into(),
    "flow".into() => "created".into(),
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

  let access_token_details = generate_token(
    user_id.to_string(),
    data.env.access_token_max_age,
    data.env.access_token_private_key.to_owned(),
  )?;

  let refresh_token_details = generate_token(
    user_id.to_string(),
    data.env.refresh_token_max_age,
    data.env.refresh_token_private_key.to_owned(),
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

  update_confirm_code(axum::extract::State(data), code_id.to_string(), "completed".to_string()).await;

  let redirect = Redirect::temporary(&redirect_to);
  let mut response = redirect.into_response();

  response.headers_mut().append(
      header::SET_COOKIE,
      access_cookie.to_string().parse().unwrap(),
  );
  response.headers_mut().append(
      header::SET_COOKIE,
      refresh_cookie.to_string().parse().unwrap(),
  );

  Ok(response)
}