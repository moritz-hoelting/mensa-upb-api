use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};
use itertools::Itertools as _;
use serde_json::json;
use shared::Canteen;
use strum::IntoEnumIterator as _;

mod menu;
mod nutrition;
mod price_history;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(index)
        .configure(menu::configure)
        .configure(nutrition::configure)
        .configure(price_history::configure);
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "supportedCanteens": Canteen::iter().map(|c| c.get_identifier().to_string()).collect_vec(),
    }))
}
