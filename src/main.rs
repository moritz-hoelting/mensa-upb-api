use std::{env, io, str::FromStr};

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use chrono::{Duration as CDuration, Utc};
use itertools::Itertools;
use mensa_upb_api::{Canteen, MenuCache};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;

#[actix_web::main]
async fn main() -> io::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => println!("Loaded .env file"),
        Err(dotenvy::Error::LineParse(..)) => eprintln!("Malformed .env file"),
        Err(_) => {}
    }

    let interface = env::var("API_INTERFACE").unwrap_or("127.0.0.1".to_string());
    let port = env::var("API_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let seconds_replenish = env::var("API_RATE_LIMIT_SECONDS")
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(5);
    let burst_size = env::var("API_RATE_LIMIT_BURST")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(5);

    let allowed_cors = env::var("API_CORS_ALLOWED")
        .map(|val| {
            val.split(',')
                .map(|domain| domain.trim().to_string())
                .collect_vec()
        })
        .ok()
        .unwrap_or_default();

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(seconds_replenish)
        .burst_size(burst_size)
        .finish()
        .unwrap();

    let menu_cache = MenuCache::default();

    println!("Starting server on {}:{}", interface, port);

    HttpServer::new(move || {
        let cors = allowed_cors
            .iter()
            .fold(Cors::default(), |cors, domain| cors.allowed_origin(domain))
            .send_wildcard()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(cors)
            .app_data(web::Data::new(menu_cache.clone()))
            .service(index)
            .service(menu_today)
    })
    .bind((interface.as_str(), port))?
    .run()
    .await
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "supportedCanteens": Canteen::iter().map(|c| c.get_identifier().to_string()).collect_vec(),
    }))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
struct MenuQuery {
    #[serde(rename = "d")]
    days_ahead: Option<String>,
}

#[get("/menu/{canteen}")]
async fn menu_today(
    cache: web::Data<MenuCache>,
    path: web::Path<String>,
    query: web::Query<MenuQuery>,
) -> impl Responder {
    let canteens = path
        .into_inner()
        .split(',')
        .map(Canteen::from_str)
        .collect_vec();
    if canteens.iter().all(Result::is_ok) {
        let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();
        let days_ahead = query
            .days_ahead
            .as_ref()
            .map_or(Ok(0), |d| d.parse::<i64>());

        if let Ok(days_ahead) = days_ahead {
            let date = (Utc::now() + CDuration::days(days_ahead)).date_naive();
            let menu = cache.get_combined(&canteens, date).await;

            HttpResponse::Ok().json(menu)
        } else {
            HttpResponse::BadRequest().json(json!({
                "error": "Invalid days query"
            }))
        }
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid canteen identifier",
            "invalid": canteens.into_iter().filter_map(|c| c.err()).collect_vec()
        }))
    }
}
