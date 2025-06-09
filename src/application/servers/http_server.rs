use std::{net::SocketAddr, sync::Arc, time::Duration};

use tracing::info;
use volo_http::{
    Address,
    context::ServerContext,
    http::StatusCode,
    server::{Server, layer::TimeoutLayer},
};

use crate::{DocumentUseCases, InMemoryDocumentRepository};

/// HTTP服务器应用服务
/// 负责HTTP服务器的启动和生命周期管理
pub struct HttpServer {
    addr: SocketAddr,
    document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
}

impl HttpServer {
    pub fn new(
        addr: SocketAddr,
        document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
    ) -> Self {
        Self {
            addr,
            document_use_cases,
        }
    }

    /// 超时处理器
    fn timeout_handler(_: &ServerContext) -> (StatusCode, &'static str) {
        (StatusCode::INTERNAL_SERVER_ERROR, "Timeout!\n")
    }

    /// 启动HTTP服务器
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting HTTP server on {}", self.addr);

        // 使用lib.rs中的create_router函数
        let app = crate::create_router().layer(TimeoutLayer::new(
            Duration::from_secs(30),
            Self::timeout_handler,
        ));

        let addr = Address::from(self.addr);

        Server::new(app)
            .run(addr)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

        Ok(())
    }
}
