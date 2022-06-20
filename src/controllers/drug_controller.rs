use crate::models::auth::Authenticated;
use actix_web::{web, Error, HttpResponse};
use entity::product;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
pub fn drug_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/{name}").route(web::get().to(get_drug)));
}

async fn get_drug(
    _user: Authenticated,
    db: web::Data<DatabaseConnection>,
    name: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = db.as_ref();
    let results = product::Entity::find()
        .filter(product::Column::DrugName.contains(name.as_str()))
        .all(conn)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(results))
}
