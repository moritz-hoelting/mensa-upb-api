use actix_web::{get, web::ServiceConfig, HttpResponse, Responder};
use itertools::Itertools as _;
use serde_json::json;
use strum::IntoEnumIterator as _;

use crate::Canteen;

mod menu;

pub fn configure(cfg: &mut ServiceConfig) {
    cfg.service(index);
    cfg.service(menu::menu);
}

#[get("/")]
async fn index() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "version": env!("CARGO_PKG_VERSION"),
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "supportedCanteens": Canteen::iter().map(|c| c.get_identifier().to_string()).collect_vec(),
    }))
}
