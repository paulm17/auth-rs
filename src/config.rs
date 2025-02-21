fn get_env_var(var_name: &str) -> String {
  std::env::var(var_name).unwrap_or_else(|_| panic!("{} must be set", var_name))
}

#[derive(Debug, Clone)]
pub struct Config {
  pub convex_url: String,    
  pub client_origin: String,
  pub server_url: String,

  pub mailer_server: String,
  pub mailer_port: u16,
  pub mailer_from: String,
  pub mailer_from_name: String,

  pub amazon_client_id: String,
  pub amazon_client_secret: String,
  pub amazon_redirect_url: String,
  pub facebook_client_id: String,
  pub facebook_client_secret: String,
  pub facebook_redirect_url: String,
  pub github_client_id: String,
  pub github_client_secret: String,
  pub github_redirect_url: String,
  pub google_client_id: String,
  pub google_client_secret: String,
  pub google_redirect_url: String,
  pub instagram_client_id: String,
  pub instagram_client_secret: String,
  pub instagram_redirect_url: String,
  pub linkedin_client_id: String,
  pub linkedin_client_secret: String,
  pub linkedin_redirect_url: String,
  pub microsoft_client_id: String,
  pub microsoft_client_secret: String,
  pub microsoft_redirect_url: String,
  pub reddit_client_id: String,
  pub reddit_client_secret: String,
  pub reddit_redirect_url: String,
  pub tiktok_client_id: String,
  pub tiktok_client_secret: String,
  pub tiktok_redirect_url: String,
  pub twitch_client_id: String,
  pub twitch_client_secret: String,
  pub twitch_redirect_url: String,
  pub twitter_client_id: String,
  pub twitter_client_secret: String,
  pub twitter_redirect_url: String,

  pub auth_token: String,
  pub access_token_expires_in: String,
  pub access_token_max_age: i64,
  pub refresh_token_expires_in: String,
  pub refresh_token_max_age: i64,
}

impl Config {
  pub fn init() -> Config {
    let convex_url = get_env_var("AUTH_CONVEX_URL");        
    let client_origin = get_env_var("AUTH_CLIENT_ORIGIN");        
    let server_url = get_env_var("AUTH_SERVER_URL");

    let auth_token = get_env_var("AUTH_TOKEN");
    let access_token_expires_in = get_env_var("AUTH_ACCESS_TOKEN_EXPIRED_IN");
    let access_token_max_age = get_env_var("AUTH_ACCESS_TOKEN_MAXAGE");
    let refresh_token_expires_in = get_env_var("AUTH_REFRESH_TOKEN_EXPIRED_IN");
    let refresh_token_max_age = get_env_var("AUTH_REFRESH_TOKEN_MAXAGE");

    let mailer_server = get_env_var("SMTP_SERVER_URL");
    let mailer_port = get_env_var("SMTP_PORT").parse::<u16>().unwrap();
    let mailer_from = get_env_var("SMTP_FROM");
    let mailer_from_name = get_env_var("SMTP_FROM_NAME");

    let amazon_client_id = get_env_var("AUTH_AMAZON_CLIENT_ID");
    let amazon_client_secret = get_env_var("AUTH_AMAZON_CLIENT_SECRET");
    let amazon_redirect_url = get_env_var("AUTH_AMAZON_REDIRECT_URI");
    let facebook_client_id = get_env_var("AUTH_FACEBOOK_CLIENT_ID");
    let facebook_client_secret = get_env_var("AUTH_FACEBOOK_CLIENT_SECRET");
    let facebook_redirect_url = get_env_var("AUTH_FACEBOOK_REDIRECT_URI");
    let github_client_id = get_env_var("AUTH_GITHUB_CLIENT_ID");
    let github_client_secret = get_env_var("AUTH_GITHUB_CLIENT_SECRET");
    let github_redirect_url = get_env_var("AUTH_GITHUB_REDIRECT_URI");
    let google_client_id = get_env_var("AUTH_GOOGLE_CLIENT_ID");
    let google_client_secret = get_env_var("AUTH_GOOGLE_CLIENT_SECRET");
    let google_redirect_url = get_env_var("AUTH_GOOGLE_REDIRECT_URI");
    let instagram_client_id = get_env_var("AUTH_INSTAGRAM_CLIENT_ID");
    let instagram_client_secret = get_env_var("AUTH_INSTAGRAM_CLIENT_SECRET");
    let instagram_redirect_url = get_env_var("AUTH_INSTAGRAM_REDIRECT_URI");
    let linkedin_client_id = get_env_var("AUTH_LINKEDIN_CLIENT_ID");
    let linkedin_client_secret = get_env_var("AUTH_LINKEDIN_CLIENT_SECRET");
    let linkedin_redirect_url = get_env_var("AUTH_LINKEDIN_REDIRECT_URI");
    let microsoft_client_id = get_env_var("AUTH_MICROSOFT_CLIENT_ID");
    let microsoft_client_secret = get_env_var("AUTH_MICROSOFT_CLIENT_SECRET");
    let microsoft_redirect_url = get_env_var("AUTH_MICROSOFT_REDIRECT_URI");
    let reddit_client_id = get_env_var("AUTH_REDDIT_CLIENT_ID");
    let reddit_client_secret = get_env_var("AUTH_REDDIT_CLIENT_SECRET");
    let reddit_redirect_url = get_env_var("AUTH_REDDIT_REDIRECT_URI");
    let tiktok_client_id = get_env_var("AUTH_TIKTOK_CLIENT_ID");
    let tiktok_client_secret = get_env_var("AUTH_TIKTOK_CLIENT_SECRET");
    let tiktok_redirect_url = get_env_var("AUTH_TIKTOK_REDIRECT_URI");
    let twitch_client_id = get_env_var("AUTH_TWITCH_CLIENT_ID");
    let twitch_client_secret = get_env_var("AUTH_TWITCH_CLIENT_SECRET");
    let twitch_redirect_url = get_env_var("AUTH_TWITCH_REDIRECT_URI");
    let twitter_client_id = get_env_var("AUTH_TWITTER_CLIENT_ID");
    let twitter_client_secret = get_env_var("AUTH_TWITTER_CLIENT_SECRET");
    let twitter_redirect_url = get_env_var("AUTH_TWITTER_REDIRECT_URI");

    Config {
      client_origin,
      convex_url,
      server_url,
      mailer_server,
      mailer_port,
      mailer_from,
      mailer_from_name,  
      amazon_client_id,
      amazon_client_secret,
      amazon_redirect_url,    
      facebook_client_id,
      facebook_client_secret,
      facebook_redirect_url,
      github_client_id,
      github_client_secret,
      github_redirect_url,
      google_client_id,
      google_client_secret,
      google_redirect_url,
      instagram_client_id,
      instagram_client_secret,
      instagram_redirect_url,
      linkedin_client_id,
      linkedin_client_secret,
      linkedin_redirect_url,
      microsoft_client_id,
      microsoft_client_secret,
      microsoft_redirect_url,
      reddit_client_id,
      reddit_client_secret,
      reddit_redirect_url,
      tiktok_client_id,
      tiktok_client_secret,
      tiktok_redirect_url,
      twitch_client_id,
      twitch_client_secret,
      twitch_redirect_url,
      twitter_client_id,
      twitter_client_secret,
      twitter_redirect_url,
      auth_token,
      access_token_expires_in,
      access_token_max_age: access_token_max_age.parse::<i64>().unwrap(),
      refresh_token_expires_in,
      refresh_token_max_age: refresh_token_max_age.parse::<i64>().unwrap(),
    }
  }
}
