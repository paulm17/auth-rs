use oauth2::{
  AccessToken, EmptyExtraTokenFields, RefreshToken, Scope, TokenResponse
};
use oauth2::basic::BasicTokenType;
use serde::de::{self, DeserializeOwned};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(untagged)]
enum ScopeField {
  String(String),
  Array(Vec<String>),
}

fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<Vec<Scope>>, D::Error>
where
  D: de::Deserializer<'de>,
{
  let scope_field = ScopeField::deserialize(deserializer)?;
  let scopes = match scope_field {
      ScopeField::String(s) => s
          .split_whitespace()
          .map(|s| Scope::new(s.to_string()))
          .collect(),
      ScopeField::Array(arr) => arr
          .into_iter()
          .map(Scope::new)
          .collect(),
  };
  Ok(Some(scopes))
}

#[derive(Serialize, Debug, Deserialize)]
pub struct TwitchTokenResponse {
  pub access_token: AccessToken,
  pub token_type: BasicTokenType,
  pub expires_in: Option<u64>,
  pub refresh_token: Option<RefreshToken>,
  #[serde(deserialize_with = "deserialize_scope")]
  pub scope: Option<Vec<Scope>>,
  #[serde(flatten)]
  pub extra_fields: EmptyExtraTokenFields,
}

impl TokenResponse for TwitchTokenResponse {
  type TokenType = BasicTokenType;

  fn access_token(&self) -> &AccessToken {
      &self.access_token
  }
  fn token_type(&self) -> &Self::TokenType {
      &self.token_type
  }
  fn expires_in(&self) -> Option<Duration> {
      self.expires_in.map(Duration::from_secs)
  }
  fn refresh_token(&self) -> Option<&RefreshToken> {
      self.refresh_token.as_ref()
  }
  fn scopes(&self) -> Option<&Vec<Scope>> {
      self.scope.as_ref()
  }
}

// ------

// use oauth2::{
//   AccessToken, EmptyExtraTokenFields, RefreshToken, Scope, TokenResponse
// };
// use oauth2::basic::BasicTokenType;
// use serde::de::{self, DeserializeOwned};
// use serde::{Deserialize, Serialize};
// use std::fmt;
// use std::time::Duration;

// #[derive(Deserialize)]
// #[serde(untagged)]
// enum ScopeField {
//   String(String),
//   Array(Vec<String>),
// }

// fn deserialize_scope<'de, D>(deserializer: D) -> Result<Option<Vec<Scope>>, D::Error>
// where
//   D: de::Deserializer<'de>,
// {
//   let scope_field = ScopeField::deserialize(deserializer)?;
//   let scopes = match scope_field {
//       ScopeField::String(s) => s
//           .split_whitespace()
//           .map(|s| Scope::new(s.to_string()))
//           .collect(),
//       ScopeField::Array(arr) => arr
//           .into_iter()
//           .map(Scope::new)
//           .collect(),
//   };
//   Ok(Some(scopes))
// }

// #[derive(Serialize, Debug, Deserialize)]
// pub struct TwitchTokenResponse {
//   pub access_token: AccessToken,
//   pub token_type: BasicTokenType,
//   pub expires_in: Option<u64>,
//   pub refresh_token: Option<RefreshToken>,
//   #[serde(deserialize_with = "deserialize_scope")]
//   pub scope: Option<Vec<Scope>>,
//   #[serde(flatten)]
//   pub extra_fields: EmptyExtraTokenFields,
// }

// impl TokenResponse for TwitchTokenResponse {
//   type TokenType = BasicTokenType;

//   fn access_token(&self) -> &AccessToken {
//       &self.access_token
//   }
//   fn token_type(&self) -> &Self::TokenType {
//       &self.token_type
//   }
//   fn expires_in(&self) -> Option<Duration> {
//       self.expires_in.map(Duration::from_secs)
//   }
//   fn refresh_token(&self) -> Option<&RefreshToken> {
//       self.refresh_token.as_ref()
//   }
//   fn scopes(&self) -> Option<&Vec<Scope>> {
//       self.scope.as_ref()
//   }
// }

