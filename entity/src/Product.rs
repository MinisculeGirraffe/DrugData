use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Products")]
pub struct Model {
    #[sea_orm(primary_key)]
    product_no: String,
    #[sea_orm(primary_key)]
    appl_no: String,
    form: Option<String>,
    strength: Option<String>,
    reference_drug: Option<i32>,
    drug_name: Option<String>,
    active_ingredient: Option<String>,
    reference_standard: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}




impl ActiveModelBehavior for ActiveModel {}
