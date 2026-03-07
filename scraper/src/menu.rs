use anyhow::Result;
use chrono::NaiveDate;
use shared::Canteen;

use crate::{Dish, canteen::CanteenExt as _};

const API_URL: &str = "https://stwpb.de/wp-json/stwk-pb/v1/meals";

#[tracing::instrument]
pub async fn scrape_menu(
    start_date: &NaiveDate,
    end_date: &NaiveDate,
    canteen: Canteen,
) -> Result<Vec<Dish>> {
    tracing::debug!("Starting scraping");

    let client = reqwest::Client::new();
    let request_builder = client.get(API_URL).query(&[
        ("venue", canteen.get_venue_id().to_string()),
        ("start_date", start_date.format("%Y-%m-%d").to_string()),
        ("end_date", end_date.format("%Y-%m-%d").to_string()),
    ]);
    let response = request_builder.send().await?;
    let response_data = response.json::<ResponseData>().await?;

    let res = response_data.meals.into_iter().map(Dish::from).collect();

    tracing::debug!("Finished scraping");

    Ok(res)
}

#[expect(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
struct ResponseData {
    venue: String,
    venue_name: String,
    start_date: NaiveDate,
    end_date: NaiveDate,
    meals: Vec<ResponseMeal>,
    total: usize,
}

#[expect(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ResponseMeal {
    pub id: usize,
    pub title: String,
    pub date: NaiveDate,
    pub date_german: String,
    pub category: String,
    pub price_students: String,
    pub price_staff: String,
    pub price_guests: String,
    pub allergens_raw: String,
    pub allergens_decoded: ResponseAllergensDecoded,
    pub nutrition: String,
    pub button: String,
    pub image_jpeg: String,
    pub image_webp: String,
    pub image_jpeg_small: String,
    pub image_webp_small: String,
    pub image_jpeg_thumb: String,
    pub image_webp_thumb: String,
}

#[expect(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ResponseAllergensDecoded {
    pub allergens: Vec<ResponseAllergen>,
    pub additives: Vec<ResponseAdditive>,
    pub raw_codes: Vec<String>,
}

#[expect(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ResponseAllergen {
    pub id: String,
    pub code: String,
    pub name_de: String,
    pub name_en: String,
    pub category: String,
    pub active: String,
    pub sort_order: String,
}

#[expect(dead_code)]
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct ResponseAdditive {
    pub id: String,
    pub code: String,
    pub name_de: String,
    pub name_en: String,
    pub active: String,
    pub sort_order: String,
}
