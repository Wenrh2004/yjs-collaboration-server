use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

/// Application configuration for the Yjs collaboration server.
///
/// This struct holds all configurable settings for the application, including
/// network addresses, logging options, and service enablement flags.
/// Configuration can be loaded from YAML files or environment variables.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// HTTP server address in format "[host]:port"
    pub http_addr: String,
    /// gRPC server address in format "[host]:port"
    pub grpc_addr: String,
    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
    /// Flag controlling whether HTTP server is enabled
    pub enable_http: bool,
    /// Flag controlling whether gRPC server is enabled
    pub enable_grpc: bool,
}

impl Default for AppConfig {
    /// Creates a default configuration with sensible defaults.
    ///
    /// # Default values
    ///
    /// - HTTP server: :8080
    /// - gRPC server: :8081
    /// - Log level: "info"
    /// - Both HTTP and gRPC servers enabled
    ///
    /// # Returns
    ///
    /// A new AppConfig instance with default settings
    fn default() -> Self {
        Self {
            http_addr: "[::]:8080".to_string(),
            grpc_addr: "[::]:8081".to_string(),
            log_level: "info".to_string(),
            enable_http: true,
            enable_grpc: true,
        }
    }
}

impl AppConfig {
    /// Loads configuration from a YAML file.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(AppConfig)` - Successfully loaded configuration
    /// * `Err(String)` - Error message if loading fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// * The file cannot be read
    /// * The file content cannot be parsed as valid YAML
    /// * The YAML structure doesn't match AppConfig
    pub fn from_yaml<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        match fs::read_to_string(path) {
            Ok(content) => match serde_yaml::from_str(&content) {
                Ok(config) => Ok(config),
                Err(e) => Err(format!("Failed to parse YAML: {}", e)),
            },
            Err(e) => Err(format!("Failed to read configuration file: {}", e)),
        }
    }

    /// Creates configuration from environment variables.
    ///
    /// Environment variables:
    /// * HTTP_ADDR - HTTP server address
    /// * GRPC_ADDR - gRPC server address
    /// * LOG_LEVEL - Logging level
    /// * ENABLE_HTTP - HTTP server enablement (true/false)
    /// * ENABLE_GRPC - gRPC server enablement (true/false)
    ///
    /// If an environment variable is not set, the default value is used.
    ///
    /// # Returns
    ///
    /// A new AppConfig instance with values from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(addr) = std::env::var("HTTP_ADDR") {
            config.http_addr = addr;
        }

        if let Ok(addr) = std::env::var("GRPC_ADDR") {
            config.grpc_addr = addr;
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

    /// Parses the HTTP address string into a SocketAddr.
    ///
    /// # Returns
    ///
    /// A socket address for the HTTP server, falling back to :8080 on parsing failure
    pub fn http_socket_addr(&self) -> SocketAddr {
        self.http_addr.parse().unwrap_or_else(|_| "[::]:8080".parse().unwrap())
    }

    /// Parses the gRPC address string into a SocketAddr.
    ///
    /// # Returns
    ///
    /// A socket address for the gRPC server, falling back to :8081 on parsing failure
    pub fn grpc_socket_addr(&self) -> SocketAddr {
        self.grpc_addr.parse().unwrap_or_else(|_| "[::]:8081".parse().unwrap())
    }

    /// Checks if a configuration file exists at the specified path.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to check
    ///
    /// # Returns
    ///
    /// `true` if the file exists, `false` otherwise
    pub fn config_exists<P: AsRef<Path>>(path: P) -> bool {
        Path::new(path.as_ref()).exists()
    }

    /// Saves the current configuration to a YAML file.
    ///
    /// # Parameters
    ///
    /// * `path` - Path where to save the configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Success
    /// * `Err(String)` - Error message if saving fails
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// * The configuration cannot be serialized to YAML
    /// * The file cannot be written
    pub fn save_to_yaml<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let yaml = match serde_yaml::to_string(self) {
            Ok(y) => y,
            Err(e) => return Err(format!("Failed to serialize configuration: {}", e)),
        };

        match fs::write(path, yaml) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to write configuration file: {}", e)),
        }
    }

    /// Initializes the logging system using the configured log level.
    ///
    /// Sets up tracing with the appropriate log level, disables targets,
    /// and enables thread names for better debugging.
    pub fn init_logging(&self) {
        tracing_subscriber::fmt()
            .with_max_level(match self.log_level.as_str() {
                "trace" => tracing::Level::TRACE,
                "debug" => tracing::Level::DEBUG,
                "info" => tracing::Level::INFO,
                "warn" => tracing::Level::WARN,
                "error" => tracing::Level::ERROR,
                _ => tracing::Level::INFO,
            })
            .with_target(false)
            .with_thread_names(true)
            .init();
    }
}
