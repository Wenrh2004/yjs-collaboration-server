use std::path::Path;

use tokio::try_join;
use tracing::{info, warn};

use crate::{
    config::AppConfig,
    container::Container,
    servers::{HttpServer, RpcServer},
};

/// Default configuration file path for the application
const DEFAULT_CONFIG_PATH: &str = "./config/bootstrap.yaml";

/// Application bootstrap service responsible for initializing and starting the application.
///
/// This service coordinates the overall application startup process including:
/// - Loading configuration from YAML files or environment variables
/// - Initializing the logging system
/// - Setting up dependency containers
/// - Starting appropriate server instances based on configuration
pub struct ApplicationBootstrap {
    /// Application configuration including server addresses and feature flags
    config: AppConfig,
    /// Dependency injection container for services and repositories
    container: Container,
}

impl ApplicationBootstrap {
    /// Creates a new application bootstrap instance with loaded configuration.
    ///
    /// This performs the following steps:
    /// 1. Loads configuration from YAML file or environment variables
    /// 2. Initializes the logging system based on configuration
    /// 3. Sets up the dependency injection container
    ///
    /// # Returns
    ///
    /// A new `ApplicationBootstrap` instance ready for running the application
    pub fn new() -> Self {
        // Try loading configuration from a yaml file
        let config = Self::load_config();
        config.init_logging();

        let container = Container::new();

        Self { config, container }
    }

    /// Loads application configuration from available sources.
    ///
    /// The configuration loading follows this priority:
    /// 1. YAML file specified by CONFIG_PATH environment variable
    /// 2. Default YAML file at ./config/bootstrap.yaml
    /// 3. Fall back to environment variables if file loading fails
    ///
    /// # Returns
    ///
    /// An `AppConfig` instance containing the application configuration
    fn load_config() -> AppConfig {
        // First check whether the configuration file path is specified through the environment
        // variable
        let config_path =
            std::env::var("CONFIG_PATH").unwrap_or_else(|_| DEFAULT_CONFIG_PATH.to_string());

        // Try to load the configuration from the specified YAML file
        if AppConfig::config_exists(&config_path) {
            match AppConfig::from_yaml(&config_path) {
                Ok(config) => {
                    info!(
                        "The configuration is loaded successfully from the configuration file {}",
                        config_path
                    );
                    return config;
                }
                Err(e) => {
                    warn!(
                        "Failed to load from configuration file: {}, will be configured using \
                         environment variables",
                        e
                    );
                }
            }
        } else {
            info!(
                "Configuration file {} does not exist, will use environment variable configuration",
                config_path
            );
        }

        // Fall back to environment variable configuration
        AppConfig::from_env()
    }

    /// Generates a default configuration file at the specified path.
    ///
    /// This is useful for creating a template configuration file that users
    /// can later modify according to their needs.
    ///
    /// # Parameters
    ///
    /// * `path` - The file path where the default configuration will be saved
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the file was successfully created
    /// * `Err(String)` - Error message if file creation failed
    pub fn generate_default_config<P: AsRef<Path>>(path: P) -> Result<(), String> {
        let config = AppConfig::default();
        config.save_to_yaml(path)
    }

    /// Runs the application by starting the configured servers.
    ///
    /// Based on the configuration, this method will start:
    /// - HTTP server (if enabled)
    /// - gRPC server (if enabled)
    /// - Both servers in parallel (if both enabled)
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all servers started and ran successfully
    /// * `Err(Box<dyn std::error::Error + Send + Sync>)` - If any server fails to start or run
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No servers are enabled in the configuration
    /// - Any server fails to initialize or run
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Yjs Collaboration Server");
        info!("Configuration: {:?}", self.config);

        // Start servers based on configuration
        match (self.config.enable_http, self.config.enable_grpc) {
            (true, true) => {
                // Start both HTTP and gRPC servers
                let http_server = HttpServer::new(
                    self.config.http_socket_addr(),
                    self.container.get_document_service(),
                );
                let rpc_server = RpcServer::new(
                    self.config.grpc_socket_addr(),
                    self.container.get_document_service(),
                );

                info!("Starting both HTTP and gRPC servers");
                try_join!(http_server.start(), rpc_server.start())?;
            }
            (true, false) => {
                // Start only HTTP server
                info!("Starting HTTP server only");
                let http_server = HttpServer::new(
                    self.config.http_socket_addr(),
                    self.container.get_document_service(),
                );
                http_server.start().await?;
            }
            (false, true) => {
                // Start only gRPC server
                info!("Starting gRPC server only");
                let rpc_server = RpcServer::new(
                    self.config.grpc_socket_addr(),
                    self.container.get_document_service(),
                );
                rpc_server.start().await?;
            }
            (false, false) => {
                return Err("No servers enabled in configuration".into());
            }
        }

        Ok(())
    }
}

/// Implementation of the Default trait for ApplicationBootstrap.
impl Default for ApplicationBootstrap {
    /// Creates a new ApplicationBootstrap instance with default settings.
    ///
    /// # Returns
    ///
    /// A new `ApplicationBootstrap` instance using the `new()` constructor
    fn default() -> Self {
        Self::new()
    }
}
