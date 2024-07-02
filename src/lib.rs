mod cache;
mod canteen;
mod dish;
mod menu;

use std::{error::Error, fmt::Display};

pub use cache::MenuCache;
pub use canteen::Canteen;
pub use dish::Dish;
pub use menu::Menu;

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
