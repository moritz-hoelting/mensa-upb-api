mod dish;
pub mod endpoints;
mod governor;
mod menu;

use std::sync::LazyLock;

pub use dish::{Dish, DishPrices};
pub use governor::get_governor;
pub use menu::Menu;

pub(crate) static USE_X_FORWARDED_HOST: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("API_USE_X_FORWARDED_HOST")
        .map(|val| val == "true")
        .unwrap_or(false)
});
