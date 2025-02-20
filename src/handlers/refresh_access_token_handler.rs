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
use convex::FunctionResult::Value;
use chrono::{DateTime, Duration, Utc};
use crate::{
  token::{self, blacklist_token}, utils::{generate_token, parse_duration}, AppState
};

pub async fn refresh_access_token_handler(
  cookie_jar: CookieJar,
  State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

  let refresh_token_details =
    match token::verify_jwt_token(data.env.refresh_token_public_key.to_owned(), &refresh_token)
    {
      Ok(token_details) => token_details,
      Err(e) => {
        let error_response = serde_json::json!({
          "status": "fail",
          "message": format_args!("{:?}", e)
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
      }
    };  

  let mut client = data.convex.clone();
  
  let token_result = client.query("tokens:getTokenById", maplit::btreemap!{
    "id".into() => refresh_token_details.token_uuid.to_string().into()
  }).await;

  let user_id = match &token_result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("userId")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format_args!("Token is invalid or session has expired {:?}", e)
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
    _ => "",
  };  

  let access_token_details = generate_token(
    user_id.into(),
    data.env.access_token_max_age,
    data.env.access_token_private_key.to_owned(),
  )?;

  let access_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
  client.mutation("tokens:insertToken", maplit::btreemap!{
    "id".into() => access_token_details.token_uuid.to_string().into(),
    "userId".into() => user_id.into(),
    "token".into() => access_token_details.token.clone().unwrap().into(),
    "expires".into() => access_token_details.expires_in.clone().unwrap().into(),
    "createdAt".into() => access_token_timestamp_float.into(),
  }).await
  .map_err(|_| {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "access token not saved to database"
    });
    (StatusCode::FORBIDDEN, Json(error_response))
  })?;

  let access_cookie = Cookie::build(
    ("access_token",
    access_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.access_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::None)
    .http_only(false);

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
    header::CONTENT_TYPE,
    "application/json".parse().unwrap(),
  );

  let expires = match &token_result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("expires")
      .and_then(|v: &convex::Value| match v {
        convex::Value::Float64(s) => Some(s),
        _ => None
      })
      .unwrap_or(&0.0),
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format_args!("Token is invalid or session has expired {:?}", e)
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
    _ => &0.0,
  };  

  // Convert expires (which is in milliseconds since epoch) to DateTime<Utc>
  let expiry_time = DateTime::<Utc>::from_timestamp((*expires / 1000.0) as i64, 0).unwrap();
  let current_time = Utc::now();
  let time_until_expiry = expiry_time.signed_duration_since(current_time);

  if time_until_expiry <= Duration::hours(1) {
    // blacklist old refresh token
    if let Ok(false) = blacklist_token(axum::extract::State(data.clone()), refresh_token).await {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "Failed to blacklist refresh token"
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // recreate refresh_token
    let refresh_token_details = generate_token(
      user_id.into(),
      data.env.refresh_token_max_age,
      data.env.refresh_token_private_key.to_owned(),
    )?;

    let refresh_token_timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
    client.mutation("tokens:insertToken", maplit::btreemap!{
      "id".into() => refresh_token_details.token_uuid.to_string().into(),
      "userId".into() => user_id.into(),
      "token".into() => refresh_token_details.token.clone().unwrap().into(),
      "expires".into() => refresh_token_details.expires_in.clone().unwrap().into(),
      "createdAt".into() => refresh_token_timestamp_float.into(),
    }).await
    .map_err(|_| {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "refresh token not saved to database"
      });
      (StatusCode::FORBIDDEN, Json(error_response))
    })?;    

    let refresh_cookie = Cookie::build(
      ("refresh_token",
      refresh_token_details.token.clone().unwrap_or_default()),
    )
    .path("/")
    .max_age(time::Duration::seconds(parse_duration(&data.env.refresh_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::None)
    .http_only(false);

    headers.append(
      header::SET_COOKIE,
      refresh_cookie.to_string().parse().unwrap(),
    );
  }

  response.headers_mut().extend(headers);
  Ok(response)
}