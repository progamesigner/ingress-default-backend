use {
    prometheus::{CounterVec, HistogramOpts, HistogramVec, Opts, Registry},
    std::env,
};

#[derive(Clone, Debug)]
pub struct State {
    pub addr: String,
    pub asset: String,
    pub debug: bool,
    pub port: String,
    pub registry: Registry,
    pub request_counter: CounterVec,
    pub request_duration: HistogramVec,
}

mod default {
    pub const ASSET_PATH: &str = "assets";
    pub const DEBUG_MODE: &str = "false";
    pub const LISTEN_ADDR: &str = "127.0.0.1";
    pub const LISTEN_PORT: &str = "3000";
    pub const METRIC_NAMESPACE: &str = "default_backend";
    pub const METRIC_SUBSYSTEM: &str = "http";
}

mod environment {
    pub const ASSET_PATH: &str = "SERVER_ASSET_PATH";
    pub const DEBUG_MODE: &str = "SERVER_DEBUG_MODE";
    pub const LISTEN_ADDR: &str = "SERVER_LISTEN_ADDR";
    pub const LISTEN_PORT: &str = "SERVER_LISTEN_PORT";
    pub const METRIC_NAMESPACE: &str = "SERVER_METRIC_NAMESPACE";
    pub const METRIC_SUBSYSTEM: &str = "SERVER_METRIC_SUBSYSTEM";
}

impl State {
    pub fn new() -> State {
        let registry = Registry::new();

        let namespace =
            env::var(environment::METRIC_NAMESPACE).unwrap_or(default::METRIC_NAMESPACE.into());
        let subsystem =
            env::var(environment::METRIC_SUBSYSTEM).unwrap_or(default::METRIC_SUBSYSTEM.into());

        let request_counter = CounterVec::new(
            Opts::new("request_count_total", "Counter of HTTP requests made.")
                .namespace(&namespace)
                .subsystem(&subsystem),
            &["proto"],
        )
        .unwrap();

        let request_duration = HistogramVec::new(
            HistogramOpts::new(
                "request_duration_milliseconds",
                "Histogram of the time (in milliseconds) each request took.",
            )
            .namespace(&namespace)
            .subsystem(&subsystem)
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

        State {
            addr: env::var(environment::LISTEN_ADDR).unwrap_or(default::LISTEN_ADDR.into()),
            asset: env::var(environment::ASSET_PATH).unwrap_or(default::ASSET_PATH.into()),
            debug: match env::var(environment::DEBUG_MODE)
                .unwrap_or(default::DEBUG_MODE.into())
                .to_lowercase()
                .as_str()
            {
                "1" | "ok" | "okay" | "on" | "true" | "yep" | "yes" => true,
                _ => false,
            },
            port: env::var(environment::LISTEN_PORT).unwrap_or(default::LISTEN_PORT.into()),
            registry,
            request_counter,
            request_duration,
        }
    }
}
