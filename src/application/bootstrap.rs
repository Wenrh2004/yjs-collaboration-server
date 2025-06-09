use tokio::try_join;
use tracing::info;

use super::{
    config::AppConfig,
    container::Container,
    servers::{HttpServer, RpcServer},
};

/// 应用引导服务
/// 负责应用的整体启动和依赖协调
pub struct ApplicationBootstrap {
    config: AppConfig,
    container: Container,
}

impl ApplicationBootstrap {
    /// 创建应用引导实例
    pub fn new() -> Self {
        let config = AppConfig::from_env();
        config.init_logging();

        let container = Container::new();

        Self { config, container }
    }

    /// 启动应用
    /// 根据配置启动不同的服务器
    pub async fn run(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting Yjs Collaboration Server");
        info!("Configuration: {:?}", self.config);

        // 根据配置启动不同的服务器
        match (self.config.enable_http, self.config.enable_grpc) {
            (true, true) => {
                // 同时启动HTTP和gRPC服务器
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
                // 只启动HTTP服务器
                info!("Starting HTTP server only");
                let http_server = HttpServer::new(
                    self.config.http_addr,
                    self.container.get_document_use_cases(),
                );
                http_server.start().await?;
            }
            (false, true) => {
                // 只启动gRPC服务器
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
