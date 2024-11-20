use std::{env, future::Future};

use anyhow::Result;
use chrono::NaiveDate;
use futures::StreamExt as _;
use num_bigint::BigInt;
use sqlx::{postgres::PgPoolOptions, types::BigDecimal, PgPool};

use crate::{menu::scrape_menu, Canteen, Dish};

pub async fn async_for_each<F, Fut>(date_canteen_combinations: &[(NaiveDate, Canteen)], f: F)
where
    F: FnMut((NaiveDate, Canteen, Vec<Dish>)) -> Fut,
    Fut: Future<Output = ()>,
{
    futures::stream::iter(date_canteen_combinations)
        .then(|(date, canteen)| async move { (*date, *canteen, scrape_menu(date, *canteen).await) })
        .filter_map(|(date, canteen, menu)| async move { menu.ok().map(|menu| (date, canteen, menu)) })
        .for_each(f)
        .await;
}

pub fn get_db() -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .connect_lazy(&env::var("DATABASE_URL").expect("missing DATABASE_URL env variable"))?)
}

#[tracing::instrument(skip(db))]
pub async fn add_meal_to_db(db: &PgPool, date: &NaiveDate, canteen: Canteen, dish: &Dish) -> Result<()> {
    let vegan = dish.is_vegan();

    sqlx::query!(
        "INSERT INTO meals (date,canteen,name,dish_type,image_src,price_students,price_employees,price_guests,vegan,vegetarian)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        ON CONFLICT (date,canteen,name) DO NOTHING",
        date, canteen.get_identifier(), dish.get_name(), 
        dish.get_type().to_string(), dish.get_image_src(),
        price_to_bigdecimal(dish.get_price_students()),
        price_to_bigdecimal(dish.get_price_employees()),
        price_to_bigdecimal(dish.get_price_guests()),
        vegan, vegan || dish.is_vegetarian()
    ).execute(db).await.inspect_err(|e| {
        tracing::error!("error during database insert: {}", e);
    })?;

    tracing::trace!("Insert to DB successfull");

    Ok(())
}

pub async fn add_menu_to_db(db: &PgPool, date: &NaiveDate, canteen: Canteen, menu: Vec<Dish>) {
    futures::stream::iter(menu)
        .for_each(|dish| async move {
            if !dish.get_name().is_empty() {
                add_meal_to_db(db, date, canteen, &dish).await.ok();
            }
        })
        .await;
}

pub fn price_to_bigdecimal(s: Option<&str>) -> BigDecimal {
    s.and_then(|p| p.trim_end_matches(" €").replace(',', ".").parse().ok())
        .unwrap_or_else(|| BigDecimal::new(BigInt::from(99999), 2))
}
