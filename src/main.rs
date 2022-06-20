use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, Error, HttpResponse, HttpServer};
use dotenv::dotenv;

use crate::models::auth::Authenticated;
use entity::{session, Product, User};
use log::info;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};
use std::env;

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
            .service(index)
            .service(signup)
            .service(login)
            .service(logout)
            .service(get_user)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[get("/drug/{name}")]
async fn index(
    user: Authenticated,
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

#[derive(Serialize, Deserialize)]
struct SignupRequest {
    username: String,
    password: String,
}

fn is_password_valid(s: &str) -> bool {
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    for c in s.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_digit(10);
    }

    !has_whitespace && has_upper && has_lower && has_digit && s.len() >= 8 && s.len() <= 128
}

#[post("/signup")]
async fn signup(
    db: web::Data<DatabaseConnection>,
    body: web::Json<SignupRequest>,
) -> Result<HttpResponse, Error> {
    if !is_password_valid(&body.password) {
        return Ok(HttpResponse::Ok().json("Password does not meet minimum requirements"));
    }
    let conn = db.as_ref();
    let model = User::ActiveModel {
        username: Set(body.username.to_owned()),
        password: Set(body.password.to_owned()),
        ..Default::default()
    };

    let result = model.save(conn).await;

    match result {
        Ok(r) => {
            let user = User::Entity::find_by_id(*r.id.as_ref())
                .one(conn)
                .await
                .unwrap();
            Ok(HttpResponse::Created().json(user))
        }
        Err(e) => Ok(HttpResponse::ExpectationFailed().json(e.to_string())),
    }
}
#[derive(Serialize, Deserialize)]
struct TokenResponse {
    token: String,
}
#[post("/login")]
async fn login(
    db: web::Data<DatabaseConnection>,
    body: web::Json<SignupRequest>,
) -> Result<HttpResponse, Error> {
    let username = body.username.to_owned();
    let db_response = User::Entity::find()
        .filter(User::Column::Username.eq(username.clone()))
        .one(db.as_ref())
        .await;

    match db_response {
        //no error returned from db
        Ok(result) => match result {
            //user returned from db
            Some(user) => {
                //if password matches
                if user.verify_password(body.password.to_owned()).unwrap() {
                    let token = user.new_login_session(db.as_ref()).await.unwrap();

                    return Ok(HttpResponse::Ok().json(TokenResponse {
                        token: token.to_string(),
                    }));
                //incorrect password
                } else {
                    return Ok(HttpResponse::Forbidden().body("Forbidden"));
                }
            }
            //no user found in db
            None => return Ok(HttpResponse::Forbidden().body("Forbidden")),
        },
        // database error
        Err(err) => return Ok(HttpResponse::InternalServerError().body(err.to_string())),
    };
}
#[get("/logout")]
async fn logout(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    match session::Entity::delete_by_id(user.session_id)
        .exec(db.as_ref())
        .await
    {
        Ok(_) => Ok(HttpResponse::Ok().body("")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}

#[get("/user/")]
async fn get_user(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    match User::Entity::find_by_id(user.user_id)
        .find_with_related(session::Entity)
        .all(db.as_ref())
        .await
    {
        Ok(usr) => Ok(HttpResponse::Ok().json(usr)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}
