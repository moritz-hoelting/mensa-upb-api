use std::sync::LazyLock;

use anyhow::Result;
use chrono::NaiveDate;
use scraper::{Html, Selector};
use shared::{Canteen, DishType};

use crate::{canteen::CanteenExt as _, CustomError, Dish};

static HTML_MAIN_DISHES_TBODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table.table-dishes.main-dishes > tbody").expect("Failed to parse selector")
});
static HTML_SIDE_DISHES_TBODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table.table-dishes.side-dishes > tbody").expect("Failed to parse selector")
});
static HTML_DESSERTS_TBODY_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("table.table-dishes.soups > tbody").expect("Failed to parse selector")
});

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

    let main_dishes = scrape_category(&document, &HTML_MAIN_DISHES_TBODY_SELECTOR, DishType::Main)?;
    let side_dishes = scrape_category(&document, &HTML_SIDE_DISHES_TBODY_SELECTOR, DishType::Side)?;
    let desserts = scrape_category(&document, &HTML_DESSERTS_TBODY_SELECTOR, DishType::Dessert)?;

    let mut res = Vec::new();
    res.extend(main_dishes);
    res.extend(side_dishes);
    res.extend(desserts);

    dbg!(&res);

    tracing::debug!("Finished scraping");

    Ok(res)
}

static ITEM_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("tr.odd > td.description > div.row").expect("Failed to parse selector")
});
static ITEM_DETAILS_SELECTOR: LazyLock<Selector> = LazyLock::new(|| {
    Selector::parse("tr.even > td.more > div.ingredients-list").expect("Failed to parse selector")
});

fn scrape_category<'a>(
    document: &'a Html,
    tbody_selector: &Selector,
    dish_type: DishType,
) -> Result<impl Iterator<Item = Dish> + 'a> {
    let tbody = document.select(tbody_selector).next().ok_or_else(|| {
        CustomError::from(format!("No tbody found for selector: {:?}", tbody_selector))
    })?;
    let dishes = tbody.select(&ITEM_SELECTOR);
    let dish_details = tbody.select(&ITEM_DETAILS_SELECTOR);

    Ok(dishes
        .zip(dish_details)
        .filter_map(move |(dish, details)| Dish::from_element(dish, details, dish_type)))
}
