use std::{
    collections::{BTreeSet, HashSet},
    str::FromStr,
    sync::LazyLock,
};

use chrono::{NaiveDate, Utc};
use itertools::Itertools;
use shared::Canteen;
use strum::IntoEnumIterator as _;

use crate::util;

static NON_FILTERED_CANTEENS: LazyLock<Vec<Canteen>> = LazyLock::new(|| {
    let all_canteens = Canteen::iter().collect::<HashSet<_>>();

    all_canteens
        .difference(&super::FILTER_CANTEENS)
        .cloned()
        .collect::<Vec<_>>()
});

#[tracing::instrument(skip(db))]
pub async fn check_refresh(db: &sqlx::PgPool, date: NaiveDate, canteens: &[Canteen]) -> bool {
    if date > Utc::now().date_naive() + chrono::Duration::days(7) {
        tracing::debug!("Not refreshing menu for date {date} as it is too far in the future");
        return false;
    }

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
        Ok(v) => v
            .iter()
            .map(|r| (Canteen::from_str(&r.canteen).expect("malformed db entry"), Some(r.scraped_at)))
            .chain(NON_FILTERED_CANTEENS.iter().filter(|c| canteens.contains(c)).map(|c| (*c, None)))
            .unique_by(|(c, _)| *c)
            .filter(|(_, scraped_at)| scraped_at.is_none_or(|scraped_at| needs_refresh(scraped_at, date)))
            .map(|(c, _)| c)
            .collect::<BTreeSet<_>>(),
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
