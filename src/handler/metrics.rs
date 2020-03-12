use {
    crate::state::MetricState,
    actix_web::{web::Data, Error, HttpResponse},
    prometheus::{Encoder, TextEncoder},
};

pub async fn handler(state: Data<MetricState>) -> Result<HttpResponse, Error> {
    let mut buffer = Vec::new();

    let encoder = TextEncoder::new();
    let metrics = state.gather();

    match encoder.encode(&metrics, &mut buffer) {
        Ok(()) => Ok(HttpResponse::Ok()
            .content_type(encoder.format_type())
            .body(buffer)),
        Err(_) => Ok(HttpResponse::InternalServerError().finish()),
    }
}
