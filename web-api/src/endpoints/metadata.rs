use std::sync::OnceLock;

use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(web::scope("/metadata").service(earliest_meal_date));
}

static EARLIEST_MEAL_DATE: OnceLock<NaiveDate> = OnceLock::new();

#[get("/earliest-meal-date")]
async fn earliest_meal_date(db: web::Data<PgPool>) -> impl Responder {
    if let Some(earliest_date) = EARLIEST_MEAL_DATE.get() {
        earliest_meal_date_ok_response(*earliest_date)
    } else {
        match sqlx::query_scalar!(
            r#"SELECT MIN(date) AS "date!" FROM meals WHERE is_latest = TRUE;"#
        )
        .fetch_one(db.as_ref())
        .await
        {
            Ok(date) => {
                EARLIEST_MEAL_DATE.set(date).ok();
                earliest_meal_date_ok_response(date)
            }
            Err(err) => {
                tracing::error!("Failed to query datebase: {err}");
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to query database"
                }))
            }
        }
    }
}

fn earliest_meal_date_ok_response(date: NaiveDate) -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "date": date,
    }))
}
