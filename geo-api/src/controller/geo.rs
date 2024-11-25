use crate::exception::ApiError;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use maxminddb::{geoip2, Reader};
use serde::Serialize;
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;

#[derive(Serialize)]
struct IpInfo {
    ip: String,
    country: Option<String>,
    city: Option<String>,
    latitude: Option<f64>,
    longitude: Option<f64>,
}

#[get("/ip/{ip}")]
async fn get_ip_info(
    path: web::Path<String>,
    reader: web::Data<Arc<Reader<Vec<u8>>>>,
) -> impl Responder {
    let ip = path.into_inner();
    let ip_addr = match IpAddr::from_str(&ip) {
        Ok(addr) => addr,
        Err(_) => {
            return web::Json(IpInfo {
                ip,
                country: None,
                city: None,
                latitude: None,
                longitude: None,
            })
        }
    };

    let city: geoip2::City = match reader.lookup(ip_addr) {
        Ok(city) => city,
        Err(_) => {
            return web::Json(IpInfo {
                ip: ip_addr.to_string(),
                country: None,
                city: None,
                latitude: None,
                longitude: None,
            })
        }
    };

    web::Json(IpInfo {
        ip: ip_addr.to_string(),
        country: city
            .country
            .and_then(|f| f.names)
            .and_then(|m| m.get("zh-CN").cloned())
            .and_then(|s| String::from_str(s).ok()),
        city: city
            .city
            .and_then(|f| f.names)
            .and_then(|m| m.get("zh-CN").cloned())
            .and_then(|s| String::from_str(s).ok()),
        latitude: city.location.clone().and_then(|f| f.latitude),
        longitude: city.location.and_then(|f| f.longitude),
    })
}

pub fn register() -> Scope {
    web::scope("").service(get_ip_info)
}
