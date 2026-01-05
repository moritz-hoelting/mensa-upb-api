use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use shared::Canteen;
use sqlx::prelude::FromRow;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct Dish {
    pub name: String,
    pub image_src: Option<String>,
    pub price: DishPrices,
    pub vegetarian: bool,
    pub vegan: bool,
    pub canteens: Vec<Canteen>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, utoipa::ToSchema)]
pub struct DishPrices {
    pub students: Decimal,
    pub employees: Decimal,
    pub guests: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, FromRow, utoipa::ToSchema)]
#[schema(examples(
    json!({
        "kjoules": 1500,
        "carbohydrates": "45.5",
        "proteins": "30.0",
        "fats": "10.0"
    })
))]
pub struct DishNutrients {
    pub kjoules: Option<i32>,
    pub carbohydrates: Option<Decimal>,
    pub proteins: Option<Decimal>,
    pub fats: Option<Decimal>,
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
            students: self.students.normalize().round_dp(2),
            employees: self.employees.normalize().round_dp(2),
            guests: self.guests.normalize().round_dp(2),
        }
    }
}

impl DishNutrients {
    pub fn normalize(self) -> Self {
        Self {
            kjoules: self.kjoules,
            carbohydrates: self.carbohydrates.map(|v| v.normalize().round_dp(2)),
            proteins: self.proteins.map(|v| v.normalize().round_dp(2)),
            fats: self.fats.map(|v| v.normalize().round_dp(2)),
        }
    }
}
