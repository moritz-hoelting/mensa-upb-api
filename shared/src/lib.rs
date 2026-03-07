use std::fmt::Display;

mod canteen;
pub use canteen::Canteen;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "dish_type_enum")]
#[sqlx(rename_all = "lowercase")]
pub enum DishType {
    Main,
    Side,
    Soup,
    Dessert,
    Other,
}

impl Display for DishType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Main => "main",
            Self::Side => "side",
            Self::Soup => "soup",
            Self::Dessert => "dessert",
            Self::Other => "other",
        };
        f.write_str(s)
    }
}
