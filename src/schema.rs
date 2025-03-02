use diesel::prelude::*;
use chrono::NaiveDateTime;

#[derive(Queryable, Insertable)]
#[diesel(table_name = auth_tokens)]
pub struct AuthToken {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub token: String,  
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  auth_tokens (id) {
    id -> Text,
    token -> Text,    
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(Queryable, Insertable, Selectable)]
#[diesel(table_name = email_confirmation)]
pub struct EmailConfirmation {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub user_id: String,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub code: String,  
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub expires: NaiveDateTime,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub flow: String,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub redirect_to: Option<String>,  
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  email_confirmation (id) {
    id -> Text,
    user_id -> Text,
    code -> Text,
    expires -> Timestamp,
    flow -> Text,
    redirect_to -> Nullable<Text>,    
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(AsChangeset)]
#[diesel(table_name = email_confirmation)]
pub struct EmailConfirmationFlowUpdate {
  pub flow: String,
  pub updated_at: Option<NaiveDateTime>,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = identities)]
pub struct Identity {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub user_id: String,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub provider_id: String,  
  #[diesel(sql_type = diesel::sql_types::Json)]
  pub identity_data: serde_json::Value, 
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub last_signin_at: NaiveDateTime,
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  identities (id) {
    id -> Text,
    user_id -> Text,    
    provider_id -> Text,    
    identity_data -> Json,    
    last_signin_at -> Timestamp,    
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = social_auth)]
pub struct SocialAuth {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub provider_id: String,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub csrf: String,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub pkce_verifier: String,  
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub expires: NaiveDateTime,  
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub redirect_to: String,  
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  social_auth (id) {
    id -> Text,
    provider_id -> Text,
    csrf -> Text,
    pkce_verifier -> Text,
    expires -> Timestamp,
    redirect_to -> Text,  
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = social_provider)]
pub struct SocialProvider {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub name: String,  
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  social_provider (id) {
    id -> Text,
    name -> Text,    
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = tokens)]
pub struct Token {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub user_id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub token: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub token_uuid: String,
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub expires: NaiveDateTime,
  #[diesel(sql_type = diesel::sql_types::Bool)]
  pub blacklisted: bool,
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  tokens (id) {
    id -> Text,
    user_id -> Text,
    token -> Text,
    token_uuid -> Text,
    expires -> Timestamp,
    blacklisted -> Bool,
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = user)]
pub struct User {
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub id: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub name: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub email: String,
  #[diesel(sql_type = diesel::sql_types::Text)]
  pub password: Option<String>,
  #[diesel(sql_type = diesel::sql_types::Bool)]
  pub verified: bool,
  #[diesel(column_name = "created_at")]
  #[diesel(sql_type = diesel::sql_types::Timestamp)]
  pub created_at: NaiveDateTime,
  #[diesel(column_name = "updated_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub updated_at: Option<NaiveDateTime>,
  #[diesel(column_name = "deleted_at")]
  #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamp>)]
  pub deleted_at: Option<NaiveDateTime>,
}

table! {
  user (id) {
    id -> Text,
    name -> Text,
    email -> Text,
    password -> Nullable<Text>,
    verified -> Bool,
    #[sql_name = "created_at"]
    created_at -> Timestamp,
    #[sql_name = "updated_at"]
    updated_at -> Nullable<Timestamp>,
    #[sql_name = "deleted_at"]
    deleted_at -> Nullable<Timestamp>,
  }
}

#[derive(AsChangeset)]
#[diesel(table_name = user)]
pub struct UserPasswordUpdate {
  pub password: Option<String>,
  pub updated_at: Option<NaiveDateTime>,
}

allow_tables_to_appear_in_same_query!(email_confirmation, user);