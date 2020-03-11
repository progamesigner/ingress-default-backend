use {
    crate::{
        handler::{handler, healthz, metrics, statusz},
        state::State,
    },
    actix_web::{
        middleware::Logger,
        web::{to, Data},
        App, HttpServer,
    },
    std::io::Result,
};

pub async fn start() -> Result<()> {
    let state = Data::new(State::new());

    let listen = format!("{}:{}", state.addr, state.port);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .route("/healthz", to(healthz))
            .route("/metrics", to(metrics))
            .route("/statusz", to(statusz))
            .route("*", to(handler))
            .wrap(Logger::default())
    })
    .bind(listen)?
    .run()
    .await
}
