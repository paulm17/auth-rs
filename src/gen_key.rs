use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rand::RngCore;
use hex;

/// Generates a valid Ed25519 private key (64 bytes) as a hex string.
/// The returned key consists of the 32-byte secret key concatenated with the 32-byte public key.
fn generate_valid_paseto_private_key() -> String {
  // Generate random bytes for the private key
  let mut rng = OsRng;
  let mut private_key_bytes = [0u8; 32];
  rng.try_fill_bytes(&mut private_key_bytes).unwrap();  // Changed to try_fill_bytes
  
  // Create signing key from random bytes
  let signing_key = SigningKey::from_bytes(&private_key_bytes);
  
  // Get the verifying key (public key)
  let verifying_key = signing_key.verifying_key();
  
  // Extract the secret and public key bytes
  let secret_bytes = signing_key.to_bytes();
  let public_bytes = verifying_key.to_bytes();

  // Concatenate secret and public bytes to form a 64-byte array
  let mut key_bytes = [0u8; 64];
  key_bytes[..32].copy_from_slice(&secret_bytes);
  key_bytes[32..].copy_from_slice(&public_bytes);

  // Convert the key bytes to a hex string
  hex::encode(key_bytes)
}

fn main() {
  // Generate a valid PASETO private key.
  let private_key_hex = generate_valid_paseto_private_key();
  println!("Generated Private Key (hex): {}", private_key_hex);
}