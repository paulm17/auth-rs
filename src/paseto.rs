use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use diesel::OptionalExtension;
use diesel::RunQueryDsl;
use diesel::{r2d2::ConnectionManager, PgConnection};
use r2d2::Pool;
use rand::{rngs::StdRng, Rng, RngCore, SeedableRng};
use sha2::{Sha256, Digest};
use toml::{Value as TomlValue, map::Map};
use ulid::Ulid;

use crate::schema::auth_tokens;
use crate::schema::AuthToken;

#[derive(Debug, Clone)]
pub struct PasetoConfig {
  pub access_key: String,
  pub refresh_key: String,
}

// Deterministic key generation for Paseto tokens
pub fn generate_paseto_secret_deterministic(
  auth_token: &str,
  token_type: &str
) -> Result<String, Box<dyn std::error::Error>> {
  let mut hasher = Sha256::new();
  hasher.update(auth_token.as_bytes());
  hasher.update(token_type.as_bytes());
  let seed = hasher.finalize();
  let seed_array: [u8; 32] = seed.into();
  let mut rng = StdRng::from_seed(seed_array);
  let mut key = [0u8; 32];
  rng.fill_bytes(&mut key);
  Ok(general_purpose::STANDARD.encode(&key))
}

pub async fn init(db_pool: Pool<ConnectionManager<PgConnection>>) -> Result<PasetoConfig, Box<dyn std::error::Error>> {
  let mut config = PasetoConfig {
    access_key: "".into(),
    refresh_key: "".into(),
  };

  // Check if a TOML file exists containing the symmetric keys
  if let Ok(content) = std::fs::read_to_string("paseto_tokens.toml") {
    if let Ok(tokens) = content.parse::<TomlValue>() {
      if let Some(table) = tokens.as_table() {
        if let (Some(access), Some(refresh)) = (table.get("access_key"), table.get("refresh_key")) {
          config = PasetoConfig {
            access_key: access.as_str().unwrap_or("").to_string(),
            refresh_key: refresh.as_str().unwrap_or("").to_string(),
          };
          return Ok(config);
        }
      }
    }
  }

  let mut conn = db_pool.get().expect("Failed to get connection from pool");

  let token_exists = auth_tokens::table
    .first::<AuthToken>(&mut conn)
    .optional();

  let token = match token_exists {
    Ok(Some(auth_token)) => auth_token.id,
    Ok(None) => String::new(),
    Err(e) => return Err(Box::new(e)),
  };

  if token.is_empty() {
    println!("🚀 Please wait generating access/refresh public/private tokens");

    // generate new token
    let generate_random_string = create_random_string_generator(&["a-z", "0-9", "A-Z", "-_"]);
   
    // Generate a 32-character string using a custom alphabet
    let auth_token = generate_random_string(32, Some("A-Z0-9"));

    // save to database
    let timestamp = Utc::now().naive_utc();
    let statement = diesel::insert_into(auth_tokens::table)
      .values(&AuthToken {
        id: Ulid::new().to_string(),
        token: auth_token.clone(),
        created_at: timestamp,
        updated_at: None,
        deleted_at: None
      })
      .execute(&mut conn);

    if let Err(e) = statement {
      let error_message = format!("Failed to save access token: validation error\nDetails: {:?}", e);
      
      return Err(Box::new(std::io::Error::new(
        std::io::ErrorKind::Other,
        error_message
      )));
    }

    // generate new access/refresh public/private tokens
    config.access_key = generate_paseto_secret_deterministic(&auth_token, "access")?;
    config.refresh_key = generate_paseto_secret_deterministic(&auth_token, "refresh")?;

    // Save the tokens to TOML file
    let mut tokens = Map::new();
    tokens.insert("access_key".into(), TomlValue::String(config.access_key.clone()));
    tokens.insert("refresh_key".into(), TomlValue::String(config.refresh_key.clone()));
    let toml_string = toml::to_string(&tokens)?;
    std::fs::write("paseto_tokens.toml", toml_string)?;
  }

  Ok(config)
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