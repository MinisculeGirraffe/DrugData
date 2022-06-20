use actix_web::web;
use entity::{UserToken::{UserToken, KEY}, User, session};
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;

pub fn decode_token(token: String) -> Result<TokenData<UserToken>,jsonwebtoken::errors::Error> {
       jsonwebtoken::decode::<UserToken>(
        &token,
        &DecodingKey::from_secret(&KEY),
        &Validation::default(),
    )
}

pub async fn verify_token(
    token_data: &TokenData<UserToken>,
    db: &web::Data<DatabaseConnection>,
) -> Result<session::Model, DbErr> {
    
    entity::User::Model::validate_login_session(&token_data.claims, &db.get_ref()).await
   
  
}
