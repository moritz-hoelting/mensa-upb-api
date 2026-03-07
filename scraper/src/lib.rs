mod canteen;
mod dish;
mod menu;
mod refresh;
pub mod util;

use std::{collections::HashSet, sync::LazyLock};

pub use dish::Dish;
pub use menu::scrape_menu;
pub use refresh::check_refresh;
use shared::Canteen;

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
