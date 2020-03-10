use {
    actix_rt::System,
    actix_web::{
        http::StatusCode, middleware::Logger, web, App, Error, HttpRequest, HttpResponse,
        HttpServer,
    },
    env_logger,
    std::{
        env,
        fs::File,
        io::{BufReader, Read, Result as IOResult},
    },
};

mod default {
    pub const ASSET_PATH: &str = "assets";
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

async fn healthz() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

async fn statusz() -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
}

async fn handler(request: HttpRequest) -> Result<HttpResponse, Error> {
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

fn main() -> IOResult<()> {
    let addr = env::var(environment::LISTEN_ADDR).unwrap_or("127.0.0.1".into());
    let port = env::var(environment::LISTEN_PORT).unwrap_or("3000".into());

    env_logger::init();

    System::new("server").block_on(async move {
        HttpServer::new(move || {
            App::new()
                .route("/healthz", web::to(healthz))
                .route("/statusz", web::to(statusz))
                .route("*", web::to(handler))
                .wrap(Logger::default())
        })
        .bind(format!("{}:{}", addr, port))?
        .run()
        .await
    })
}
