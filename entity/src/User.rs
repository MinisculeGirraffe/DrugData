use argon2;
use chrono::{DateTime, Utc};
use rand::Rng;
use sea_orm::{entity::prelude::*, ActiveValue::NotSet, Set, Unset};
use serde::{Deserialize, Serialize};

use crate::UserToken::UserToken;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: Uuid,
    username: String,
    #[serde(skip_serializing, skip_deserializing)]
    password: String,
    created: ChronoDateTimeUtc,
    updated: ChronoDateTimeUtc,
    login_session: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Model {
    pub fn verify_password(&self, password: String) -> Result<bool, argon2::Error> {
        argon2::verify_encoded(&self.password, password.as_bytes())
    }

    pub async fn validate_login_session(
        token: &UserToken,
        db: &DatabaseConnection,
    ) -> Result<Model, DbErr> {
        println!("{:?}", token.model);
        let user = Entity::find()
            .filter(Column::Username.eq(token.model.username.clone()))
            .filter(Column::LoginSession.eq(token.model.login_session.clone()))
            .one(db)
            .await?;
        match user {
            Some(v) => Ok(v),
            None => Err(DbErr::RecordNotFound("missing".to_string())),
        }
    }

    pub async fn save_login_session(
        &self,
        db: &DatabaseConnection,
        session: String,
    ) -> Result<(), DbErr> {
        let mut user: ActiveModel = Entity::find()
            .filter(Column::Username.eq(self.username.clone()))
            .one(db)
            .await?
            .unwrap()
            .into();

        user.login_session = Set(Some(session));
        user.save(db).await?;
        Ok(())
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            created: Set(Utc::now()),
            updated: Set(Utc::now()),
            password: NotSet,
            ..ActiveModelTrait::default()
        }
    }
    fn before_save(mut self, insert: bool) -> Result<Self, DbErr> {
        println!("Before Save");
        let timestamp = Utc::now();
        if self.id.is_not_set() {
            self.id = Set(Uuid::new_v4());
        }
        if !self.password.is_unchanged() {
            let salt = rand::thread_rng().gen::<[u8; 32]>();
            let config = argon2::Config {
                variant: argon2::Variant::Argon2id,
                version: argon2::Version::Version13,
                mem_cost: 65536,
                time_cost: 10,
                lanes: 4,
                thread_mode: argon2::ThreadMode::Parallel,
                secret: &[],
                ad: &[],
                hash_length: 32,
            };
            let pw = self.password.as_ref().as_bytes();
            let hash = argon2::hash_encoded(&pw, &salt, &config).unwrap();

            self.password = Set(hash);
        }
        if self.created.is_not_set() {
            self.created = Set(timestamp)
        }

        self.updated = Set(timestamp);
        Ok(self)
    }
}
