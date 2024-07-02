use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{Canteen, CustomError, Dish};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Menu {
    main_dishes: Vec<Dish>,
    side_dishes: Vec<Dish>,
    desserts: Vec<Dish>,
}

impl Menu {
    pub async fn new(day: NaiveDate, canteen: Canteen) -> Result<Self> {
        scrape_menu(canteen, day).await
    }

    pub fn get_main_dishes(&self) -> &[Dish] {
        &self.main_dishes
    }

    pub fn get_side_dishes(&self) -> &[Dish] {
        &self.side_dishes
    }

    pub fn get_desserts(&self) -> &[Dish] {
        &self.desserts
    }

    pub fn merged(self, other: Self) -> Self {
        let mut main_dishes = self.main_dishes;
        let mut side_dishes = self.side_dishes;
        let mut desserts = self.desserts;

        for dish in other.main_dishes {
            if let Some(existing) = main_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                main_dishes.push(dish);
            }
        }
        for dish in other.side_dishes {
            if let Some(existing) = side_dishes.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                side_dishes.push(dish);
            }
        }
        for dish in other.desserts {
            if let Some(existing) = desserts.iter_mut().find(|d| dish.same_as(d)) {
                existing.merge(dish);
            } else {
                desserts.push(dish);
            }
        }

        main_dishes.sort_by(|a, b| a.get_name().cmp(b.get_name()));
        side_dishes.sort_by(|a, b| a.get_name().cmp(b.get_name()));
        desserts.sort_by(|a, b| a.get_name().cmp(b.get_name()));

        Self {
            main_dishes,
            side_dishes,
            desserts,
        }
    }
}

async fn scrape_menu(canteen: Canteen, day: NaiveDate) -> Result<Menu> {
    let url = canteen.get_url();
    let client = reqwest::Client::new();
    let request_builder = client
        .post(url)
        .query(&[("tx_pamensa_mensa[date]", day.format("%Y-%m-%d").to_string())]);
    let response = request_builder.send().await?;
    let html_content = response.text().await?;

    let document = scraper::Html::parse_document(&html_content);

    let html_main_dishes_selector = scraper::Selector::parse(
        "table.table-dishes.main-dishes > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_main_dishes = document.select(&html_main_dishes_selector);
    let main_dishes = html_main_dishes
        .filter_map(|dish| Dish::from_element(dish, canteen))
        .collect::<Vec<_>>();

    let html_side_dishes_selector = scraper::Selector::parse(
        "table.table-dishes.side-dishes > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_side_dishes = document.select(&html_side_dishes_selector);
    let side_dishes = html_side_dishes
        .filter_map(|dish| Dish::from_element(dish, canteen))
        .collect::<Vec<_>>();

    let html_desserts_selector = scraper::Selector::parse(
        "table.table-dishes.soups > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_desserts = document.select(&html_desserts_selector);
    let desserts = html_desserts
        .filter_map(|dish| Dish::from_element(dish, canteen))
        .collect::<Vec<_>>();

    Ok(Menu {
        main_dishes,
        side_dishes,
        desserts,
    })
}
