mod canteen;
mod dish;
pub mod endpoints;
mod governor;
mod menu;

use std::{error::Error, fmt::Display, sync::LazyLock};

pub use canteen::Canteen;
pub use dish::{Dish, DishPrices};
pub use governor::get_governor;
pub use menu::Menu;

pub(crate) static USE_X_FORWARDED_HOST: LazyLock<bool> = LazyLock::new(|| {
    std::env::var("API_USE_X_FORWARDED_HOST")
        .map(|val| val == "true")
        .unwrap_or(false)
});

#[derive(Debug, Clone)]
struct CustomError(String);

impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for CustomError {}

impl From<&str> for CustomError {
    fn from(s: &str) -> Self {
        CustomError(s.to_string())
    }
}

impl From<String> for CustomError {
    fn from(s: String) -> Self {
        CustomError(s)
    }
}
