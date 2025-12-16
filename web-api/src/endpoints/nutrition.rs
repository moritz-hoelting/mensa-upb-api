use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::dish::DishNutrients;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(nutrition);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NutritionQuery {
    date: Option<NaiveDate>,
}

#[get("/nutrition/{name}")]
async fn nutrition(
    path: web::Path<String>,
    query: web::Query<NutritionQuery>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let db = db.as_ref();
    let dish_name = path.into_inner();

    let res = if let Some(date) = query.date {
        sqlx::query_as!(
            DishNutrients,
            r#"SELECT kjoules, proteins, carbohydrates, fats FROM meals m WHERE is_latest = TRUE AND LOWER("name") = $1 AND date = $2 LIMIT 1;"#,
            dish_name.to_lowercase(),
            date,
        ).fetch_optional(db).await
    } else {
        sqlx::query_as!(
            DishNutrients,
            r#"SELECT kjoules, proteins, carbohydrates, fats FROM meals m WHERE is_latest = TRUE AND LOWER("name") = $1 ORDER BY date DESC LIMIT 1;"#,
            dish_name.to_lowercase(),
        ).fetch_optional(db).await
    };

    match res {
        Ok(Some(nutrition)) => HttpResponse::Ok().json(nutrition.normalize()),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": "Dish cannot be found",
        })),
        Err(err) => {
            tracing::error!("Failed to query database: {err:?}");
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to query database",
            }))
        }
    }
}
