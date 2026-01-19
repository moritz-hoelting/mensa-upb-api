use anyhow::Result;
use clap::Parser;
use futures::future;
use mensa_upb_scraper::check_refresh;
use sqlx::postgres::PgPoolOptions;
use strum::IntoEnumIterator as _;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, clap::Parser)]
struct Cli {
    /// Database connection string
    #[clap(env = "DATABASE_URL")]
    database: String,
    /// Canteen to scrape
    #[clap(short, long = "canteen")]
    canteens: Vec<shared::Canteen>,
    /// Date to scrape (YYYY-MM-DD)
    #[clap(short, long = "date", required = true)]
    dates: Vec<chrono::NaiveDate>,
    /// Force refresh even if not needed
    #[clap(short, long)]
    force: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let mut cli = Cli::parse();

    if cli.canteens.is_empty() {
        cli.canteens = shared::Canteen::iter().collect();
    }

    let db = PgPoolOptions::new().connect_lazy(&cli.database)?;

    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .from_env()
        .expect("Invalid filter")
        .add_directive("mensa_upb_scraper=debug".parse().unwrap());
    tracing_subscriber::fmt().with_env_filter(env_filter).init();

    sqlx::migrate!("../migrations").run(&db).await?;

    tracing::info!("Starting up...");

    let handles = cli.dates.into_iter().map(|date| {
        let db = db.clone();
        let canteens = cli.canteens.clone();
        tokio::spawn(async move { check_refresh(&db, date, &canteens, cli.force).await })
    });

    future::join_all(handles).await;

    tracing::info!("Finished scraping menu");

    Ok(())
}
