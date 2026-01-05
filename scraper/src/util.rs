use std::env;

use anyhow::Result;
use chrono::NaiveDate;
use futures::{Stream, StreamExt as _};
use shared::{Canteen, DishType};
use sqlx::{postgres::PgPoolOptions, types::Decimal, PgPool, PgTransaction};

use crate::{scrape_menu, Dish};

pub fn get_db() -> Result<PgPool> {
    Ok(PgPoolOptions::new()
        .connect_lazy(&env::var("DATABASE_URL").expect("missing DATABASE_URL env variable"))?)
}

pub fn scrape_canteens_at_days<'a>(
    date_canteen_combinations: &'a [(NaiveDate, Canteen)],
) -> impl Stream<Item = Result<(NaiveDate, Canteen, Vec<Dish>)>> + 'a {
    futures::stream::iter(date_canteen_combinations).then(|(date, canteen)| async move {
        scrape_menu(date, *canteen)
            .await
            .map(|menu| (*date, *canteen, menu))
    })
}

pub async fn add_menu_to_db(
    db: &mut PgTransaction<'_>,
    date: &NaiveDate,
    canteen: Canteen,
    menu: Vec<Dish>,
) -> Result<(), sqlx::Error> {
    if !menu.is_empty() {
        let mut query = sqlx::QueryBuilder::new("INSERT INTO meals (date,canteen,name,dish_type,image_src,price_students,price_employees,price_guests,vegan,vegetarian,kjoules,proteins,carbohydrates,fats) ");

        query
            .push_values(menu, |mut sep, item| {
                let vegan = item.is_vegan();

                sep.push_bind(date)
                    .push_bind(canteen.get_identifier())
                    .push_bind(item.get_name().to_string())
                    .push_bind(item.get_type() as DishType)
                    .push_bind(item.get_image_src().map(str::to_string))
                    .push_bind(item.get_price_students().to_owned())
                    .push_bind(item.get_price_employees().to_owned())
                    .push_bind(item.get_price_guests().to_owned())
                    .push_bind(vegan)
                    .push_bind(vegan || item.is_vegetarian())
                    .push_bind(item.nutrition_values.kjoule)
                    .push_bind(item.nutrition_values.protein.to_owned())
                    .push_bind(item.nutrition_values.carbs.to_owned())
                    .push_bind(item.nutrition_values.fat.to_owned());
            })
            .build()
            .execute(&mut **db)
            .await?;
    }

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

pub fn normalize_price_bigdecimal(price: Decimal) -> Decimal {
    price.normalize().round_dp(2)
}
