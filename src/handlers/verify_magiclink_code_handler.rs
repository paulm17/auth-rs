use std::sync::Arc;
use axum::{
  extract::{Query, State}, http::{header, StatusCode}, response::{IntoResponse, Redirect}, Json
};
use anyhow::Result;
use axum_extra::extract::cookie::{Cookie, SameSite};
use chrono::{DateTime, Utc};
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use ulid::Ulid;
use crate::{
  model::VerifyMagicLinkSchema, schema::{email_confirmation, tokens, EmailConfirmation, Token}, token::generate_paseto_token, utils::{parse_duration, update_confirm_code}, AppState
};

pub async fn verify_magiclink_code_handler(
  State(data): State<Arc<AppState>>,
  Query(body): Query<VerifyMagicLinkSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let redirect_to = body.redirect_to.to_owned();
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  
  let confirmation_exists = email_confirmation::table
    .filter(email_confirmation::code.eq(body.code.to_owned()))
    .filter(email_confirmation::flow.eq("created"))
    .first::<EmailConfirmation>(&mut conn)
    .optional();

  let confirmation = if let Ok(Some(confirmation)) = confirmation_exists {
    confirmation
  } else {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Code is invalid or has expired"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  let user_id = confirmation.user_id;
  let code_id = confirmation.code;

  let access_token_details = generate_paseto_token(
    user_id.to_string(),
    data.env.access_token_max_age,
    &data.env.auth_key,
  ).unwrap();

  let refresh_token_details = generate_paseto_token(
    user_id.to_string(),
    data.env.refresh_token_max_age,
    &data.env.auth_key,
  ).unwrap();

  // Save tokens
  // Access token
  let expires = DateTime::<Utc>::from_timestamp(access_token_details.expires_in.unwrap(), 0)
    .map(|dt| dt.naive_utc())
    .unwrap_or_else(|| {
      // Handle invalid timestamps
      DateTime::<Utc>::from_timestamp(0, 0)
        .unwrap()
        .naive_utc()
    });
  let timestamp = Utc::now().naive_utc();
  let statement = diesel::insert_into(tokens::table)
    .values(&Token {
      id: Ulid::new().to_string(),
      user_id: user_id.clone().into(),
      expires,
      blacklisted: false,
      token: access_token_details.token.clone().unwrap(),
      token_uuid: access_token_details.token_uuid.to_string(),
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);
  
  if let Err(e) = statement {
    let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", e);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  // Refresh Token
  let expires = DateTime::<Utc>::from_timestamp(access_token_details.expires_in.unwrap(), 0)
    .map(|dt| dt.naive_utc())
    .unwrap_or_else(|| {
      // Handle invalid timestamps
      DateTime::<Utc>::from_timestamp(0, 0)
        .unwrap()
        .naive_utc()
    });
  let timestamp = Utc::now().naive_utc();
  let statement = diesel::insert_into(tokens::table)
    .values(&Token {
      id: Ulid::new().to_string(),
      user_id: user_id.into(),
      expires,
      blacklisted: false,
      token: refresh_token_details.token.clone().unwrap(),
      token_uuid: refresh_token_details.token_uuid.to_string(),
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .execute(&mut conn);
  
  if let Err(e) = statement {
    let error_message = format!("Failed to save refresh token: validation error\nDetails: {:?}", e);
    let error_response = serde_json::json!({
        "status": "fail",
        "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

  let _ = update_confirm_code(axum::extract::State(data.clone()), code_id.to_string(), "completed".to_string()).await;

  // set cookies
  let access_cookie = Cookie::build(
    ("access_token",
    access_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .secure(true)
    .max_age(time::Duration::seconds(parse_duration(&data.env.access_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::Strict)
    .http_only(true);

  let refresh_cookie = Cookie::build(
    ("refresh_token",
    refresh_token_details.token.clone().unwrap_or_default()),
  )
    .path("/")
    .secure(true)
    .max_age(time::Duration::seconds(parse_duration(&data.env.refresh_token_expires_in).unwrap_or(900))) // 15 minutes default
    .same_site(SameSite::Strict)
    .http_only(true);

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