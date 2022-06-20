use actix_web::web;
use entity::session::{Model, KEY};
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use sea_orm::DatabaseConnection;
use sea_orm::DbErr;

pub fn decode_token(token: String) -> Result<TokenData<Model>, jsonwebtoken::errors::Error> {
    jsonwebtoken::decode::<Model>(
        &token,
        &DecodingKey::from_secret(&KEY),
        &Validation::default(),
    )
}

pub async fn verify_token(
    token_data: &TokenData<Model>,
    db: &web::Data<DatabaseConnection>,
) -> Result<Model, DbErr> {
    entity::user::Model::validate_login_session(&token_data.claims, &db.get_ref()).await
}
