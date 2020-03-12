use {
    crate::{
        env::Env,
        handler::{handler, healthz, metrics, statusz},
        state::State,
    },
    actix_web::{
        middleware::Logger,
        web::{to, Data},
        App, HttpServer,
    },
    futures::future::{join, FutureExt},
    std::io::Result,
};

pub async fn start() -> Result<()> {
    let (service_state, metric_state) = State::new();

    let (service_data, metric_data) = (Data::new(service_state), Data::new(metric_state));

    let metric_server = HttpServer::new(move || {
        App::new()
            .app_data(metric_data.clone())
            .route("/healthz", to(healthz))
            .route("/metrics", to(metrics))
            .route("/statusz", to(statusz))
    })
    .bind(Env::parse_metric_address())?
    .run();

    let service_server = HttpServer::new(move || {
        App::new()
            .app_data(service_data.clone())
            .route("*", to(handler))
            .wrap(Logger::default())
    })
    .bind(Env::parse_service_address())?
    .run();

    join(metric_server, service_server)
        .map(|(_, _)| Ok(()))
        .await
}
