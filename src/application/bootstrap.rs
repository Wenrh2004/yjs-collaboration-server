use tokio::try_join;
use tracing::info;

use super::{
    config::AppConfig,
    container::Container,
    servers::{HttpServer, RpcServer},
};

/// Application bootstrap service
/// Responsible for overall application startup and dependency coordination
pub struct ApplicationBootstrap {
    config: AppConfig,
    container: Container,
}

impl ApplicationBootstrap {
    /// Create an application bootstrap instance
    pub fn new() -> Self {
        let config = AppConfig::from_env();
        config.init_logging();

        let container = Container::new();

        Self { config, container }
    }

    /// Run the application
    /// Start servers based on configuration
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Yjs Collaboration Server");
        info!("Configuration: {:?}", self.config);

        // Start servers based on configuration
        match (self.config.enable_http, self.config.enable_grpc) {
            (true, true) => {
                // Start both HTTP and gRPC servers
                let http_server = HttpServer::new(
                    self.config.http_addr,
                    self.container.get_document_use_cases(),
                );
                let rpc_server = RpcServer::new(
                    self.config.grpc_addr,
                    self.container.get_document_use_cases(),
                );

                info!("Starting both HTTP and gRPC servers");
                try_join!(http_server.start(), rpc_server.start())?;
            }
            (true, false) => {
                // Start only HTTP server
                info!("Starting HTTP server only");
                let http_server = HttpServer::new(
                    self.config.http_addr,
                    self.container.get_document_use_cases(),
                );
                http_server.start().await?;
            }
            (false, true) => {
                // Start only gRPC server
                info!("Starting gRPC server only");
                let rpc_server = RpcServer::new(
                    self.config.grpc_addr,
                    self.container.get_document_use_cases(),
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

impl Default for ApplicationBootstrap {
    fn default() -> Self {
        Self::new()
    }
}
