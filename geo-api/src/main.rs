mod auth;
mod config;
mod controller;
mod db;
mod exception;
#[macro_use]
mod dev;

use std::sync::Arc;

use crate::controller::config_controller;
use actix_web::{error, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use env_logger::Env;

use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("Geo Api version: {}", env!("CARGO_PKG_VERSION"));

    let reader = maxminddb::Reader::open_readfile("GeoLite2-City.mmdb").unwrap();
    let reader_arc = Arc::new(reader);

    let json_config = web::JsonConfig::default()
        .limit(1024)
        .error_handler(|err, _req| {
            error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
        });
        
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(reader_arc.clone()))
            .app_data(json_config.to_owned())
            .configure(config_controller)
    })
    .workers(1)
    .bind("0.0.0.0:8080")?
    .run()
    .await
}
