use actix_web::web;
use auth_controller::auth_service;
use drug_controller::drug_service;
use log::info;
use schedule_controller::schedule_service;
use user_controller::user_service;

pub mod auth_controller;
pub mod drug_controller;
pub mod schedule_controller;
pub mod user_controller;

pub fn config_app(cfg: &mut web::ServiceConfig) {
    info!("Configuring routes");
    cfg.service(
        web::scope("/api")
            .service(web::scope("/drug").configure(drug_service))
            .service(web::scope("/user").configure(user_service))
            .service(web::scope("/schedule").configure(schedule_service)),
    )
    .service(web::scope("/auth").configure(auth_service));
}
