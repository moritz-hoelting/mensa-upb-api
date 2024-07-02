use itertools::Itertools;
use scraper::ElementRef;
use serde::{Deserialize, Serialize};

use crate::Canteen;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dish {
    name: String,
    image_src: String,
    price_students: Option<String>,
    price_employees: Option<String>,
    price_guests: Option<String>,
    extras: Vec<String>,
    #[serde(skip)]
    canteens: Vec<Canteen>,
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
    pub fn get_extras(&self) -> &[String] {
        &self.extras
    }
    pub fn get_canteens(&self) -> &[Canteen] {
        &self.canteens
    }

    pub fn same_as(&self, other: &Self) -> bool {
        self.name == other.name
            && self.price_employees == other.price_employees
            && self.price_guests == other.price_guests
            && self.price_students == other.price_students
            && self.extras.iter().sorted().collect_vec()
                == self.extras.iter().sorted().collect_vec()
    }

    pub fn merge(&mut self, other: Self) {
        self.canteens.extend(other.canteens);
        self.canteens.sort();
        self.canteens.dedup();
    }
}

impl Dish {
    pub fn from_element(element: ElementRef, canteen: Canteen) -> Option<Self> {
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
        let img_src_path = element.select(&img_selector).next()?.value().attr("src")?;
        let img_src = format!("https://www.studierendenwerk-pb.de/{}", img_src_path);

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
            canteens: vec![canteen],
        })
    }
}

impl PartialOrd for Dish {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}
