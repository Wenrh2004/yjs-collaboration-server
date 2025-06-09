use std::{net::SocketAddr, sync::Arc, time::Duration};

use tracing::info;
use volo_http::{
    context::ServerContext,
    http::StatusCode,
    server::{layer::TimeoutLayer, Server},
    Address,
};

use crate::{
    adapter::http::router, application::use_cases::document_use_cases::DocumentUseCases,
    infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository,
};

/// HTTP server application service
/// Responsible for starting and managing the lifecycle of the HTTP server
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

    /// Timeout handler
    fn timeout_handler(_: &ServerContext) -> (StatusCode, &'static str) {
        (StatusCode::INTERNAL_SERVER_ERROR, "Timeout!\n")
    }

    /// Start the HTTP server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting HTTP server on {}", self.addr);

        // Use the create_router function from lib.rs
        let app = router::create_router().layer(TimeoutLayer::new(
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
