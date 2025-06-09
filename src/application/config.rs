use std::net::SocketAddr;

/// Application configuration struct
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub http_addr: SocketAddr,
    pub grpc_addr: SocketAddr,
    pub log_level: String,
    pub enable_http: bool,
    pub enable_grpc: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http_addr: "[::]:8080".parse().unwrap(),
            grpc_addr: "[::]:8081".parse().unwrap(),
            log_level: "info".to_string(),
            enable_http: true,
            enable_grpc: true,
        }
    }
}

impl AppConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("HTTP_ADDR") {
            if let Ok(parsed_addr) = addr.parse() {
                config.http_addr = parsed_addr;
            }
        }

        if let Ok(addr) = std::env::var("GRPC_ADDR") {
            if let Ok(parsed_addr) = addr.parse() {
                config.grpc_addr = parsed_addr;
            }
        }

        if let Ok(level) = std::env::var("LOG_LEVEL") {
            config.log_level = level;
        }

        if let Ok(enable) = std::env::var("ENABLE_HTTP") {
            config.enable_http = enable.parse().unwrap_or(true);
        }

        if let Ok(enable) = std::env::var("ENABLE_GRPC") {
            config.enable_grpc = enable.parse().unwrap_or(true);
        }

        config
    }

    /// Initialize logging
    pub fn init_logging(&self) {
        tracing_subscriber::fmt()
            .with_target(false)
            .with_thread_names(true)
            .init();
    }
}
