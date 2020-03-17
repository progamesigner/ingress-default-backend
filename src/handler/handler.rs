use {
    crate::{env::Env, state::ServiceState},
    actix_web::{http::StatusCode, http::Version, web::Data, Error, HttpRequest, HttpResponse},
    handlebars::{Context, Handlebars, Helper, HelperResult, JsonRender, Output, RenderContext},
    serde_json::value::Map,
    std::{
        fs::File,
        io::{BufReader, Read},
    },
};

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

fn handlebar_helper_env(
    helper: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    output: &mut dyn Output,
) -> HelperResult {
    match helper.param(0) {
        Some(name) => {
            let name = format!("{}", name.value().render());
            output.write(Env::parse(&name).unwrap_or("".into()).as_ref())?;
            Ok(())
        }
        None => {
            for (key, value) in Env::vars() {
                let line = format!("{}={}\n", key, value);
                output.write(line.as_ref())?;
            }
            Ok(())
        }
    }
}

pub async fn handler(
    request: HttpRequest,
    state: Data<ServiceState>,
) -> Result<HttpResponse, Error> {
    let proto = match request.version() {
        Version::HTTP_09 => "HTTP/0.9",
        Version::HTTP_10 => "HTTP/1.0",
        Version::HTTP_11 => "HTTP/1.1",
        Version::HTTP_2 => "HTTP/2.0",
        Version::HTTP_3 => "HTTP/3.0",
        _ => "Unknown",
    };

    let _ = state.start_timer(proto);

    let asset = Env::parse_asset_path();

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

    let data = Map::new();

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

    let (format, file) = match File::open(format!("{}/{}.{}", asset, code, extension)) {
        Ok(file) => (format, Some(file)),
        Err(_) => match File::open(format!("{}/{}x.{}", asset, code / 10, extension)) {
            Ok(file) => (format, Some(file)),
            Err(_) => match File::open(format!("{}/{}xx.{}", asset, code / 100, extension)) {
                Ok(file) => (format, Some(file)),
                Err(_) => match File::open(format!("{}/index.{}", asset, extension)) {
                    Ok(file) => (format, Some(file)),
                    Err(_) => match File::open(format!("{}/index.html", asset)) {
                        Ok(file) => ("text/html", Some(file)),
                        Err(_) => ("text/html", None),
                    },
                },
            },
        },
    };

    let mut response = HttpResponse::build(status);
    let mut template = Handlebars::new();

    response.content_type(format);
    template.register_helper("env", Box::new(handlebar_helper_env));

    if Env::is_debug_mode() {
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

    state.increase_request_counter(proto);

    if let Some(file) = file {
        let mut body = String::new();
        let mut reader = BufReader::new(file);
        if let Ok(_) = reader.read_to_string(&mut body) {
            if let Ok(body) = template.render_template(body.as_str(), &data) {
                return Ok(response.body(body));
            }
        }
    }
    Ok(response.status(StatusCode::NOT_FOUND).finish())
}
