use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use shared::Canteen;

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
