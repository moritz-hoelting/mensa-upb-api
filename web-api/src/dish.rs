use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use shared::Canteen;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dish {
    pub name: String,
    pub image_src: Option<String>,
    pub price: DishPrices,
    pub vegetarian: bool,
    pub vegan: bool,
    pub canteens: Vec<Canteen>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DishPrices {
    pub students: BigDecimal,
    pub employees: BigDecimal,
    pub guests: BigDecimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct DishNutrients {
    pub kjoules: Option<i32>,
    pub carbohydrates: Option<BigDecimal>,
    pub proteins: Option<BigDecimal>,
    pub fats: Option<BigDecimal>,
}

impl Dish {
    pub fn same_as(&self, other: &Self) -> bool {
        self.name == other.name
            && self.price == other.price
            && self.vegan == other.vegan
            && self.vegetarian == other.vegetarian
    }

    pub fn merge(&mut self, other: Self) {
        self.canteens.extend(other.canteens);
        self.canteens.sort();
        self.canteens.dedup();
    }
}

impl PartialOrd for Dish {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl DishPrices {
    pub fn normalize(self) -> Self {
        Self {
            students: self.students.with_prec(5).with_scale(2),
            employees: self.employees.with_prec(5).with_scale(2),
            guests: self.guests.with_prec(5).with_scale(2),
        }
    }
}

impl DishNutrients {
    pub fn normalize(self) -> Self {
        Self {
            kjoules: self.kjoules,
            carbohydrates: self.carbohydrates.map(|v| v.with_prec(6).with_scale(2)),
            proteins: self.proteins.map(|v| v.with_prec(6).with_scale(2)),
            fats: self.fats.map(|v| v.with_prec(6).with_scale(2)),
        }
    }
}
