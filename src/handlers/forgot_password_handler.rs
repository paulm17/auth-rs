use std::sync::Arc;
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use diesel::{query_dsl::methods::FilterDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use serde_json::json;
use ulid::Ulid;
use chrono::{Duration, Utc};
use crate::{
  model::ForgotPasswordSchema, schema::{email_confirmation, user, EmailConfirmation, User}, smtp::{self, generate_random_string, EmailBaseParams, EmailParams}, AppState
};

pub async fn forgot_password_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<ForgotPasswordSchema>,
  ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let mut conn = data.db_pool.get().expect("Failed to get connection from pool");
    let email = body.email.to_owned();
    let redirect_to = body.redirect_to.to_owned();
  
    // check whether user email exists
    let user_exists = user::table
    .filter(user::email.eq(email.clone()))
    .first::<User>(&mut conn)
    .optional();

    let user_id = if let Ok(Some(user)) = user_exists {
      user.id
    } else {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "User email does not exist"
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    };
  
    // insert into database        
    let code = generate_random_string();
    let expires = (Utc::now() + Duration::days(10)).naive_utc();
    let timestamp = Utc::now().naive_utc();
    let statement = diesel::insert_into(email_confirmation::table)
      .values(&EmailConfirmation {
        id: Ulid::new().to_string(),
        user_id: user_id.into(),
        code: code.clone().into(),
        redirect_to: redirect_to.into(),
        expires,
        flow: "created".into(),
        created_at: timestamp,
        updated_at: None,
        deleted_at: None
      })
      .execute(&mut conn);
    
    if let Err(e) = statement {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": format!("forgot password code not saved to database: validation error\nDetails: {:?}", e)
      });
      return Err((StatusCode::BAD_REQUEST, Json(error_response)));
    }
  
    let base_params = EmailBaseParams {
      from: data.env.mailer_from.clone(),
      from_name: data.env.mailer_from_name.clone(),    
      to: email,
      subject: "Forgot Password".to_string(),
    };

    let params = EmailParams::PasswordReset {
      base: base_params,
      code: code.to_string(),
    };
  
    let result = smtp::send_email(params, axum::extract::State(data)).await;
  
    let mut headers = HeaderMap::new();
    headers.append(
      header::CONTENT_TYPE,
      "application/json".parse().unwrap(),
    );
  
    let mut response = Response::new(
      json!({
        "status": match result {
            Ok(_) => "success",
            Err(_) => "fail"
        }
      })
      .to_string(),
    );
  
    response.headers_mut().extend(headers);
  
    Ok(response)
  }