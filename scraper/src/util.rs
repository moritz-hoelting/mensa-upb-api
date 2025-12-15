use std::env;

use anyhow::Result;
use chrono::NaiveDate;
use futures::StreamExt as _;
use num_bigint::BigInt;
use shared::{Canteen, DishType};
use sqlx::{postgres::PgPoolOptions, types::BigDecimal, PgPool, PgTransaction};

use crate::{scrape_menu, Dish};

pub fn get_db() -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .connect_lazy(&env::var("DATABASE_URL").expect("missing DATABASE_URL env variable"))?)
}

pub async fn scrape_canteens_at_days(
    db: &PgPool,
    date_canteen_combinations: &[(NaiveDate, Canteen)],
) -> Result<()> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<(NaiveDate, Canteen, Vec<Dish>)>(128);

    let mut transaction = db.begin().await?;

    for (date, canteen) in date_canteen_combinations {
        sqlx::query!(
            "UPDATE meals SET is_latest = FALSE WHERE date = $1 AND canteen = $2 AND is_latest = TRUE",
            date,
            canteen.get_identifier()
        )
        .execute(&mut *transaction)
        .await
        .ok();
    }

    let insert_handle = tokio::spawn(async move {
        while let Some((date, canteen, menu)) = rx.recv().await {
            add_menu_to_db(&mut transaction, &date, canteen, menu).await?;
        }

        transaction.commit().await
    });

    futures::stream::iter(date_canteen_combinations)
        .then(|(date, canteen)| async move { (*date, *canteen, scrape_menu(date, *canteen).await) })
        .filter_map(
            |(date, canteen, menu)| async move { menu.ok().map(|menu| (date, canteen, menu)) },
        )
        .for_each(|(date, canteen, menu)| {
            let tx = tx.clone();
            async move {
                tx.send((date, canteen, menu)).await.ok();
            }
        })
        .await;

    drop(tx);

    insert_handle.await??;

    Ok(())
}

pub async fn add_menu_to_db(
    db: &mut PgTransaction<'_>,
    date: &NaiveDate,
    canteen: Canteen,
    menu: Vec<Dish>,
) -> Result<(), sqlx::Error> {
    if menu.is_empty() {
        return Ok(());
    }

    let mut query = sqlx::QueryBuilder::new("INSERT INTO meals (date,canteen,name,dish_type,image_src,price_students,price_employees,price_guests,vegan,vegetarian) ");

    query
        .push_values(menu, |mut sep, item| {
            let vegan = item.is_vegan();

            sep.push_bind(date)
                .push_bind(canteen.get_identifier())
                .push_bind(item.get_name().to_string())
                .push_bind(item.get_type() as DishType)
                .push_bind(item.get_image_src().map(str::to_string))
                .push_bind(price_to_bigdecimal(item.get_price_students()))
                .push_bind(price_to_bigdecimal(item.get_price_employees()))
                .push_bind(price_to_bigdecimal(item.get_price_guests()))
                .push_bind(vegan)
                .push_bind(vegan || item.is_vegetarian());
        })
        .build()
        .execute(&mut **db)
        .await?;

    sqlx::query!(
        "INSERT INTO canteens_scraped (scraped_for, canteen) VALUES ($1, $2)",
        date,
        canteen.get_identifier()
    )
    .execute(&mut **db)
    .await?;

    tracing::trace!("Insert to DB successfull");

    Ok(())
}

pub fn price_to_bigdecimal(s: Option<&str>) -> BigDecimal {
    s.and_then(|p| p.trim_end_matches(" €").replace(',', ".").parse().ok())
        .unwrap_or_else(|| BigDecimal::from_bigint(BigInt::from(99999), 2))
}
