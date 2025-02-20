use chrono::prelude::*;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
  pub id: String,
  pub name: String,
  pub email: String,
  pub role: String,
  pub photo: String,
  pub verified: bool,
  #[serde(rename = "createdAt")]
  pub created_at: Option<DateTime<Utc>>,
  #[serde(rename = "updatedAt")]
  pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RegisterUserSchema {
  pub name: String,
  pub email: String,
  pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserSchema {
  pub email: String,
  pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordSchema {
  pub email: String,
  #[serde(rename = "redirectTo")]
  pub redirect_to: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyCodeSchema {
  pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct CheckCodeSchema {
  pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct ResetPasswordSchema {
  pub code: String,
  pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct MagicLinkSchema {
  pub email: String,  
  #[serde(rename = "redirectTo")]
  pub redirect_to: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyMagicLinkSchema {
  pub code: String,
  pub redirect_to: String,
}

#[derive(Debug, Deserialize)]
pub struct OAuthSchema {
  pub provider: String,
  pub scopes: String,
  pub callback_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PkceChallenge {
  pub challenge: String,
  pub method: String,
}

