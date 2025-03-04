use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};
use std::sync::Arc;
use uuid::Uuid;
use crate::schema::{tokens, Token};
use crate::AppState;

// (Make sure to import your Paseto types and builder—this example assumes you’re using v4 local tokens.)
use rusty_paseto::prelude::*;


#[derive(Debug)]
pub struct TokenDetails {
  pub user_id: String,
  pub token_uuid: Uuid,
  pub expires_in: Option<i64>,
  pub token: Option<String>,
}

#[derive(Debug)]
pub struct TokenClaims {
  pub sub: String,
  pub token_uuid: String,
  #[allow(dead_code)]
  pub exp: i64,
  #[allow(dead_code)]
  pub iat: i64,
  #[allow(dead_code)]
  pub nbf: i64,
}

pub fn generate_paseto_token(
  user_id: String,
  ttl: i64,
  secret: &str,
) -> Result<TokenDetails, Box<dyn Error>> {
  let private_key = Key::<64>::try_from(&secret[..]).unwrap();
  let pk: &[u8] = private_key.as_slice();
  let private_key = PasetoAsymmetricPrivateKey::<V4, Public>::from(pk);

  let token_uuid = Uuid::new_v4();
  let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
  let exp = now + (ttl * 60); // ttl in minutes converted to seconds

  let claims = TokenClaims {
    sub: user_id.clone(),
    token_uuid: token_uuid.to_string(),
    exp,
    iat: now,
    nbf: now,
  };

  // Convert timestamps to RFC3339 DateTime strings
  let exp_datetime: DateTime<Utc> = DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(exp as u64));
  let iat_datetime: DateTime<Utc> = DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(now as u64));
  let nbf_datetime: DateTime<Utc> = DateTime::<Utc>::from(UNIX_EPOCH + std::time::Duration::from_secs(now as u64));

  let token = PasetoBuilder::<V4, Public>::default()
    .set_claim(SubjectClaim::from(claims.sub.as_str()))
    .set_claim(CustomClaim::try_from(("token_uuid", claims.token_uuid.clone())).unwrap())
    .set_claim(ExpirationClaim::try_from(exp_datetime.to_rfc3339()).unwrap())
    .set_claim(IssuedAtClaim::try_from(iat_datetime.to_rfc3339()).unwrap())
    .set_claim(NotBeforeClaim::try_from(nbf_datetime.to_rfc3339()).unwrap())
    .build(&private_key)?;

  Ok(TokenDetails {
    user_id,
    token_uuid,
    expires_in: Some(exp),
    token: Some(token),
  })
}

pub fn verify_paseto_token(
  secret: &str,
  token: &str,
) -> Result<TokenDetails, Box<dyn Error>> {
  // Convert hex string to bytes
  let secret_bytes = hex::decode(secret)?;

  let mut key_data = [0u8; 32];
  key_data.copy_from_slice(&secret_bytes[32..]);

  let key = Key::<32>::from(key_data);
  let public_key = PasetoAsymmetricPublicKey::<V4, Public>::from(&key);

  // Create a parser without claim validation first
  let mut parser = PasetoParser::<V4, Public>::default();
  
  // Parse the token first to get raw claims
  let parsed_token = parser.parse(token, &public_key)
    .map_err(|e| format!("Failed to parse token: {}", e))?;

  let claims = parsed_token.as_object().ok_or("Invalid token structure")?;

  // Extract claims manually
  let sub = claims.get("sub")
    .and_then(|v| v.as_str())
    .ok_or("Missing subject claim")?
    .to_string();

  let token_uuid = claims.get("token_uuid")
    .and_then(|v| v.as_str())
    .ok_or("Missing token_uuid claim")?;

  // Validate expiration and not before claims if they exist
  if let Some(exp) = claims.get("exp") {
    // Handle different possible formats for exp
    match exp {
      serde_json::Value::String(exp_str) => {
        // Try to parse as RFC3339 date string first
        if let Ok(exp_datetime) = chrono::DateTime::parse_from_rfc3339(exp_str) {
          if exp_datetime < chrono::Utc::now() {
              return Err("Token has expired".into());
          }
        } else {
          // If not RFC3339, try as numeric timestamp
          match exp_str.parse::<i64>() {
            Ok(exp_timestamp) => {
              if exp_timestamp < chrono::Utc::now().timestamp() {
                return Err("Token has expired".into());
              }
            },
            Err(_) => {
              // Log the error but don't fail validation
              eprintln!("Warning: Could not parse exp claim: {}", exp_str);
            }
          }
        }
      },
      serde_json::Value::Number(num) => {
        if let Some(exp_timestamp) = num.as_i64() {
          if exp_timestamp < chrono::Utc::now().timestamp() {
            return Err("Token has expired".into());
          }
        }
      },
      _ => {
        eprintln!("Warning: exp claim has unexpected format");
      }
    }
  }

  // Similar handling for nbf claim
  if let Some(nbf) = claims.get("nbf") {
    match nbf {
      serde_json::Value::String(nbf_str) => {
        if let Ok(nbf_datetime) = chrono::DateTime::parse_from_rfc3339(nbf_str) {
          if nbf_datetime > chrono::Utc::now() {
            return Err("Token not yet valid".into());
          }
        } else {
          match nbf_str.parse::<i64>() {
            Ok(nbf_timestamp) => {
              if nbf_timestamp > chrono::Utc::now().timestamp() {
                return Err("Token not yet valid".into());
              }
            },
            Err(_) => {
              eprintln!("Warning: Could not parse nbf claim: {}", nbf_str);
            }
          }
        }
      },
      serde_json::Value::Number(num) => {
        if let Some(nbf_timestamp) = num.as_i64() {
          if nbf_timestamp > chrono::Utc::now().timestamp() {
            return Err("Token not yet valid".into());
          }
        }
      },
      _ => {
        eprintln!("Warning: nbf claim has unexpected format");
      }
    }
  }

  Ok(TokenDetails {
    token: None,
    token_uuid: Uuid::parse_str(token_uuid)?,
    user_id: sub,
    expires_in: None,
  })
}

pub async fn blacklist_token(
  State(data): State<Arc<AppState>>,
  secret: &str,
  token: &str,
) -> Result<bool, (StatusCode, Json<serde_json::Value>)> {
  let mut conn = data.db_pool.get().expect("Failed to get connection from pool");

  let token_details = match self::verify_paseto_token(secret, &token) {
    Ok(details) => details,
    Err(e) => {
      let error_response = serde_json::json!({
        "status": "fail",
        "message": format!("{:?}", e)
      });
      return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }
  };

  let token_exists = match tokens::table
    .filter(tokens::token_uuid.eq(token_details.token_uuid.to_string()))
    .first::<Token>(&mut conn)
    .optional() {
      Ok(Some(token)) => token,
      Ok(None) => {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token not found"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
      }
      Err(e) => {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("{:?}", e)
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
      }
    };

  let statement = diesel::delete(tokens::table)
    .filter(tokens::token.eq(token_exists.token))
    .execute(&mut conn);

  if let Err(e) = statement {
    let error_response = serde_json::json!({
      "status": "fail",
      "message": format!("Token not found, {}", e)
    });
    return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
  }

  Ok(true)
}