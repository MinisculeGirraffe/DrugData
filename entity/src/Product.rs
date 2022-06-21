use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Deserialize, Serialize)]
#[sea_orm(table_name = "Products")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(alias = "ProductNo")]
    product_no: String,
    #[sea_orm(primary_key)]
    #[serde(alias = "ApplNo")]
    appl_no: String,
    #[serde(alias = "Form")]
    form: Option<String>,
    #[serde(alias = "Strength")]
    strength: Option<String>,
    #[serde(alias = "ReferenceDrug")]
    reference_drug: Option<i32>,
    #[serde(alias = "DrugName")]
    drug_name: Option<String>,
    #[serde(alias = "ActiveIngredient")]
    active_ingredient: Option<String>,
    #[serde(alias = "ReferenceStandard")]
    reference_standard: Option<i32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}




impl ActiveModelBehavior for ActiveModel {}
