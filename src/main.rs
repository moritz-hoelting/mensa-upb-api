use std::{env, io, str::FromStr};

use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use chrono::{Duration as CDuration, Utc};
use itertools::Itertools;
use mensa_upb_api::{Canteen, Menu};
use serde::{Deserialize, Serialize};
use serde_json::json;
use strum::IntoEnumIterator;

#[actix_web::main]
async fn main() -> io::Result<()> {
    if dotenvy::dotenv().is_ok() {
        println!("Loaded .env file");
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

    let governor_conf = GovernorConfigBuilder::default()
        .per_second(seconds_replenish)
        .burst_size(burst_size)
        .finish()
        .unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Governor::new(&governor_conf))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
struct MenuQuery {
    #[serde(rename = "d")]
    days_ahead: Option<u32>,
}

#[get("/menu/{canteen}")]
async fn menu_today(path: web::Path<String>, query: web::Query<MenuQuery>) -> impl Responder {
    let canteens = path
        .into_inner()
        .split(',')
        .map(Canteen::from_str)
        .collect_vec();
    if canteens.iter().all(Result::is_ok) {
        let canteens = canteens.into_iter().filter_map(Result::ok).collect_vec();
        let days_ahead = query.days_ahead.unwrap_or(0);

        let menu = Menu::new(
            (Utc::now() + CDuration::days(days_ahead as i64)).date_naive(),
            &canteens,
        )
        .await
        .unwrap();

        HttpResponse::Ok().json(menu)
    } else {
        HttpResponse::BadRequest().json(json!({
            "error": "Invalid canteen identifier",
            "invalid": canteens.into_iter().filter_map(|c| c.err()).collect_vec()
        }))
    }
}
