use std::str::FromStr;

use chrono::Utc;
use cron::Schedule;
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "schedule")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    user_id: Uuid,
    drug_name: String,
    pill_count: i32,
    pill_amount: i32,
    cron: String,
    added_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::accounting_entry::Entity")]
    AccountingEntry,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::accounting_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountingEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            added_at: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }
    }
    fn before_save(self, _insert: bool) -> Result<Self, DbErr> {
        if !self.cron.is_unchanged() {
            match Schedule::from_str(self.cron.as_ref()) {
                Ok(_) => return Ok(self),
                Err(_) => return Err(DbErr::Type(
                    "Cron value is not a valid expression".to_string(),
                )),
            }
        };
        Ok(self)
    }
}
