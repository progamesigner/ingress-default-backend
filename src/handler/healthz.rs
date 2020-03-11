use actix_web::{Error, HttpResponse};

pub async fn handler() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}
