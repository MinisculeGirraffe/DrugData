

use chrono::Utc;
use jsonwebtoken::{EncodingKey, Header};
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};
use std::fmt;


static ONE_WEEK: i64 = 60 * 60 * 24 * 7; // in seconds

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Session")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub iat: i64,
    // expiration
    pub exp: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
}

impl fmt::Display for Model {
    fn fmt (&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let string = std::env::var("SECRET_KEY").unwrap();
        let key = string.as_bytes();
      let token =  jsonwebtoken::encode(
            &Header::default(),
            &self,
            &EncodingKey::from_secret(&key),
        ).unwrap();

        fmt.write_str(&token)?;
        Ok(())
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Model {
    pub fn generate(user_id: Uuid, session_id: Uuid) -> String {
        let string = std::env::var("SECRET_KEY").unwrap();
        let key = string.as_bytes();
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
        let payload = Model {
            iat: now,
            exp: now + ONE_WEEK,
            user_id: user_id,
            session_id: session_id,
        };
        jsonwebtoken::encode(
            &Header::default(),
            &payload,
            &EncodingKey::from_secret(&key),
        )
        .unwrap()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        let now = Utc::now().timestamp_nanos() / 1_000_000_000; // nanosecond -> second
        Self {
            iat: Set(now),
            exp: Set(now + ONE_WEEK),
            session_id: Set(Uuid::new_v4()),
            ..ActiveModelTrait::default()
        }
    }
}
