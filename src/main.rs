mod config;
mod social_handlers;
mod handlers;
mod jwt_auth;
mod model;
mod response;
mod route;
mod token;
mod rsa;
mod smtp;
mod template;
mod utils;

use convex::ConvexClient;
use config::Config;
use rsa::RsaConfig;
use std::sync::Arc;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}, HeaderValue, Method
};
use dotenv::dotenv;
use route::create_router;
use tower_http::cors::CorsLayer;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use axum_server::tls_rustls::RustlsConfig;
use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::FmtSubscriber;

pub struct AppState {
  convex: ConvexClient,
  env: Config,
  rsa: RsaConfig
}

#[tokio::main]
async fn main() {
  dotenv().ok();

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .finish();
  
  set_global_default(subscriber)
    .expect("Failed to set up logger");

  // Install default crypto provider for rustls
  rustls::crypto::ring::default_provider()
  .install_default()
  .expect("failed to install default crypto provider");

  let config = Config::init();
  let client = ConvexClient::new(&config.convex_url).await.unwrap();
  let cors = CorsLayer::new()
    .allow_origin(config.clone().client_origin.parse::<HeaderValue>().unwrap())
    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
    .allow_credentials(true)
    .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);  

  let rsa_tokens = rsa::init(client.clone()).await;

  let app = create_router(Arc::new(AppState {
    convex: client.clone(),
    env: config.clone(),
    rsa: rsa_tokens.unwrap()
  }))
  .layer(cors);

  // Create the app router for HTTPS
  let https_app = app.clone();

  println!("🚀 Server started successfully");

  // Run both HTTP and HTTPS servers concurrently
  tokio::join!(
    // HTTPS Server
    async {
      let subject_alt_names = vec!["localhost".to_string()];
      let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();

      let config = RustlsConfig::from_pem(
        cert.pem().as_bytes().to_vec(),
        key_pair.serialize_pem().as_bytes().to_vec(),
      )
        .await
        .unwrap();

      axum_server::bind_rustls("0.0.0.0:8443".parse().unwrap(), config)
        .serve(https_app.into_make_service())
        .await
        .unwrap();
    },
    // HTTP Server
    async {
      let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
      axum::serve(listener, app).await.unwrap();
    }
  );
}