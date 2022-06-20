use crate::models::auth::Authenticated;
use actix_web::{web, Error, HttpResponse};
use entity::{session, user};
use sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

pub fn user_service(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(get_user)));
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    user: user::Model,
    sessions: Vec<session::Model>,
}
impl UserResponse {
    fn new(query: Vec<(user::Model, Vec<session::Model>)>) -> UserResponse {
        let result = query.get(0).unwrap();
        UserResponse {
            user: result.0.clone(),
            sessions: result.1.clone(),
        }
    }
}

async fn get_user(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let query = user::Entity::find_by_id(user.user_id)
        .find_with_related(session::Entity)
        .all(db.as_ref())
        .await;
    // if there was a database error, return an internal server error
    if query.is_err() {
        return Ok(HttpResponse::InternalServerError().body(""));
    }
    let result = query.unwrap();

    Ok(HttpResponse::Ok().json(UserResponse::new(result)))
}
