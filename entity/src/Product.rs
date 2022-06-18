use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Products")]
pub struct Model {
    #[sea_orm(primary_key)]
    ProductNo: String,
    #[sea_orm(primary_key)]
    ApplNo: String,
    Form: Option<String>,
    Strength: Option<String>,
    ReferenceDrug: Option<i32>,
    DrugName: Option<String>,
    ActiveIngredient: Option<String>,
    ReferenceStandard: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::Applications::Entity",
        from = "Column::ApplNo",
        to = "super::Applications::Column::ApplNo"
    )]
    Application,

}

impl Related<super::Applications::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Application.def()
    }
}



impl ActiveModelBehavior for ActiveModel {}
