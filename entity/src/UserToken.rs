use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::marker::Copy;
use crate::{User, session};
use sea_orm::prelude::Uuid;

pub struct LoginInfoDTO {
    pub username: String,
    pub login_session: String,
}
pub static KEY: [u8; 16] = *include_bytes!("../../secret.key");
static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Serialize, Deserialize,Clone)]
pub struct UserToken {
    // issued at
    pub iat: i64,
    // expiration
    pub exp: i64,
    // data
    pub user: Uuid,
    pub session: Uuid

}

impl UserToken {
    pub fn generate(model: session::Model) -> String {
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
        let payload = UserToken {
            iat: now,
            exp: now + ONE_WEEK,
            user: model.user_id,
            session: model.id
        };
        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&KEY),
        ).unwrap()
    }
}
