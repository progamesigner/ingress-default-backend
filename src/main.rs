use {
    actix_rt::System,
    actix_web::{
        http::StatusCode, http::Version, middleware::Logger, web, App, Error, HttpRequest,
        HttpResponse, HttpServer,
    },
    env_logger,
    prometheus::{CounterVec, Encoder, HistogramOpts, HistogramVec, Opts, Registry, TextEncoder},
    std::{
        env,
        fs::File,
        io::{BufReader, Read, Result as IOResult},
    },
};

mod default {
    pub const ASSET_PATH: &str = "assets";
    pub const METRIC_NAMESPACE: &str = "default_backend";
    pub const METRIC_SUBSYSTEM: &str = "http";
}

mod environment {
    pub const ASSET_PATH: &str = "SERVER_ASSET_PATH";
    pub const DEBUG_MODE: &str = "SERVER_DEBUG_MODE";
    pub const LISTEN_ADDR: &str = "SERVER_LISTEN_ADDR";
    pub const LISTEN_PORT: &str = "SERVER_LISTEN_PORT";
}

mod header {
    pub const CODE: &str = "X-Code";
    pub const FORMAT: &str = "X-Format";
    pub const INGRESS_NAME: &str = "X-Ingress-Name";
    pub const NAMESPACE: &str = "X-Namespace";
    pub const ORIGINAL_URI: &str = "X-Original-URI";
    pub const REQUEST_ID: &str = "X-Request-Id";
    pub const SERVICE_NAME: &str = "X-Service-Name";
    pub const SERVICE_PORT: &str = "X-Service-Port";
}

#[derive(Clone, Debug)]
struct State {
    registry: Registry,
    request_counter: CounterVec,
    request_duration: HistogramVec,
}

async fn healthz() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

async fn metrics(state: web::Data<State>) -> Result<HttpResponse, Error> {
    let mut buffer = Vec::new();
    let encoder = TextEncoder::new();
    let metrics = state.registry.gather();
    encoder.encode(&metrics, &mut buffer).unwrap();
    Ok(HttpResponse::Ok()
        .content_type(encoder.format_type())
        .body(buffer))
}

async fn statusz() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

