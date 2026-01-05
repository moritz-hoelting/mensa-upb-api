use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDate;
use itertools::Itertools as _;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{
    util::{self, GenericServerError},
    Menu,
};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(menu);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct MenuQuery {
    date: Option<NaiveDate>,
    #[serde(default)]
    no_update: bool,
}

#[expect(dead_code)]
#[derive(utoipa::ToSchema)]
pub(super) struct InvalidCanteenError {
    error: &'static str,

    /// Which of the given canteen identifiers is invalid
    invalid: Vec<String>,
}

#[utoipa::path(
    summary = "Get menu of canteen(s)", 
    description = "Get the menu of a canteen(s) (at specified date).", 
    params(
        ("canteens" = String, Path, description = "Comma-separated list of canteen identifiers to get the menu for", example = "forum,academica"),
        ("date" = Option<NaiveDate>, Query, description = "Date to get the menu for (defaults to today)"),
        ("noUpdate" = Option<bool>, Query, description = "If set to true, the menu will not be updated before querying (default: false)", example = false),
    ),
    responses(
        (status = OK, description = "The menu of the specified canteen(s).", body = [Menu]),
        (status = BAD_REQUEST, description = "Invalid canteen identifier.", body = InvalidCanteenError, example = json!({
            "error": "Invalid canteen identifier",
            "invalid": ["invalid_canteen_1", "invalid_canteen_2"]
        })),
        (status = INTERNAL_SERVER_ERROR, description = "Server failed to answer request.", body = GenericServerError, example = json!({
            "error": "Failed to query database",
        }))
    )
)]
#[get("/menu/{canteens}")]
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
