use std::{
    collections::{BTreeSet, HashSet},
    str::FromStr,
    sync::LazyLock,
};

use chrono::{NaiveDate, Utc};
use futures::{StreamExt, TryStreamExt as _};
use itertools::Itertools;
use shared::{Canteen, DishType};
use sqlx::QueryBuilder;
use strum::IntoEnumIterator as _;

use crate::{
    Dish,
    dish::NutritionValues,
    util::{self, add_menu_to_db, normalize_price_bigdecimal},
};

static NON_FILTERED_CANTEENS: LazyLock<Vec<Canteen>> = LazyLock::new(|| {
    let all_canteens = Canteen::iter().collect::<HashSet<_>>();

    all_canteens
        .difference(&super::FILTER_CANTEENS)
        .cloned()
        .collect::<Vec<_>>()
});

#[tracing::instrument(skip(db))]
pub async fn check_refresh(
    db: &sqlx::PgPool,
    date: NaiveDate,
    canteens: &[Canteen],
    force: bool,
) -> bool {
    if !force && date > Utc::now().date_naive() + chrono::Duration::days(31) {
        tracing::debug!("Not refreshing menu for date {date} as it is too far in the future");
        return false;
    }

    if !force && date < Utc::now().date_naive() {
        tracing::trace!("Not refreshing menu for date {date} as it is in the past");
        return false;
    }

    let canteens_needing_refresh = if force {
        canteens.iter().cloned().collect::<BTreeSet<_>>()
    } else {
        match sqlx::query!(
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
                .filter(|(c, scraped_at)| 
                    canteens.contains(c) && scraped_at.is_none_or(|scraped_at| needs_refresh(scraped_at, date)))
                .map(|(c, _)| c)
                .collect::<BTreeSet<_>>(),
            Err(err) => {
                tracing::error!("Error checking for existing scrapes: {}", err);
                return false;
            }
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

        let canteen_date_pairs = canteens_needing_refresh
            .iter()
            .map(|c| (date, *c))
            .collect::<Vec<_>>();

        let scraped_dishes = util::scrape_canteens_at_days(&canteen_date_pairs)
            .filter_map(|res| async move { res.ok() })
            .flat_map(|(_, canteen, menu)| {
                futures::stream::iter(menu).map(move |dish| (canteen, dish))
            })
            .collect::<HashSet<_>>();

        let db_data = sqlx::query!(
            r#"SELECT canteen, name, image_src, price_students, price_employees, price_guests, vegetarian, vegan, dish_type AS "dish_type: DishType", kjoules, proteins, carbohydrates, fats FROM meals WHERE date = $1 AND is_latest = TRUE AND canteen = ANY($2)"#,
            date,
            &canteens_needing_refresh
                .iter()
                .map(|c| c.get_identifier().to_string())
                .collect::<Vec<_>>(),
        ).map(|r| {
            (
                Canteen::from_str(&r.canteen).expect("malformed db entry") ,
                Dish {
                    name: r.name,
                    image_src: r.image_src,
                    price_students: normalize_price_bigdecimal(r.price_students),
                    price_employees: normalize_price_bigdecimal(r.price_employees),
                    price_guests: normalize_price_bigdecimal(r.price_guests),
                    vegetarian: r.vegetarian,
                    vegan: r.vegan,
                    dish_type: r.dish_type,
                    nutrition_values: NutritionValues {
                        kjoule: r.kjoules,
                        protein: r.proteins,
                        carbs: r.carbohydrates,
                        fat: r.fats,
                    }.normalize(),
                }
        )
        }).fetch(db).try_collect::<HashSet<_>>();

        let (scraped_dishes, db_data) = futures::join!(scraped_dishes, db_data);

        match db_data {
            Ok(db_dishes) => {
                let stale_dishes = db_dishes
                    .difference(&scraped_dishes)
                    .collect::<HashSet<_>>();
                let new_dishes = scraped_dishes
                    .difference(&db_dishes)
                    .collect::<HashSet<_>>();

                if let Err(err) =
                    update_stale_dishes(db, date, &stale_dishes, &new_dishes, canteens).await
                {
                    tracing::error!("Error updating stale dishes in db: {}", err);
                    false
                } else {
                    true
                }
            }
            Err(err) => {
                tracing::error!("Error fetching existing dishes from db: {}", err);
                false
            }
        }
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

#[tracing::instrument(skip(db, date, stale_dishes, new_dishes), fields(date = %date, stale_dish_count = %stale_dishes.len(), new_dish_count = %new_dishes.len()))]
async fn update_stale_dishes(
    db: &sqlx::PgPool,
    date: NaiveDate,
    stale_dishes: &HashSet<&(Canteen, Dish)>,
    new_dishes: &HashSet<&(Canteen, Dish)>,
    canteens: &[Canteen],
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;

    if !stale_dishes.is_empty() {
        QueryBuilder::new("UPDATE meals SET is_latest = FALSE WHERE date = ")
            .push_bind(date)
            .push(r#" AND ("name", canteen) IN "#)
            .push_tuples(stale_dishes, |mut sep, (canteen, dish)| {
                sep.push_bind(&dish.name)
                    .push_bind(canteen.get_identifier());
            })
            .push(";")
            .build()
            .execute(&mut *tx)
            .await?;

        if new_dishes.is_empty() {
            tracing::debug!("No new dishes to add after marking stale dishes");
        }
    }

    let chunks = new_dishes
        .iter()
        .sorted_by_key(|(c, _)| c)
        .chunk_by(|(c, _)| c);

    let new_dishes_iter = chunks
        .into_iter()
        .map(|(canteen, g)| {
            (
                *canteen,
                g.map(|(_, dish)| dish).cloned().collect::<Vec<_>>(),
            )
        })
        .chain(canteens.iter().map(|canteen| (*canteen, Vec::new())))
        .unique_by(|(c, _)| *c)
        .collect::<Vec<_>>();

    for (canteen, menu) in new_dishes_iter {
        add_menu_to_db(&mut tx, &date, canteen, menu).await?;
    }

    tx.commit().await?;

    Ok(())
}
