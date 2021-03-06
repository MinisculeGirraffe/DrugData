use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{Utc,DateTime};
use sea_orm::Set;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "accounting")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    id: Uuid,
    schedule_id: Uuid,
    amount: i32,
    timestamp:DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::schedule::Entity",
        from = "Column::ScheduleId",
        to = "super::schedule::Column::Id"
    )]
    Schedule,
}

impl Related<super::schedule::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Schedule.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self{
            id: Set(Uuid::new_v4()),
            timestamp: Set(Utc::now()),
            ..ActiveModelTrait::default()
        }

    }
}
