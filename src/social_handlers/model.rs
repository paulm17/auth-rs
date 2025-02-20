use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq)]
pub enum OAuthProvider {
  Amazon,
  Facebook,
  Github,
  Google,
  Instagram,
  LinkedIn,
  Microsoft,
  Reddit,
  Tiktok,
  Twitch,
  Twitter
}