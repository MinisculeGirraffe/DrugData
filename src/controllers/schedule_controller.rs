use crate::models::auth::Authenticated;
use crate::utils::validate_cron_expression;
use actix_web::{error, web, Error, HttpResponse};
use entity::{accounting_entry, schedule};
use futures::{try_join, TryFutureExt};
use sea_orm::{
    ActiveModelBehavior, ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};
pub fn schedule_service(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::get().to(get_schedules))
            .route(web::post().to(add_schedule)),
    )
    .service(
        web::resource("/{id}")
            .route(web::get().to(get_schedule_by_id))
            .route(web::put().to(update_schedule))
            .route(web::delete().to(delete_schedule)),
    );
}

async fn get_schedules(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
) -> Result<HttpResponse, Error> {
    let query = schedule::Entity::find()
        .filter(schedule::Column::UserId.eq(user.user_id))
        .all(db.get_ref())
        .await;

    match query {
        Ok(schedules) => Ok(HttpResponse::Ok().json(schedules)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}

async fn get_schedule_from_db(
    db: &web::Data<DatabaseConnection>,
    id: web::Path<sea_orm::prelude::Uuid>,
    userId: sea_orm::prelude::Uuid,
) -> Result<schedule::Model, Error> {
    let query = schedule::Entity::find()
        .filter(schedule::Column::Id.eq(id.clone()))
        .filter(schedule::Column::UserId.eq(userId))
        .one(db.get_ref())
        .await;

    match query {
        Ok(option) => match option {
            Some(schedule) => Ok(schedule),
            None => Err(error::ErrorNotFound("")),
        },
        Err(_) => Err(error::ErrorInternalServerError("")),
    }
}

async fn log_accounting_entry(
    db: &web::Data<DatabaseConnection>,
    old: &i32,
    new: &i32,
    sched_id: &sea_orm::prelude::Uuid,
) -> Result<(), Error> {
    let mut entry = accounting_entry::ActiveModel::new();
    entry.amount = Set(new - old);
    entry.schedule_id = Set(sched_id.clone());

    match entry.insert(db.get_ref()).await {
        Ok(_) => Ok(()),
        Err(_) => Err(error::ErrorInternalServerError("")),
    }
}
#[derive(Serialize, Deserialize)]
struct ScheduleDetailResponse {
    schedule: schedule::Model,
    history: Vec<accounting_entry::Model>,
}

async fn get_schedule_by_id(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
    id: web::Path<sea_orm::prelude::Uuid>,
) -> Result<HttpResponse, Error> {
    let history = accounting_entry::Entity::find()
        .filter(accounting_entry::Column::ScheduleId.eq(id.clone()))
        .order_by_asc(accounting_entry::Column::Timestamp)
        .all(db.get_ref())
        .map_err(|_| error::ErrorInternalServerError(""));
    let model = get_schedule_from_db(&db, id, user.user_id);

    let result = try_join!(model, history)?;
    Ok(HttpResponse::Ok().json(ScheduleDetailResponse {
        schedule: result.0,
        history: result.1,
    }))
}

#[derive(Serialize, Deserialize)]
struct ScheduleRequest {
    cron: String,
    drug_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pill_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pill_amount: Option<i32>,
}
async fn add_schedule(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
    body: web::Json<ScheduleRequest>,
) -> Result<HttpResponse, Error> {
    let pill_count = body.pill_count.unwrap_or(0);
    let pill_amount = body.pill_amount.unwrap_or(0);

    if !validate_cron_expression(body.cron.clone()) {
        return Ok(HttpResponse::BadRequest().body("Invalid Cron expression"));
    }

    let mut schedule = schedule::ActiveModel::new();
    schedule.user_id = Set(user.user_id);
    schedule.drug_name = Set(body.drug_name.clone());
    schedule.cron = Set(body.cron.clone());
    schedule.pill_count = Set(pill_count);
    schedule.pill_amount = Set(pill_amount);
    let query = schedule.insert(db.get_ref()).await;
    match query {
        Ok(result) => {
            log_accounting_entry(&db, &0, &pill_count, &result.id).await?;
            Ok(HttpResponse::Ok().json(result))
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
struct UpdateScheduleReq {
    #[serde(skip_serializing_if = "Option::is_none")]
    cron: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pill_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pill_amount: Option<i32>,
}
async fn update_schedule(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
    id: web::Path<sea_orm::prelude::Uuid>,
    body: web::Json<UpdateScheduleReq>,
) -> Result<HttpResponse, Error> {
    let model = get_schedule_from_db(&db, id, user.user_id).await?;
    let mut active_model: schedule::ActiveModel = model.into();

    if let Some(cron) = body.cron.to_owned() {
        active_model.cron = Set(cron);
    }
    if let Some(pill_amount) = body.pill_amount.to_owned() {
        active_model.pill_amount = Set(pill_amount);
    }
    if let Some(pill_count) = body.pill_count {
        log_accounting_entry(
            &db,
            active_model.pill_count.as_ref(),
            &pill_count,
            active_model.id.as_ref(),
        )
        .await?;

        active_model.pill_count = Set(pill_count);
    }

    match active_model.update(db.get_ref()).await {
        Ok(result) => Ok(HttpResponse::Ok().json(result)),
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}

async fn delete_schedule(
    user: Authenticated,
    db: web::Data<DatabaseConnection>,
    id: web::Path<sea_orm::prelude::Uuid>,
) -> Result<HttpResponse, Error> {
    let model = get_schedule_from_db(&db, id, user.user_id).await?;
    let active_model: schedule::ActiveModel = model.into();
    match active_model.delete(db.get_ref()).await {
        Ok(_) => Ok(HttpResponse::Ok().body("")),
        Err(_) => Ok(HttpResponse::InternalServerError().body("")),
    }
}
