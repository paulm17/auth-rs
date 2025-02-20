use axum::{extract::State, http::StatusCode, Json};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::sync::Arc;
use convex::FunctionResult::Value;
use crate::AppState;
use crate::token;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub token_uuid: uuid::Uuid,
    pub user_id: String,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub token_uuid: String,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

pub fn generate_jwt_token(
    user_id: String,
    ttl: i64,
    private_key: String,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_private_key = general_purpose::STANDARD.decode(private_key).unwrap();
    let decoded_private_key = String::from_utf8(bytes_private_key).unwrap();

    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        user_id,
        token_uuid: Uuid::new_v4(),
        expires_in: Some((now + chrono::Duration::minutes(ttl)).timestamp()),
        token: None,
    };

    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        token_uuid: token_details.token_uuid.to_string(),
        exp: token_details.expires_in.unwrap(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };

    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &jsonwebtoken::EncodingKey::from_rsa_pem(decoded_private_key.as_bytes())?,
    )?;
    token_details.token = Some(token);
    Ok(token_details)
}

pub fn verify_jwt_token(
    public_key: String,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_public_key = general_purpose::STANDARD.decode(public_key).unwrap();
    let decoded_public_key = String::from_utf8(bytes_public_key).unwrap();

    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
        &validation,
    )?;

    Ok(TokenDetails {
        token: None,
        token_uuid: Uuid::parse_str(decoded.claims.token_uuid.as_str()).unwrap(),
        user_id: decoded.claims.sub,
        expires_in: None,
    })
}

pub async fn blacklist_token(
  State(data): State<Arc<AppState>>,
  _token: String
) -> Result<bool, (StatusCode, Json<serde_json::Value>)> {
  let mut client = data.convex.clone();

  let token_details =
    match token::verify_jwt_token(data.env.access_token_public_key.to_owned(), &_token)
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

  let token_result = client.query("tokens:getTokenById", maplit::btreemap!{
    "id".into() => token_details.token_uuid.to_string().into()
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

  let token_result = client.mutation("tokens:blacklistToken", maplit::btreemap!{
    "id".into() => _id.into()
  }).await;

  Ok(token_result.is_ok())
}