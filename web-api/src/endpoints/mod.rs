use actix_web::{get, HttpResponse, Responder};
use shared::Canteen;
use strum::IntoEnumIterator as _;
use utoipa_actix_web::service_config::ServiceConfig;

mod menu;
mod metadata;
mod nutrition;
mod price_history;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(index)
        .configure(metadata::configure)
        .configure(menu::configure)
        .configure(nutrition::configure)
        .configure(price_history::configure);
}

#[derive(serde::Serialize, utoipa::ToSchema)]
struct IndexResponse {
    /// The current version of the API.
    version: &'static str,
    /// A short description of the API.
    description: &'static str,
    /// A list of supported canteens.
    supported_canteens: Vec<String>,
}

#[utoipa::path(summary = "Get API version and capabilities", description = "Get information about the api version and capabilities.", responses((status = 200, body = IndexResponse, example = json!(IndexResponse {
    version: env!("CARGO_PKG_VERSION"),
    description: env!("CARGO_PKG_DESCRIPTION"),
    supported_canteens: Canteen::iter().map(|c| c.get_identifier().to_string()).collect::<Vec<String>>()
}))))]
#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(IndexResponse {
        version: env!("CARGO_PKG_VERSION"),
        description: env!("CARGO_PKG_DESCRIPTION"),
        supported_canteens: Canteen::iter()
            .map(|c| c.get_identifier().to_string())
            .collect(),
    })
}
