use std::{collections::BTreeSet, str::FromStr};

use chrono::{NaiveDate, Utc};
use shared::Canteen;

use crate::util;

pub async fn check_refresh(db: &sqlx::PgPool, date: NaiveDate, canteens: &[Canteen]) -> bool {
    let canteens_needing_refresh = match sqlx::query!(
        r#"SELECT canteen, max(scraped_at) AS "scraped_at!" FROM canteens_scraped WHERE canteen = ANY($1) AND scraped_for = $2 GROUP BY canteen"#,
        &canteens
            .iter()
            .map(|c| c.get_identifier().to_string())
            .collect::<Vec<_>>(),
        date
    )
    .fetch_all(db)
    .await
    {
        Ok(v) => v.iter().filter_map(|r| if needs_refresh(r.scraped_at, date) { Some(Canteen::from_str(&r.canteen).expect("malformed db canteen entry")) } else { None }).collect::<BTreeSet<_>>(),
        Err(err) => {
            tracing::error!("Error checking for existing scrapes: {}", err);
            return false;
        }
    };

    if canteens_needing_refresh.is_empty() {
        false
    } else {
        tracing::debug!(
            "Refreshing menu for date {} for canteens: {:?}",
            date,
            canteens_needing_refresh
        );

        if let Err(err) = util::scrape_canteens_at_days(
            db,
            &canteens_needing_refresh
                .iter()
                .map(|c| (date, *c))
                .collect::<Vec<_>>(),
        )
        .await
        {
            tracing::error!("Error during refresh scrape: {}", err);
            return false;
        }

        true
    }
}

fn needs_refresh(last_refreshed: chrono::DateTime<Utc>, date_entry: chrono::NaiveDate) -> bool {
    let now = Utc::now();

    if date_entry == now.naive_local().date() {
        now.signed_duration_since(last_refreshed) >= chrono::Duration::hours(8)
    } else if date_entry < now.naive_local().date() {
        false
    } else {
        now.signed_duration_since(last_refreshed) >= chrono::Duration::days(2)
    }
}
