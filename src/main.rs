use actix_web::middleware::Logger;
use actix_web::{ web, App, HttpServer};
use dotenv::dotenv;

use log::info;
use migration::{Migrator, MigratorTrait};
use std::env;

use crate::controllers::config_app;
mod controllers;
mod constants;
mod middleware;
mod models;
mod utils;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    info!("Application Started");
    dotenv().ok();
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    info!("Setting up database connection");
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db = sea_orm::Database::connect(&db_url).await.unwrap();

    Migrator::up(&db, None).await.unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(middleware::auth::AuthenticateMiddlewareFactory {})
            .app_data(web::Data::new(db.clone()))
            .configure(config_app)
    })
    .bind(("::", 8080))?
    .run()
    .await
}



