use actix_web::web;
use entity::session::Model;
use jsonwebtoken::{DecodingKey, TokenData, Validation};

use sea_orm::DatabaseConnection;
use sea_orm::DbErr;

pub fn decode_token(token: String) -> Result<TokenData<Model>, jsonwebtoken::errors::Error> {
    let string = std::env::var("SECRET_KEY").unwrap();
    let key = string.as_bytes();
    println!("{:?}", key.len());
    jsonwebtoken::decode::<Model>(
        &token,
        &DecodingKey::from_secret(key),
        &Validation::default(),
    )
}

pub async fn verify_token(
    token_data: &TokenData<Model>,
    db: &web::Data<DatabaseConnection>,
) -> Result<Model, DbErr> {
    entity::user::Model::validate_login_session(&token_data.claims, &db.get_ref()).await
}
