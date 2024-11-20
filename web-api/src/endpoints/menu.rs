use std::str::FromStr as _;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDate;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::{Canteen, Menu};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct MenuQuery {
    date: Option<NaiveDate>,
}
#[get("/menu/{canteen}")]
async fn menu(
    path: web::Path<String>,
    query: web::Query<MenuQuery>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let canteens = path
        .into_inner()
        .split(',')
        .map(Canteen::from_str)
        .collect_vec();
    if canteens.iter().all(Result::is_ok) {
        let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();

        let date = query
            .date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let menu = Menu::query(&db, date, &canteens).await;

        if let Ok(menu) = menu {
            HttpResponse::Ok().json(menu)
        } else {
            HttpResponse::InternalServerError().json(json!({
                "error": "Failed to query database",
            }))
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid canteen identifier",
            "invalid": canteens.into_iter().filter_map(|c| c.err()).collect_vec()
        }))
    }
}
