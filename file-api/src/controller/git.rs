use std::{fs, path::Path};

use crate::exception::ApiError;
use actix_web::{get, web, HttpResponse, Responder, Scope};
use log::info;

#[get("/file/{file:.*}")]
async fn get_file(path: web::Path<String>) -> Result<impl Responder, ApiError> {
    let file_path = Path::new("").join(path.into_inner());

    info!("file request for {:?}", file_path);

    if !file_path.exists() {
        return Err(ApiError::NotFoundFile(file_path.to_string_lossy().to_string()));
    }

    let content = fs::read_to_string(&file_path)
        .map_err(move |_e| ApiError::NotFoundFile(file_path.to_string_lossy().to_string()))?;

    Ok(HttpResponse::Ok().content_type("text/plain").body(content))
}
pub fn register() -> Scope {
    web::scope("").service(get_file)
}
