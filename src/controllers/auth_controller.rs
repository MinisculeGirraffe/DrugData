use actix_web::web;
use actix_web::{Error, HttpResponse};
use entity::{session, User};
use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};
use crate::utils::is_password_valid;
use serde::{Deserialize, Serialize};

use crate::models::auth::Authenticated;
pub fn auth_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/signup").route(web::post().to(signup)))
        .service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::get().to(logout)));
}

#[derive(Serialize, Deserialize)]
struct SignupRequest {
    username: String,
    password: String,
}


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
