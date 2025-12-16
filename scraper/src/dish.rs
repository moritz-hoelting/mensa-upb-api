use std::sync::LazyLock;

use itertools::Itertools;
use num_bigint::BigInt;
use scraper::{ElementRef, Selector};
use shared::DishType;
use sqlx::types::BigDecimal;

static IMG_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".img img").expect("Failed to parse selector"));
static HTML_PRICE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".desc .price").expect("Failed to parse selector"));
static HTML_EXTRAS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".desc .buttons > *").expect("Failed to parse selector"));
static HTML_NUTRITIONS_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse(".nutritions > p").expect("Failed to parse selector"));

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dish {
    name: String,
    image_src: Option<String>,
    price_students: BigDecimal,
    price_employees: BigDecimal,
    price_guests: BigDecimal,
    extras: Vec<String>,
    dish_type: DishType,
    pub nutrition_values: NutritionValues,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NutritionValues {
    pub kjoule: Option<i32>,
    pub protein: Option<BigDecimal>,
    pub carbs: Option<BigDecimal>,
    pub fat: Option<BigDecimal>,
}

impl Dish {
    pub fn get_name(&self) -> &str {
        &self.name
    }
    pub fn get_price_students(&self) -> &BigDecimal {
        &self.price_students
    }
    pub fn get_price_employees(&self) -> &BigDecimal {
        &self.price_employees
    }
    pub fn get_price_guests(&self) -> &BigDecimal {
        &self.price_guests
    }
    pub fn get_image_src(&self) -> Option<&str> {
        self.image_src.as_deref()
    }
    pub fn is_vegan(&self) -> bool {
        self.extras.contains(&"vegan".to_string())
    }
    pub fn is_vegetarian(&self) -> bool {
        self.extras.contains(&"vegetarisch".to_string())
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

    pub fn from_element(
        element: ElementRef,
        details: ElementRef,
        dish_type: DishType,
    ) -> Option<Self> {
        let html_name_selector = Selector::parse(".desc h4").ok()?;
        let name = element
            .select(&html_name_selector)
            .next()?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();

        let img_src = element.select(&IMG_SELECTOR).next().and_then(|el| {
            el.value()
                .attr("src")
                .map(|img_src_path| format!("https://www.studierendenwerk-pb.de/{}", img_src_path))
        });

        let mut prices = element
            .select(&HTML_PRICE_SELECTOR)
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

        let extras = element
            .select(&HTML_EXTRAS_SELECTOR)
            .filter_map(|extra| extra.value().attr("title").map(|title| title.to_string()))
            .collect::<Vec<_>>();

        let nutritions_element = details.select(&HTML_NUTRITIONS_SELECTOR).next();
        let nutrition_values = if let Some(nutritions_element) = nutritions_element {
            let mut kjoule = None;
            let mut protein = None;
            let mut carbs = None;
            let mut fat = None;

            for s in nutritions_element.text() {
                let s = s.trim();
                if !s.is_empty() {
                    if let Some(rest) = s.strip_prefix("Brennwert = ") {
                        kjoule = rest
                            .split_whitespace()
                            .next()
                            .and_then(|num_str| num_str.parse().ok());
                    } else if let Some(rest) = s.strip_prefix("Eiweiß = ") {
                        protein = grams_to_bigdecimal(rest);
                    } else if let Some(rest) = s.strip_prefix("Kohlenhydrate = ") {
                        carbs = grams_to_bigdecimal(rest);
                    } else if let Some(rest) = s.strip_prefix("Fett = ") {
                        fat = grams_to_bigdecimal(rest);
                    }
                }
            }

            NutritionValues {
                kjoule,
                protein,
                carbs,
                fat,
            }
        } else {
            NutritionValues::default()
        };

        Some(Self {
            name,
            image_src: img_src,
            price_students: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Studierende")
                .map(|(_, price)| price_to_bigdecimal(Some(price)))?,
            price_employees: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Bedienstete")
                .map(|(_, price)| price_to_bigdecimal(Some(price)))?,
            price_guests: prices
                .iter_mut()
                .find(|(price_for, _)| price_for == "Gäste")
                .map(|(_, price)| price_to_bigdecimal(Some(price)))?,
            extras,
            dish_type,
            nutrition_values,
        })
    }
}

impl PartialOrd for Dish {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

fn price_to_bigdecimal(s: Option<&str>) -> BigDecimal {
    s.and_then(|p| p.trim_end_matches(" €").replace(',', ".").parse().ok())
        .unwrap_or_else(|| BigDecimal::from_bigint(BigInt::from(99999), 2))
}

fn grams_to_bigdecimal(s: &str) -> Option<BigDecimal> {
    s.trim_end_matches("g")
        .replace(',', ".")
        .trim()
        .parse()
        .ok()
}
