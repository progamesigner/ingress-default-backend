use {
    std::{env, net::SocketAddr},
};

#[derive(Debug)]
pub struct Env {}

mod default {
    pub const ASSET_PATH: &str = "assets";
    pub const DEBUG_MODE: &str = "false";
    pub const LISTEN_ADDR: &str = "127.0.0.1";
    pub const LISTEN_PORT: &str = "3000";
    pub const METRIC_LISTEN_PORT: &str = "9402";
    pub const METRIC_NAMESPACE: &str = "default_backend";
    pub const METRIC_SUBSYSTEM: &str = "http";
}

mod environment {
    pub const ASSET_PATH: &str = "SERVER_ASSET_PATH";
    pub const DEBUG_MODE: &str = "SERVER_DEBUG_MODE";
    pub const LISTEN_ADDR: &str = "SERVER_LISTEN_ADDR";
    pub const LISTEN_PORT: &str = "SERVER_LISTEN_PORT";
    pub const METRIC_LISTEN_PORT: &str = "SERVER_METRIC_LISTEN_PORT";
    pub const METRIC_NAMESPACE: &str = "SERVER_METRIC_NAMESPACE";
    pub const METRIC_SUBSYSTEM: &str = "SERVER_METRIC_SUBSYSTEM";
}

impl Env {
    pub fn is_debug_mode() -> bool {
        match env::var(environment::DEBUG_MODE)
            .unwrap_or(default::DEBUG_MODE.into())
            .to_lowercase()
            .as_str()
        {
            "1" | "ok" | "okay" | "on" | "true" | "yep" | "yes" => true,
            _ => false,
        }
    }

    pub fn parse_asset_path() -> String {
        env::var(environment::ASSET_PATH).unwrap_or(default::ASSET_PATH.into())
    }

    pub fn parse_metric_address() -> SocketAddr {
        let addr = match env::var(environment::LISTEN_ADDR) {
            Ok(addr) => addr,
            Err(_) => default::LISTEN_ADDR.into(),
        };
        let port = match env::var(environment::METRIC_LISTEN_PORT) {
            Ok(port) => port,
            Err(_) => default::METRIC_LISTEN_PORT.into(),
        };
        format!("{}:{}", addr, port).parse().unwrap()
    }

    pub fn parse_metric_namespace() -> String {
        env::var(environment::METRIC_NAMESPACE).unwrap_or(default::METRIC_NAMESPACE.into())
    }

    pub fn parse_metric_subsystem() -> String {
        env::var(environment::METRIC_SUBSYSTEM).unwrap_or(default::METRIC_SUBSYSTEM.into())
    }

    pub fn parse_service_address() -> SocketAddr {
        let addr = match env::var(environment::LISTEN_ADDR) {
            Ok(addr) => addr,
            Err(_) => default::LISTEN_ADDR.into(),
        };
        let port = match env::var(environment::LISTEN_PORT) {
            Ok(port) => port,
            Err(_) => default::LISTEN_PORT.into(),
        };
        format!("{}:{}", addr, port).parse().unwrap()
    }
}
