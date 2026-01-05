use std::collections::BTreeMap;

use actix_web::{get, web, HttpResponse, Responder};
use chrono::NaiveDate;
use itertools::Itertools;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{prelude::FromRow, PgPool};
use utoipa_actix_web::service_config::ServiceConfig;

use crate::{
    endpoints::menu::InvalidCanteenError,
    util::{self, GenericServerError},
    DishPrices,
};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(price_history);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
struct PriceHistoryQuery {
    canteens: Option<String>,
    limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, FromRow)]
struct PriceHistoryRow {
    date: NaiveDate,
    canteen: String,
    price_students: Decimal,
    price_employees: Decimal,
    price_guests: Decimal,
}

#[utoipa::path(
    summary = "Get price history of a dish",
    description = "Query the price history of a dish (optionally filtered by canteen(s)).",
    params(
        ("name" = String, Path, description = "Name of the dish to query price history for", example = "Bratwurst mit Currysauce und Pommes Frites"),
        ("canteens" = Option<String>, Query, description = "Comma-separated list of canteen identifiers to filter the price history by", example = "forum,academica"), 
        ("limit" = Option<u32>, Query, description = "Maximum number of entries to return", minimum = 1, maximum = 1000, example = 100),
    ),
    responses(
        (status = OK, description = "Query the price history of a dish.", body = BTreeMap<String, BTreeMap<NaiveDate, DishPrices>>, example = json!({
            "forum": {
                "2024-06-01": {
                    "students": "2.50",
                    "employees": "3.50",
                    "guests": "4.50"
                },
                "2024-05-31": {
                    "students": "2.40",
                    "employees": "3.40",
                    "guests": "4.40"
                }
            },
            "academica": {
                "2024-06-01": {
                    "students": "2.60",
                    "employees": "3.60",
                    "guests": "4.60"
                }
            }
        })),
        (status = BAD_REQUEST, description = "Invalid canteen identifier.", body = InvalidCanteenError, example = json!({
            "error": "Invalid canteen identifier",
            "invalid": ["invalid_canteen_1", "invalid_canteen_2"]
        })),
        (status = INTERNAL_SERVER_ERROR, description = "Server failed to answer request.", body = GenericServerError, example = json!({
            "error": "Failed to query database",
        }))
    )
)]
#[get("/price-history/{name}")]
async fn price_history(
    path: web::Path<String>,
    query: web::Query<PriceHistoryQuery>,
    db: web::Data<PgPool>,
) -> impl Responder {
    let db = db.as_ref();
    let canteens = query
        .canteens
        .as_deref()
        .map(util::parse_canteens_comma_separated);
    let dish_name = path.into_inner();
    let limit = query.limit.unwrap_or(1000).clamp(1, 1000) as i64;

    if let Some(canteens) = canteens {
        if canteens.iter().all(Result::is_ok) {
            let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();

            let res = sqlx::query_as!(PriceHistoryRow,
                    r#"SELECT date, canteen, price_students, price_employees, price_guests FROM meals WHERE canteen = ANY($1) AND LOWER("name") = $2 AND is_latest = TRUE ORDER BY date DESC LIMIT $3;"#,
                    &canteens.iter().map(|c| c.get_identifier().to_string()).collect_vec(),
                    dish_name.to_lowercase(),
                    limit
                )
                .fetch_all(db)
                .await;

            match res {
                Ok(recs) => {
                    let structured = structure_multiple_canteens(recs);

                    HttpResponse::Ok().json(structured)
                }
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
    } else {
        let res = sqlx::query_as!(PriceHistoryRow,
            r#"SELECT date, canteen, price_students, price_employees, price_guests FROM meals WHERE LOWER("name") = $1 AND is_latest = TRUE ORDER BY date DESC LIMIT $2;"#,
            dish_name.to_lowercase(),
            limit as i64,
        )
        .fetch_all(db)
        .await;

        match res {
            Ok(recs) => {
                let structured = structure_multiple_canteens(recs);

                HttpResponse::Ok().json(structured)
            }
            Err(err) => {
                tracing::error!("Failed to query database: {err:?}");
                HttpResponse::InternalServerError().json(json!({
                    "error": "Failed to query database",
                }))
            }
        }
    }
}

fn structure_multiple_canteens(
    v: Vec<PriceHistoryRow>,
) -> BTreeMap<String, BTreeMap<NaiveDate, DishPrices>> {
    v.into_iter()
        .sorted_by_cached_key(|r| r.canteen.clone())
        .chunk_by(|r| r.canteen.clone())
        .into_iter()
        .map(|(d, g)| {
            (
                d,
                g.map(|r| {
                    (
                        r.date,
                        DishPrices {
                            students: r.price_students,
                            employees: r.price_employees,
                            guests: r.price_guests,
                        }
                        .normalize(),
                    )
                })
                .collect(),
            )
        })
        .collect()
}
