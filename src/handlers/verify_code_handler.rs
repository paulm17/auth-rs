use std::sync::Arc;
use axum::{
  extract::{Query, State}, http::StatusCode, response::{IntoResponse, Redirect}, Json
};
use anyhow::Result;
use convex::FunctionResult::Value;
use chrono::{DateTime, Utc};
use crate::{
  model::VerifyCodeSchema, utils::update_confirm_code, AppState
};

pub async fn verify_code_handler(
  State(data): State<Arc<AppState>>,
  Query(body): Query<VerifyCodeSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
  let code = body.code.to_owned();

  // verify code exists and has not been used
  let mut client = data.convex.clone();
  let result = client.query("emailConfirmation:getConfirmationByCode", maplit::btreemap!{
    "code".into() => code.into(),
    "flow".into() => "created".into(),
  }).await;

  let code = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("code")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  if code == "" {
    let error_response = serde_json::json!({
        "status": "fail",
        "message": "verification code does not exist",
    });
    return Err((StatusCode::CONFLICT, Json(error_response)));
  }

  let redirect_to = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("redirectTo")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  let expires = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("expires")
      .and_then(|v: &convex::Value| match v {
        convex::Value::Float64(s) => Some(s),
        _ => None
      })
      .unwrap_or(&0.0),
    Err(_) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": "Code is invalid or has expired"
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
    _ => &0.0,
  };  

  let current_time = Utc::now();
  let expiry_time = DateTime::<Utc>::from_timestamp((*expires / 1000.0) as i64, 0).unwrap();

  if current_time > expiry_time {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": "Code is invalid or has expired"
    });
    return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
  }

  let id = match &result {
    Ok(Value(convex::Value::Object(obj))) => obj.get("_id")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  update_confirm_code(axum::extract::State(data), id.to_string(), "seen".to_string()).await;

  Ok(Redirect::temporary(&format!("{}?code={}", redirect_to, code)))
}