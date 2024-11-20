use std::str::FromStr as _;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{Canteen, Dish, DishPrices};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Menu {
    date: NaiveDate,
    main_dishes: Vec<Dish>,
    side_dishes: Vec<Dish>,
    desserts: Vec<Dish>,
}

impl Menu {
    pub async fn query(db: &PgPool, date: NaiveDate, canteens: &[Canteen]) -> sqlx::Result<Self> {
        let canteens = canteens
            .iter()
            .map(|c| c.get_identifier().to_string())
            .collect::<Vec<_>>();
        let result = sqlx::query!("SELECT name, array_agg(DISTINCT canteen ORDER BY canteen) AS canteens, dish_type, image_src, price_students, price_employees, price_guests, vegan, vegetarian 
                FROM meals WHERE date = $1 AND canteen = ANY($2) 
                GROUP BY name, dish_type, image_src, price_students, price_employees, price_guests, vegan, vegetarian
                ORDER BY name", 
                date, &canteens)
            .fetch_all(db)
            .await?;

        let mut main_dishes = Vec::new();
        let mut side_dishes = Vec::new();
        let mut desserts = Vec::new();

        for row in result {
            let dish = Dish {
                name: row.name,
                image_src: row.image_src,
                canteens: row.canteens.map_or_else(Vec::new, |canteens| {
                    canteens
                        .iter()
                        .map(|canteen| Canteen::from_str(canteen).expect("Invalid database entry"))
                        .collect()
                }),
                vegan: row.vegan,
                vegetarian: row.vegetarian,
                price: DishPrices {
                    students: row.price_students.with_prec(5).with_scale(2),
                    employees: row.price_employees.with_prec(5).with_scale(2),
                    guests: row.price_guests.with_prec(5).with_scale(2),
                },
            };
            if row.dish_type == "main" {
                main_dishes.push(dish);
            } else if row.dish_type == "side" {
                side_dishes.push(dish);
            } else if row.dish_type == "dessert" {
                desserts.push(dish);
            }
        }

        Ok(Self {
            date,
            main_dishes,
            side_dishes,
            desserts,
        })
    }

    pub fn get_main_dishes(&self) -> &[Dish] {
        &self.main_dishes
    }

    pub fn get_side_dishes(&self) -> &[Dish] {
        &self.side_dishes
    }

    pub fn get_desserts(&self) -> &[Dish] {
        &self.desserts
    }

    pub fn merged(self, other: Self) -> Self {
        let mut main_dishes = self.main_dishes;
        let mut side_dishes = self.side_dishes;
        let mut desserts = self.desserts;

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
        for dish in other.desserts {
            if let Some(existing) = desserts.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                desserts.push(dish);
            }
        }

        Self {
            date: self.date,
            main_dishes,
            side_dishes,
            desserts,
        }
    }
}