async fn handler(request: HttpRequest, state: web::Data<State>) -> Result<HttpResponse, Error> {
    let proto = match request.version() {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        Version::HTTP_3 => "HTTP/3.0",
        _ => "Unknown",
    };

    let _timer = state
        .request_duration
        .with_label_values(&[proto])
        .start_timer();

    let asset = env::var(environment::ASSET_PATH).unwrap_or(default::ASSET_PATH.into());

    let code = match request.headers().get(header::CODE) {
        Some(code) => match code.to_str() {
            Ok(code) => match code.parse() {
                Ok(code) => code,
                Err(_) => 404,
            },
            Err(_) => 404,
        },
        None => 404,
    };

    let format = match request.headers().get(header::FORMAT) {
        Some(format) => match format.to_str() {
            Ok(format) => match format {
                "application/json" => "application/json",
                _ => "text/html",
            },
            Err(_) => "text/html",
        },
        None => "text/html",
    };

    let extension = match format {
        "application/json" => "json",
        _ => "html",
    };

    let status = match StatusCode::from_u16(code) {
        Ok(status) => status,
        Err(_) => StatusCode::NOT_FOUND,
    };

    let debug = match env::var(environment::DEBUG_MODE)
        .unwrap_or("".into())
        .to_lowercase()
        .as_str()
    {
        "1" | "ok" | "okay" | "on" | "true" | "yep" | "yes" => true,
        _ => false,
    };

    let file = match File::open(format!("{}/{}.{}", asset, code, extension)) {
        Ok(file) => Some(file),
        Err(_) => match File::open(format!("{}/{}x.{}", asset, code / 10, extension)) {
            Ok(file) => Some(file),
            Err(_) => match File::open(format!("{}/{}xx.{}", asset, code / 100, extension)) {
                Ok(file) => Some(file),
                Err(_) => match File::open(format!("{}/index.{}", asset, extension)) {
                    Ok(file) => Some(file),
                    Err(_) => match File::open(format!("{}/index.html", asset)) {
                        Ok(file) => Some(file),
                        Err(_) => None,
                    },
                },
            },
        },
    };

    let mut response = HttpResponse::build(status);

    response.content_type(format);

    if debug {
        response
            .if_some(request.headers().get(header::CODE), |header, response| {
                response.header(header::CODE, header.as_bytes());
            })
            .if_some(request.headers().get(header::FORMAT), |header, response| {
                response.header(header::FORMAT, header.as_bytes());
            })
            .if_some(
                request.headers().get(header::ORIGINAL_URI),
                |header, response| {
                    response.header(header::ORIGINAL_URI, header.as_bytes());
                },
            )
            .if_some(
                request.headers().get(header::NAMESPACE),
                |header, response| {
                    response.header(header::NAMESPACE, header.as_bytes());
                },
            )
            .if_some(
                request.headers().get(header::INGRESS_NAME),
                |header, response| {
                    response.header(header::INGRESS_NAME, header.as_bytes());
                },
            )
            .if_some(
                request.headers().get(header::SERVICE_NAME),
                |header, response| {
                    response.header(header::SERVICE_NAME, header.as_bytes());
                },
            )
            .if_some(
                request.headers().get(header::SERVICE_PORT),
                |header, response| {
                    response.header(header::SERVICE_PORT, header.as_bytes());
                },
            )
            .if_some(
                request.headers().get(header::REQUEST_ID),
                |header, response| {
                    response.header(header::REQUEST_ID, header.as_bytes());
                },
            );
    }

    state.request_counter.with_label_values(&[proto]).inc();

    match file {
        Some(file) => {
            let mut body = String::new();
            let mut reader = BufReader::new(file);
            match reader.read_to_string(&mut body) {
                Ok(_) => Ok(response.body(body)),
                Err(_) => Ok(response.status(StatusCode::NOT_FOUND).finish()),
            }
        }
        None => Ok(response.status(StatusCode::NOT_FOUND).finish()),
    }
}

fn initialize_app_state() -> web::Data<State> {
    let registry = Registry::new();

    let request_counter = CounterVec::new(
        Opts::new("request_count_total", "Counter of HTTP requests made.")
            .namespace(default::METRIC_NAMESPACE)
            .subsystem(default::METRIC_SUBSYSTEM),
        &["proto"],
    )
    .unwrap();

    let request_duration = HistogramVec::new(
        HistogramOpts::new(
            "request_duration_milliseconds",
            "Histogram of the time (in milliseconds) each request took.",
        )
        .namespace(default::METRIC_NAMESPACE)
        .subsystem(default::METRIC_SUBSYSTEM)
        .buckets(vec![
            0.001, 0.003, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ]),
        &["proto"],
    )
    .unwrap();

    registry
        .register(Box::new(request_counter.clone()))
        .unwrap();
    registry
        .register(Box::new(request_duration.clone()))
        .unwrap();

    web::Data::new(State {
        registry,
        request_counter,
        request_duration,
    })
}

fn main() -> IOResult<()> {
    let addr = env::var(environment::LISTEN_ADDR).unwrap_or("127.0.0.1".into());
    let port = env::var(environment::LISTEN_PORT).unwrap_or("3000".into());

    env_logger::init();

    let data = initialize_app_state();

    System::new("server").block_on(async move {
        HttpServer::new(move || {
            App::new()
                .app_data(data.clone())
                .route("/healthz", web::to(healthz))
                .route("/metrics", web::to(metrics))
                .route("/statusz", web::to(statusz))
                .route("*", web::to(handler))
                .wrap(Logger::default())
        })
        .bind(format!("{}:{}", addr, port))?
        .run()
        .await
    })
}
