use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Application")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    ApplNo: String,
    ApplType: String,
    ApplPublicNotes: Option<String>,
    SponsorName: Option<String> 
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::Product::Entity")]
    Products
}


impl Related<super::Product::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Products.def()
    }
}




impl ActiveModelBehavior for ActiveModel {}

