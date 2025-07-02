use std::env;

use actix_cors::Cors;
use actix_governor::{Governor, GovernorConfigBuilder};
use actix_web::{web, App, HttpServer};
use anyhow::Result;
use itertools::Itertools;
use sqlx::postgres::PgPoolOptions;
use tracing::{debug, error, info, level_filters::LevelFilter};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .expect("Invalid filter")
        .add_directive("mensa_upb_api=debug".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    match dotenvy::dotenv() {
        Ok(_) => debug!("Loaded .env file"),
        Err(dotenvy::Error::LineParse(..)) => error!("Malformed .env file"),
        Err(_) => {}
    }

    let db = PgPoolOptions::new()
        .connect_lazy(&env::var("DATABASE_URL").expect("missing DATABASE_URL env variable"))?;

    sqlx::migrate!("../migrations").run(&db).await?;

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
        .seconds_per_request(seconds_replenish)
        .burst_size(burst_size)
        .finish()
        .unwrap();

    info!("Starting server on {}:{}", interface, port);

    HttpServer::new(move || {
        let cors = allowed_cors
            .iter()
            .fold(Cors::default(), |cors, domain| {
                if domain == "*" {
                    cors.allow_any_origin()
                } else {
                    cors.allowed_origin(domain)
                }
            })
            .send_wildcard()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        App::new()
            .wrap(Governor::new(&governor_conf))
            .wrap(cors)
            .app_data(web::Data::new(db.clone()))
            .configure(mensa_upb_api::endpoints::configure)
    })
    .bind((interface.as_str(), port))?
    .run()
    .await?;

    Ok(())
}
