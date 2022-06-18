use actix_web::middleware::Logger;
use actix_web::{get, web, App, Error, HttpResponse, HttpServer};
use entity::Product;
use env_logger::Env;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
mod init;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db = init::setup().await.unwrap();

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/drug/{name}")]
async fn index(
    db: web::Data<DatabaseConnection>,
    name: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let conn = db.as_ref();
    let results = Product::Entity::find()
        .filter(Product::Column::DrugName.contains(name.as_str()))
        .all(conn)
        .await
        .unwrap();

    Ok(HttpResponse::Ok().json(results))
}
