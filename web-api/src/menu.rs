use chrono::NaiveDate;
use mensa_upb_scraper::check_refresh;
use serde::{Deserialize, Serialize};
use shared::{Canteen, DishType};
use sqlx::PgPool;
use std::str::FromStr as _;

use crate::{Dish, DishPrices};

#[derive(Debug, Clone, Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct Menu {
    date: NaiveDate,
    main_dishes: Vec<Dish>,
    side_dishes: Vec<Dish>,
    soup_dishes: Vec<Dish>,
    dessert_dishes: Vec<Dish>,
    other_dishes: Vec<Dish>,
}

impl Menu {
    pub async fn query(
        db: &PgPool,
        date: NaiveDate,
        canteens: &[Canteen],
        allow_refresh: bool,
    ) -> sqlx::Result<Self> {
        let canteens_str = canteens
            .iter()
            .map(|c| c.get_identifier().to_string())
            .collect::<Vec<_>>();

        if allow_refresh {
            check_refresh(db, date, canteens, false).await;
        };

        let result = sqlx::query!(r#"SELECT name, array_agg(DISTINCT canteen ORDER BY canteen) AS "canteens!", dish_type AS "dish_type: DishType", image_src, price_students, price_employees, price_guests, vegan, vegetarian 
                FROM meals WHERE date = $1 AND canteen = ANY($2) AND is_latest = TRUE
                GROUP BY name, dish_type, image_src, price_students, price_employees, price_guests, vegan, vegetarian
                ORDER BY name"#, 
                date, &canteens_str)
            .fetch_all(db)
            .await?;

        let mut main_dishes = Vec::new();
        let mut side_dishes = Vec::new();
        let mut soup_dishes = Vec::new();
        let mut dessert_dishes = Vec::new();
        let mut other_dishes = Vec::new();

        for row in result {
            let dish = Dish {
                name: row.name,
                image_src: row.image_src,
                canteens: row
                    .canteens
                    .iter()
                    .map(|canteen| Canteen::from_str(canteen).expect("Invalid database entry"))
                    .collect(),
                vegan: row.vegan,
                vegetarian: row.vegetarian,
                price: DishPrices {
                    students: row.price_students,
                    employees: row.price_employees,
                    guests: row.price_guests,
                }
                .normalize(),
            };
            if row.dish_type == DishType::Main {
                main_dishes.push(dish);
            } else if row.dish_type == DishType::Side {
                side_dishes.push(dish);
            } else if row.dish_type == DishType::Soup {
                soup_dishes.push(dish);
            } else if row.dish_type == DishType::Dessert {
                dessert_dishes.push(dish);
            } else if row.dish_type == DishType::Other {
                other_dishes.push(dish);
            }
        }

        Ok(Self {
            date,
            main_dishes,
            side_dishes,
            soup_dishes,
            dessert_dishes,
            other_dishes,
        })
    }

    pub fn get_main_dishes(&self) -> &[Dish] {
        &self.main_dishes
    }

    pub fn get_side_dishes(&self) -> &[Dish] {
        &self.side_dishes
    }

    pub fn get_soup_dishes(&self) -> &[Dish] {
        &self.soup_dishes
    }

    pub fn get_dessert_dishes(&self) -> &[Dish] {
        &self.dessert_dishes
    }

    pub fn get_other_dishes(&self) -> &[Dish] {
        &self.other_dishes
    }

    pub fn merged(self, other: Self) -> Self {
        let mut main_dishes = self.main_dishes;
        let mut side_dishes = self.side_dishes;
        let mut soup_dishes = self.soup_dishes;
        let mut dessert_dishes = self.dessert_dishes;
        let mut other_dishes = self.other_dishes;

        for dish in other.main_dishes {
            if let Some(existing) = main_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                main_dishes.push(dish);
            }
        }
        for dish in other.side_dishes {
            if let Some(existing) = side_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                side_dishes.push(dish);
            }
        }
        for dish in other.soup_dishes {
            if let Some(existing) = soup_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                soup_dishes.push(dish);
            }
        }
        for dish in other.dessert_dishes {
            if let Some(existing) = dessert_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                dessert_dishes.push(dish);
            }
        }
        for dish in other.other_dishes {
            if let Some(existing) = other_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                other_dishes.push(dish);
            }
        }

        Self {
            date: self.date,
            main_dishes,
            side_dishes,
            soup_dishes,
            dessert_dishes,
            other_dishes,
        }
    }
}
