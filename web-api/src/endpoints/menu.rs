use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use chrono::NaiveDate;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::{util, Menu};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(menu);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MenuQuery {
    date: Option<NaiveDate>,
    #[serde(default)]
    no_update: bool,
}

#[get("/menu/{canteen}")]
async fn menu(
    path: web::Path<String>,
    query: web::Query<MenuQuery>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let canteens = util::parse_canteens_comma_separated(&path);
    if canteens.iter().all(Result::is_ok) {
        let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();

        let date = query
            .date
            .unwrap_or_else(|| chrono::Local::now().date_naive());

        let menu = Menu::query(&db, date, &canteens, !query.no_update).await;

        match menu {
            Ok(menu) => HttpResponse::Ok().json(menu),
            Err(err) => {
                tracing::error!("Failed to query database: {err:?}");
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to query database",
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid canteen identifier",
            "invalid": canteens.into_iter().filter_map(|c| c.err()).collect_vec()
        }))
    }
}
