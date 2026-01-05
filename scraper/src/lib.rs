mod canteen;
mod dish;
mod menu;
mod refresh;
pub mod util;

use std::{collections::HashSet, error::Error, fmt::Display, sync::LazyLock};

pub use dish::Dish;
pub use menu::scrape_menu;
pub use refresh::check_refresh;
use shared::Canteen;

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

pub static FILTER_CANTEENS: LazyLock<HashSet<Canteen>> = LazyLock::new(|| {
    std::env::var("FILTER_CANTEENS")
        .ok()
        .map(|s| {
            s.split(',')
                .filter_map(|el| el.parse::<Canteen>().ok())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
});
