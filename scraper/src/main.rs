use std::{collections::HashSet, env};

use anyhow::Result;
use chrono::{Duration, Utc};
use itertools::Itertools as _;
use mensa_upb_scraper::{util, Canteen};
use strum::IntoEnumIterator;

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
        "SELECT DISTINCT date, canteen FROM MEALS WHERE date >= $1 AND date <= $2",
        start_date,
        end_date
    )
    .fetch_all(&db)
    .await?
    .into_iter()
    .map(|r| {
        (
            r.date,
            r.canteen.parse::<Canteen>().expect("Invalid db entry"),
        )
    })
    .collect::<HashSet<_>>();

    let filter_canteens = env::var("FILTER_CANTEENS")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|el| el.parse::<Canteen>().ok())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();

    let date_canteen_combinations = (0..7)
        .map(|d| (Utc::now() + Duration::days(d)).date_naive())
        .cartesian_product(Canteen::iter())
        .filter(|entry| !filter_canteens.contains(&entry.1) && !already_scraped.contains(entry))
        .collect::<Vec<_>>();
    util::async_for_each(&date_canteen_combinations, |(date, canteen, menu)| {
        let db = db.clone();
        async move {
            util::add_menu_to_db(&db, &date, canteen, menu).await;
        }
    })
    .await;

    tracing::info!("Finished scraping menu");

    Ok(())
}
