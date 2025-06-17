use std::{net::SocketAddr, sync::Arc, time::Duration};

use tracing::info;
use volo_http::{
    context::ServerContext,
    http::StatusCode,
    server::{layer::TimeoutLayer, Server},
    Address,
};

use yjs_collaboration_server_adapter::http::router;
use yjs_collaboration_server_domain::services::document_service::DocumentService;
use yjs_collaboration_server_infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository;

/// HTTP server application service
/// Responsible for starting and managing the lifecycle of the HTTP server
pub struct HttpServer {
    addr: SocketAddr,
    document_service: Arc<DocumentService<InMemoryDocumentRepository>>,
}

impl HttpServer {
    pub fn new(
        addr: SocketAddr,
        document_service: Arc<DocumentService<InMemoryDocumentRepository>>,
    ) -> Self {
        Self {
            addr,
            document_service,
        }
    }

    /// Timeout handler
    fn timeout_handler(_: &ServerContext) -> (StatusCode, &'static str) {
        (StatusCode::INTERNAL_SERVER_ERROR, "Timeout!\n")
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting HTTP server on {}", self.addr);

        // Create router with dependency injection
        let http_router = router::HttpRouter::new(self.document_service.clone());
        let app = http_router.build_router().layer(TimeoutLayer::new(
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
