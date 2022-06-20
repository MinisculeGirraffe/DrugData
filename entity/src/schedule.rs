use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "schedule")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: Uuid,
    user_id: Uuid,
    drug_name: String,
    pill_count: i32,
    cron: String,
    added_at: ChronoDateTimeUtc
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::User::Entity",
        from = "Column::UserId",
        to = "super::User::Column::Id"
    )]
    User,
    #[sea_orm(has_many = "super::accounting_entry::Entity")]
    AccountingEntry,
}

impl Related<super::User::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::accounting_entry::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AccountingEntry.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
