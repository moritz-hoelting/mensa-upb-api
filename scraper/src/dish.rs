use std::fmt::Display;

use itertools::Itertools;
use scraper::ElementRef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dish {
    name: String,
    image_src: Option<String>,
    price_students: Option<String>,
    price_employees: Option<String>,
    price_guests: Option<String>,
    extras: Vec<String>,
    dish_type: DishType,
}

impl Dish {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_price_students(&self) -> Option<&str> {
        self.price_students.as_deref()
    }
    pub fn get_price_employees(&self) -> Option<&str> {
        self.price_employees.as_deref()
    }
    pub fn get_price_guests(&self) -> Option<&str> {
        self.price_guests.as_deref()
    }
    pub fn get_image_src(&self) -> Option<&str> {
        self.image_src.as_deref()
    }
    pub fn is_vegan(&self) -> bool {
        self.extras.contains(&"vegan".to_string())
    }
    pub fn is_vegetarian(&self) -> bool {
        self.extras.contains(&"vegetarian".to_string())
    }
    pub fn get_extras(&self) -> &[String] {
        &self.extras
    }
    pub fn get_type(&self) -> DishType {
        self.dish_type
    }

    pub fn same_as(&self, other: &Self) -> bool {
        self.name == other.name
            && self.price_employees == other.price_employees
            && self.price_guests == other.price_guests
            && self.price_students == other.price_students
            && self.extras.iter().sorted().collect_vec()
                == self.extras.iter().sorted().collect_vec()
    }

    pub fn from_element(element: ElementRef, dish_type: DishType) -> Option<Self> {
        let html_name_selector = scraper::Selector::parse(".desc h4").ok()?;
        let name = element
            .select(&html_name_selector)
            .next()?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let img_selector = scraper::Selector::parse(".img img").ok()?;
        let img_src = element.select(&img_selector).next().and_then(|el| {
            el.value()
                .attr("src")
                .map(|img_src_path| format!("https://www.studierendenwerk-pb.de/{}", img_src_path))
        });

        let html_price_selector = scraper::Selector::parse(".desc .price").ok()?;
        let mut prices = element
            .select(&html_price_selector)
            .filter_map(|price| {
                let price_for = price.first_child().and_then(|strong| {
                    strong.first_child().and_then(|text_element| {
                        text_element
                            .value()
                            .as_text()
                            .map(|text| text.trim().trim_end_matches(':').to_string())
                    })
                });
                let price_value = price.last_child().and_then(|text_element| {
                    text_element
                        .value()
                        .as_text()
                        .map(|text| text.trim().to_string())
                });
                price_for
                    .and_then(|price_for| price_value.map(|price_value| (price_for, price_value)))
            })
            .collect::<Vec<_>>();

        let html_extras_selector = scraper::Selector::parse(".desc .buttons > *").ok()?;
        let extras = element
            .select(&html_extras_selector)
            .filter_map(|extra| extra.value().attr("title").map(|title| title.to_string()))
            .collect::<Vec<_>>();

        Some(Self {
            name,
            image_src: img_src,
            price_students: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Studierende")
                .map(|(_, price)| std::mem::take(price)),
            price_employees: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Bedienstete")
                .map(|(_, price)| std::mem::take(price)),
            price_guests: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Gäste")
                .map(|(_, price)| std::mem::take(price)),
            extras,
            dish_type,
        })
    }
}

impl PartialOrd for Dish {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DishType {
    Main,
    Side,
    Dessert,
}

impl Display for DishType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Main => "main",
            Self::Side => "side",
            Self::Dessert => "dessert",
        };
        f.write_str(s)
    }
}
