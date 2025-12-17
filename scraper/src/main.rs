use std::collections::HashSet;

use anyhow::Result;
use chrono::{Duration, Utc};
use itertools::Itertools as _;
use mensa_upb_scraper::{util, FILTER_CANTEENS};
use shared::Canteen;
use strum::IntoEnumIterator as _;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let db = util::get_db()?;

    tracing_subscriber::fmt::init();

    sqlx::migrate!("../migrations").run(&db).await?;

    tracing::info!("Starting up...");

    let start_date = Utc::now().date_naive();
    let end_date = (Utc::now() + Duration::days(6)).date_naive();

    let already_scraped = sqlx::query!(
        "SELECT DISTINCT scraped_for, canteen FROM canteens_scraped WHERE scraped_for >= $1 AND scraped_for <= $2",
        start_date,
        end_date
    )
    .fetch_all(&db)
    .await?
    .into_iter()
    .map(|r| {
        (
            r.scraped_for,
            r.canteen.parse::<Canteen>().expect("Invalid db entry"),
        )
    })
    .collect::<HashSet<_>>();

    let date_canteen_combinations = (0..7)
        .map(|d| (Utc::now() + Duration::days(d)).date_naive())
        .cartesian_product(Canteen::iter())
        .filter(|entry @ (_, canteen)| {
            !FILTER_CANTEENS.contains(canteen) && !already_scraped.contains(entry)
        })
        .collect::<Vec<_>>();

    util::scrape_canteens_at_days(&db, &date_canteen_combinations).await?;

    tracing::info!("Finished scraping menu");

    Ok(())
}
