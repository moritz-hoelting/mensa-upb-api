use anyhow::Result;
use chrono::NaiveDate;
use shared::{Canteen, DishType};

use crate::{canteen::CanteenExt as _, CustomError, Dish};

#[tracing::instrument]
pub async fn scrape_menu(date: &NaiveDate, canteen: Canteen) -> Result<Vec<Dish>> {
    tracing::debug!("Starting scraping");

    let url = canteen.get_url();
    let client = reqwest::Client::new();
    let request_builder = client.post(url).query(&[(
        "tx_pamensa_mensa[date]",
        date.format("%Y-%m-%d").to_string(),
    )]);
    let response = request_builder.send().await?;
    let html_content = response.text().await?;

    let document = scraper::Html::parse_document(&html_content);

    let html_main_dishes_selector = scraper::Selector::parse(
        "table.table-dishes.main-dishes > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_main_dishes = document.select(&html_main_dishes_selector);
    let main_dishes = html_main_dishes
        .filter_map(|dish| Dish::from_element(dish, DishType::Main))
        .collect::<Vec<_>>();

    let html_side_dishes_selector = scraper::Selector::parse(
        "table.table-dishes.side-dishes > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_side_dishes = document.select(&html_side_dishes_selector);
    let side_dishes = html_side_dishes
        .filter_map(|dish| Dish::from_element(dish, DishType::Side))
        .collect::<Vec<_>>();

    let html_desserts_selector = scraper::Selector::parse(
        "table.table-dishes.soups > tbody > tr.odd > td.description > div.row",
    )
    .map_err(|_| CustomError::from("Failed to parse selector"))?;
    let html_desserts = document.select(&html_desserts_selector);
    let desserts = html_desserts
        .filter_map(|dish| Dish::from_element(dish, DishType::Dessert))
        .collect::<Vec<_>>();

    let mut res = Vec::new();
    res.extend(main_dishes);
    res.extend(side_dishes);
    res.extend(desserts);

    tracing::debug!("Finished scraping");

    Ok(res)
}
