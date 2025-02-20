use std::sync::Arc;
use axum::{
  extract::State, http::{header, HeaderMap, Response, StatusCode}, response::IntoResponse, Json
};
use anyhow::Result;
use serde_json::json;
use convex::FunctionResult::Value;
use crate::{
  model::CheckCodeSchema, AppState
};

pub async fn check_code_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<CheckCodeSchema>,
  ) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let code = body.code.to_owned();
  
    // check whether code valid
    let mut client = data.convex.clone();
    let result = client.query("emailConfirmation:getConfirmationByCode", maplit::btreemap!{
      "code".into() => code.into(),
      "flow".into() => "seen".into(),
    }).await;
  
    let flow = match &result {
      Ok(Value(convex::Value::Object(obj))) => obj.get("flow")
        .and_then(|v: &convex::Value| match v {
          convex::Value::String(s) => Some(s.as_str()),
          _ => None
        })
        .unwrap_or(""),
      _ => "",
    };
  
    if flow != "seen" {
      let error_response = serde_json::json!({
        "is_valid": false
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
  
    let mut response = Response::new(
      json!({"is_valid": true})
        .to_string(),
    );
  
    let mut headers = HeaderMap::new();
    headers.append(
      header::CONTENT_TYPE,
      "application/json".parse().unwrap(),
    );
  
    response.headers_mut().extend(headers);
  
    Ok(response)
  }