use std::sync::OnceLock;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDate;
use serde::Serialize;
use serde_json::json;
use sqlx::PgPool;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::util::GenericServerError;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(utoipa_actix_web::scope("/metadata").service(earliest_meal_date));
}

static EARLIEST_MEAL_DATE: OnceLock<NaiveDate> = OnceLock::new();

#[derive(Serialize, utoipa::ToSchema)]
struct DateResponse {
    date: NaiveDate,
}

#[utoipa::path(summary = "Earliest meal date", description = "Get the date of the earliest meal saved.", responses(
    (status = OK, description = "Get the date of the earliest meal saved.", body = DateResponse), 
    (status = INTERNAL_SERVER_ERROR, description = "Server failed to answer request.", body = GenericServerError)
))]
#[get("/earliest-meal-date")]
async fn earliest_meal_date(db: web::Data<PgPool>) -> impl Responder {
    if let Some(earliest_date) = EARLIEST_MEAL_DATE.get() {
        HttpResponse::Ok().json(DateResponse {
            date: *earliest_date,
        })
    } else {
        match sqlx::query_scalar!(
            r#"SELECT MIN(date) AS "date!" FROM meals WHERE is_latest = TRUE;"#
        )
        .fetch_one(db.as_ref())
        .await
        {
            Ok(date) => {
                EARLIEST_MEAL_DATE.set(date).ok();
                HttpResponse::Ok().json(DateResponse { date })
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
