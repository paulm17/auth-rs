use std::sync::Arc;

use axum::{
  middleware,
  routing::{get, post},
  Router,
};

use crate::{
  handlers::{check_code_handler::check_code_handler, forgot_password_handler::forgot_password_handler, generate_magiclink_handler::generate_magiclink_handler, get_me_handler::get_me_handler, login_user_handler::login_user_handler, logout_handler::logout_handler, refresh_access_token_handler::refresh_access_token_handler, register_user_handler::register_user_handler, reset_password_handler::reset_password_handler, verify_code_handler::verify_code_handler, verify_magiclink_code_handler::verify_magiclink_code_handler}, jwt_auth::auth, social_handlers::{callback_handler::callback_handler, url_handler::url_handler}, AppState
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
  Router::new()
    // user/password
    .route("/register", post(register_user_handler))
    .route("/login", post(login_user_handler))      
    .route("/refresh", get(refresh_access_token_handler))        
    .route("/forgot_password", post(forgot_password_handler))        
    .route("/verify_code", get(verify_code_handler))        
    .route("/check_code", post(check_code_handler))     
    .route("/reset_password", post(reset_password_handler))        
    .route("/generate_magiclink", post(generate_magiclink_handler))        
    .route("/verify_magiclink_code", get(verify_magiclink_code_handler))        
    //oauth
    .route("/oauth/url", post(url_handler))
    .route("/oauth/callback", get(callback_handler))
    // needs middleware
    .route(
      "/logout",
      get(logout_handler)
      .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
    )
    .route(
      "/users/me",
      get(get_me_handler)
      .route_layer(middleware::from_fn_with_state(app_state.clone(), auth)),
    )
    .with_state(app_state)
}
