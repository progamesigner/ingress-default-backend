use {
    crate::state::State,
    actix_web::{web::Data, Error, HttpResponse},
    prometheus::{Encoder, TextEncoder},
};

pub async fn handler(state: Data<State>) -> Result<HttpResponse, Error> {
    let mut buffer = Vec::new();

    let encoder = TextEncoder::new();
    let metrics = state.registry.gather();

    match encoder.encode(&metrics, &mut buffer) {
        Ok(()) => Ok(HttpResponse::Ok()
            .content_type(encoder.format_type())
            .body(buffer)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
