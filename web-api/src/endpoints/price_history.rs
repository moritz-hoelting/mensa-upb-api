use std::collections::BTreeMap;

use actix_web::{
    get,
    web::{self, ServiceConfig},
    HttpResponse, Responder,
};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{prelude::FromRow, PgPool};

use crate::{util, DishPrices};

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(price_history);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PriceHistoryQuery {
    canteens: Option<String>,
    limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, FromRow)]
struct PriceHistoryRow {
    date: NaiveDate,
    canteen: String,
    price_students: BigDecimal,
    price_employees: BigDecimal,
    price_guests: BigDecimal,
}

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
    let limit = query.limit.unwrap_or(1000) as i64;

    if let Some(canteens) = canteens {
        if canteens.iter().all(Result::is_ok) {
            let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();

            if canteens.len() == 1 {
                let canteen = canteens.into_iter().next().expect("length is 1");

                let res = sqlx::query!(
                    r#"SELECT date, price_students, price_employees, price_guests FROM meals WHERE canteen = $1 AND LOWER("name") = $2 AND is_latest = TRUE ORDER BY date DESC LIMIT $3;"#,
                    canteen.get_identifier(),
                    dish_name.to_lowercase(),
                    limit,
                )
                .fetch_all(db)
                .await;

                match res {
                    Ok(recs) => {
                        let structured = recs
                            .into_iter()
                            .map(|r| {
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
                            .collect::<BTreeMap<_, _>>();

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
