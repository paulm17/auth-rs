use axum::Json;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use convex::ConvexClient;
use rand::{rngs::StdRng, SeedableRng, Rng};
use reqwest::StatusCode;
use sha2::{Sha256, Digest};
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use convex::FunctionResult::Value;
use toml::{Value as TomlValue, map::Map};
use ulid::Ulid;

#[derive(Debug, Clone)]
pub struct KeyPair {
  pub private_key: String,
  pub public_key: String,
}

#[derive(Debug, Clone)]
pub struct RsaConfig {
  pub access_tokens: KeyPair,
  pub refresh_tokens: KeyPair
}

#[derive(Debug, PartialEq, Eq)]
pub enum TokenType  {
  AccessToken,
  RefreshToken
}

pub fn generate_rsa_key_pair(auth_token: &str, token_type: TokenType) -> Result<KeyPair, Box<dyn std::error::Error>> {
  let mut hasher = Sha256::new();
  hasher.update(auth_token.as_bytes());

  if token_type == TokenType::RefreshToken {
      hasher.update(b"refresh");
  } else {
      hasher.update(b"access");
  }

  let seed = hasher.finalize();
  let seed_array: [u8; 32] = seed.into();
  let mut rng = StdRng::from_seed(seed_array);
  
  // Generate the private key
  let private_key = rsa::RsaPrivateKey::new(&mut rng, 4096)?;
  
  // Extract the public key
  let public_key = private_key.to_public_key();

  // Convert both keys to PEM format
  let private_pem = private_key.to_pkcs8_pem(LineEnding::LF)?;
  let public_pem = public_key.to_public_key_pem(LineEnding::LF)?;
  
  // Return both keys encoded in base64
  Ok(KeyPair {
    private_key: general_purpose::STANDARD.encode(private_pem.as_bytes()),
    public_key: general_purpose::STANDARD.encode(public_pem.as_bytes()),
  })
}

pub fn gen_tokens(auth_token: &str) -> Result<RsaConfig, Box<dyn std::error::Error>> {
  let access_tokens = generate_rsa_key_pair(&auth_token, TokenType::AccessToken).map_err(|e| {
    let error_response = serde_json::json!({
      "status": "error",
      "message": format!("error generating RSA key pair: {}", e),
    });
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
  }).unwrap();

  let refresh_tokens = generate_rsa_key_pair(&auth_token, TokenType::RefreshToken).map_err(|e| {
    let error_response = serde_json::json!({
      "status": "error",
      "message": format!("error generating RSA key pair: {}", e),
    });
    (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
  }).unwrap();

  Ok(RsaConfig{
    access_tokens,
    refresh_tokens
  })
} 

pub async fn init(client: ConvexClient) -> Result<RsaConfig, Box<dyn std::error::Error>> {
  let mut rsa = RsaConfig{
    access_tokens: KeyPair{
      private_key: "".into(),
      public_key: "".into(),
    },
    refresh_tokens: KeyPair{
      private_key: "".into(),
      public_key: "".into(),
    }
  };

  // check whether a file exists with the access/refresh public/private tokens
  if let Ok(content) = std::fs::read_to_string("rsa_tokens.toml") {
    if let Ok(tokens) = content.parse::<TomlValue>() {
      if let Some(table) = tokens.as_table() {
        if let (Some(access), Some(refresh)) = (table.get("access_tokens"), table.get("refresh_tokens")) {
          rsa = RsaConfig {
            access_tokens: KeyPair {
              private_key: access.get("private_key")
                .and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
              public_key: access.get("public_key")
                .and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
            },
            refresh_tokens: KeyPair {
              private_key: refresh.get("private_key")
                .and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
              public_key: refresh.get("public_key")
                .and_then(|v| v.as_str())
                .unwrap_or("").to_string(),
            }
          };
          return Ok(rsa);
        }
      }
    }
  }

  let mut client = client.clone();
  let token_exists = client.query("authTokens:getToken", maplit::btreemap!{}).await;

  let token = match &token_exists {
    Ok(Value(convex::Value::Object(obj))) => obj.get("id")
      .and_then(|v: &convex::Value| match v {
        convex::Value::String(s) => Some(s.as_str()),
        _ => None
      })
      .unwrap_or(""),
    _ => "",
  };

  if token.is_empty() {
    println!("🚀 Please wait generating access/refresh public/private tokens");

    // generate new token
    let generate_random_string = create_random_string_generator(&["a-z", "0-9", "A-Z", "-_"]);
   
    // Generate a 32-character string using a custom alphabet
    let auth_token = generate_random_string(32, Some("A-Z0-9"));

    // save to database
    let timestamp_float = Utc::now().timestamp_millis() as f64 / 1000.0;
    let result = client.mutation("authTokens:insertToken", maplit::btreemap!{
      "id".into() => Ulid::new().to_string().into(),
      "token".into() => auth_token.clone().into(),
      "createdAt".into() => timestamp_float.into()
    }).await;

    if format!("{:?}", result).contains("Server Error") {
      let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", result);
      
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error_message
      )));
    }

    // generate new access/refresh public/private tokens
    rsa = gen_tokens(&auth_token).unwrap();

    // Save the tokens to TOML file
    let mut tokens = Map::new();
    let mut access = Map::new();
    let mut refresh = Map::new();

    access.insert("private_key".into(), TomlValue::String(rsa.access_tokens.private_key.clone()));
    access.insert("public_key".into(), TomlValue::String(rsa.access_tokens.public_key.clone()));
    refresh.insert("private_key".into(), TomlValue::String(rsa.refresh_tokens.private_key.clone()));
    refresh.insert("public_key".into(), TomlValue::String(rsa.refresh_tokens.public_key.clone()));

    tokens.insert("access_tokens".into(), TomlValue::Table(access));
    tokens.insert("refresh_tokens".into(), TomlValue::Table(refresh));

    let toml_string = toml::to_string(&tokens)?;
    std::fs::write("rsa_tokens.toml", toml_string)?;
  }

  Ok(rsa)
}

fn expand_alphabet(pattern: &str) -> String {
  let mut result = String::new();
  let mut chars = pattern.chars().peekable();
  
  while let Some(c) = chars.next() {
    if chars.peek() == Some(&'-') {
      chars.next(); // consume the '-'
      if let Some(end) = chars.next() {
        for ch in c..=end {
          result.push(ch);
        }
      }
    } else {
      result.push(c);
    }
  }
  result
}

fn create_random_string_generator(base_patterns: &[&str]) -> impl Fn(usize, Option<&str>) -> String {
  let base_char_set: String = base_patterns.iter()
    .map(|&pattern| expand_alphabet(pattern))
    .collect();
  
  if base_char_set.is_empty() {
    panic!("No valid characters provided for random string generation.");
  }
  
  let base_chars: Vec<char> = base_char_set.chars().collect();
  
  move |length: usize, alphabet: Option<&str>| {
    if length == 0 {
      panic!("Length must be a positive integer.");
    }
    
    let chars = match alphabet {
      Some(pattern) => expand_alphabet(pattern).chars().collect::<Vec<_>>(),
      None => base_chars.clone(),
    };
      
    let char_set_len = chars.len();
    let mut rng = rand::thread_rng();
    
    (0..length)
      .map(|_| {
        let ch = chars[rng.gen_range(0..char_set_len)];
        // Randomly decide whether to make the character lowercase
        if rng.gen_range(0..2) == 0 {
          ch.to_ascii_lowercase()
        } else {
          ch
        }
      })
      .collect()
  }
}