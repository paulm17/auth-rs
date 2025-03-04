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
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde_json::json;
use chrono::{DateTime, Utc};
use ulid::Ulid;
use crate::{
  model::LoginUserSchema, schema::{tokens, user, Token, User}, token::generate_paseto_token, utils::parse_duration, AppState
};

pub async fn login_user_handler(
  State(data): State<Arc<AppState>>,
  Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let email = body.email.to_owned().to_ascii_lowercase();
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
  
  let user_exists = user::table
    .filter(user::email.eq(email.clone()))
    .first::<User>(&mut conn)
    .optional();

  let user = if let Ok(Some(user)) = user_exists {
    user
  } else {
    let error_response = serde_json::json!({
        "status": "fail",
        "message": "Invalid email or password"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  };

  let user_id = user.id.as_str();
  let password_hash = user.password.unwrap();
  
  let is_valid_password = match PasswordHash::new(&password_hash) {
    Ok(parsed_hash) => Argon2::default()
      .verify_password(body.password.as_bytes(), &parsed_hash)
      .map_or(false, |_| true),
    Err(_) => false,
  };    

  if !is_valid_password {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Invalid email or password"
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }    

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

  let expires = DateTime::<Utc>::from_timestamp(refresh_token_details.expires_in.unwrap(), 0)
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
