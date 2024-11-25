use actix_cors::Cors;
use actix_web::{get, web, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde_json::json;
use url::Url;

use crate::auth::{enable_auth, validator};

pub(crate) mod geo;

#[get("/")]
async fn welcome() -> impl Responder {
    let version = env!("CARGO_PKG_VERSION");
    json!({"message":"Welcome to the Geo API", "version": version})
        .to_string()
        .customize()
        .insert_header(("Content-Type", "application/json"))
}

pub fn config_controller(cfg: &mut web::ServiceConfig) {
    let cors = Cors::default()
        .allowed_origin_fn(|origin, _| {
            if origin.as_bytes().is_empty() {
                return true;
            }
            if let Ok(url) = Url::parse(origin.to_str().unwrap()) {
                if let Some(host) = url.host_str() {
                    return host.ends_with(".mazhangjing.com");
                }
            }
            false
        })
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);
    if enable_auth() {
        let auth = HttpAuthentication::with_fn(validator);
        cfg.service(welcome)
            .service(geo::register().wrap(auth.clone()).wrap(cors));
    } else {
        cfg.service(welcome).service(geo::register().wrap(cors));
    }
}
