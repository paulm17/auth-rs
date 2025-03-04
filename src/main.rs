mod config;
mod social_handlers;
mod handlers;
mod jwt_auth;
mod model;
mod response;
mod route;
mod token;
mod schema;
mod smtp;
mod template;
mod utils;

use config::Config;
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use std::sync::Arc;
use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, 
      ORIGIN, USER_AGENT, ACCESS_CONTROL_REQUEST_HEADERS,
      ACCESS_CONTROL_REQUEST_METHOD}, HeaderValue, Method
};
use dotenv::dotenv;
use route::create_router;
use tower_http::cors::CorsLayer;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use axum_server::tls_rustls::RustlsConfig;
use tracing::{Level, subscriber::set_global_default};
use tracing_subscriber::FmtSubscriber;

type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct AppState {
  db_pool: DbPool,
  env: Config,
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
  // let cors = CorsLayer::new()
  //   .allow_origin(config.clone().client_origin.parse::<HeaderValue>().unwrap())
  //   .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
  //   .allow_credentials(true)
  //   .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);  

  let cors = CorsLayer::new()
    .allow_origin(config.clone().client_origin.parse::<HeaderValue>().unwrap()) // Explicit frontend origin
    .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS]) // Add OPTIONS
    .allow_credentials(true)
    .allow_headers([
      AUTHORIZATION, 
      ACCEPT, 
      CONTENT_TYPE,
      ORIGIN,
      USER_AGENT,
      ACCESS_CONTROL_REQUEST_HEADERS,
      ACCESS_CONTROL_REQUEST_METHOD,
    ]);

  let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
  let pool = r2d2::Pool::builder()
    .build(manager)
    .expect("Failed to create pool.");

  let app = create_router(Arc::new(AppState {
    db_pool: pool,
    env: config.clone(),
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

      axum_server::bind_rustls("0.0.0.0:9179".parse().unwrap(), config)
        .serve(https_app.into_make_service())
        .await
        .unwrap();
    },
    // HTTP Server
    async {
      let listener = tokio::net::TcpListener::bind("0.0.0.0:9178").await.unwrap();
      axum::serve(listener, app).await.unwrap();
    }
  );
}