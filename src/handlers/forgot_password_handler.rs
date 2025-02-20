use std::sync::Arc;
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use serde_json::json;
use convex::FunctionResult::Value;
use ulid::Ulid;
use chrono::{DateTime, Duration, Utc};
use crate::{
  model::ForgotPasswordSchema, smtp::{self, generate_random_string, EmailBaseParams, EmailParams}, AppState
};

pub async fn forgot_password_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<ForgotPasswordSchema>,
  ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let email = body.email.to_owned();
    let redirect_to = body.redirect_to.to_owned();
  
    // check whether user email exists
    let mut client = data.convex.clone();
    let user_exists = client.query("users:getUserbyEmail", maplit::btreemap!{
      "email".into() => email.clone().into(),
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
  
    if user_id == "" {
      let error_response = serde_json::json!({
          "status": "fail",
          "message": "User email does not exist",
      });
      return Err((StatusCode::CONFLICT, Json(error_response)));
    }
  
    // insert into database
    let code = generate_random_string();
    let expires: DateTime<Utc> = Utc::now() + Duration::days(10);
    let expires_timestamp = expires.timestamp() as f64 * 1000.0;
    let timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
    
    let result = client.mutation("emailConfirmation:insertCode", maplit::btreemap!{
      "id".into() => Ulid::new().to_string().into(),
      "userId".into() => user_id.into(),
      "code".into() => code.clone().into(),    
      "redirectTo".into() => redirect_to.into(),
      "expires".into() => expires_timestamp.into(),
      "flow".into() => "created".into(),
      "createdAt".into() => timestamp_float.into(),
    }).await;
  
    if format!("{:?}", result).contains("Server Error") {
      println!("{:?}", result);
  
      let error_message = "forgot password code not saved to database";
      let error_response = serde_json::json!({
          "status": "fail",
          "message": error_message
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