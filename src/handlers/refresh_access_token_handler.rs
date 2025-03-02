use std::sync::Arc;
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use axum_extra::extract::{
  cookie::{Cookie, SameSite},
  CookieJar,
};
use anyhow::Result;
use diesel::RunQueryDsl;
use serde_json::json;
use chrono::{DateTime, Duration, Utc, TimeZone};
use ulid::Ulid;
use crate::{
  schema::{tokens, Token}, token::{self, blacklist_token, generate_paseto_token}, utils::parse_duration, AppState
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
    match token::verify_paseto_token(&data.paseto.refresh_key, &refresh_token)
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

  let user_id = refresh_token_details.user_id;

  let access_token_details = generate_paseto_token(
    user_id.clone().into(),
    data.env.access_token_max_age,
    &data.paseto.access_key,
  ).unwrap();

  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

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
      token: access_token_details.token.clone().unwrap().into(),
      token_uuid: access_token_details.token_uuid.to_string(),
      expires,
      blacklisted: false,
      created_at: timestamp,
      updated_at: None,
      deleted_at: None
    })
    .get_result::<Token>(&mut conn);

  if let Err(e) = statement {
    let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", e);
    let error_response = serde_json::json!({
      "status": "fail",
      "message": error_message
    });
    return Err((StatusCode::BAD_REQUEST, Json(error_response)));
  }

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

  // Convert expires (which is in milliseconds since epoch) to DateTime<Utc>
  let expiry_time: DateTime<Utc> = Utc.from_utc_datetime(&expires);
  let current_time = Utc::now();
  let time_until_expiry = expiry_time.signed_duration_since(current_time);

  if time_until_expiry <= Duration::hours(1) {
    // blacklist old refresh token
    if let Ok(false) = blacklist_token(axum::extract::State(data.clone()), &data.paseto.refresh_key, &refresh_token).await {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "Failed to blacklist refresh token"
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

    // recreate refresh_token
    let refresh_token_details = generate_paseto_token(
      user_id.clone().into(),
      data.env.refresh_token_max_age,
      &data.paseto.refresh_key,
    ).unwrap();

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
        token: refresh_token_details.token.clone().unwrap().into(),
        token_uuid: refresh_token_details.token_uuid.to_string(),
        expires,
        blacklisted: false,
        created_at: timestamp,
        updated_at: None,
        deleted_at: None
      })
      .get_result::<Token>(&mut conn);

    if let Err(e) = statement {
      let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", e);
      let error_response = serde_json::json!({
          "status": "fail",
          "message": error_message
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }

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