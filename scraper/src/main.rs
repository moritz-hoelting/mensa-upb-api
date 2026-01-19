use std::sync::LazyLock;

use anyhow::Result;
use chrono::{Duration, Utc};
use futures::future;
use mensa_upb_scraper::{FILTER_CANTEENS, check_refresh, util};
use shared::Canteen;
use strum::IntoEnumIterator as _;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

static CANTEENS: LazyLock<Vec<Canteen>> = LazyLock::new(|| {
    Canteen::iter()
        .filter(|c| !FILTER_CANTEENS.contains(c))
        .collect::<Vec<_>>()
});

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db = util::get_db()?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .expect("Invalid filter")
        .add_directive("mensa_upb_scraper=debug".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    sqlx::migrate!("../migrations").run(&db).await?;

    tracing::info!("Starting up...");

    let handles = (0..7)
        .map(|d| (Utc::now() + Duration::days(d)).date_naive())
        .map(|date| {
            let db = db.clone();
            tokio::spawn(async move { check_refresh(&db, date, &CANTEENS, false).await })
        });

    future::join_all(handles).await;

    tracing::info!("Finished scraping menu");

    Ok(())
}
