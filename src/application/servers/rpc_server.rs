use std::{net::SocketAddr, sync::Arc};

use tracing::info;
use volo_grpc::server::{Server, ServiceBuilder};

use crate::{
    adapter::rpc::CollaborationServiceImpl,
    application::services::document_application_service::DocumentUseCases,
    infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository,
};

/// RPC server application service
/// Responsible for starting and managing the lifecycle of the gRPC server
pub struct RpcServer {
    addr: SocketAddr,
    document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
}

impl RpcServer {
    pub fn new(
        addr: SocketAddr,
        document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
    ) -> Self {
        Self {
            addr,
            document_use_cases,
        }
    }

    /// Start the gRPC server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting gRPC server on {}", self.addr);

        // Create collaboration service
        let collaboration_service = CollaborationServiceImpl::new(self.document_use_cases);

        let addr = volo::net::Address::from(self.addr);

        Server::new()
            .add_service(
                ServiceBuilder::new(volo_gen::collaboration::CollaborationServiceServer::new(
                    collaboration_service,
                ))
                .build(),
            )
            .run(addr)
            .await
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { e.into() })?;

        Ok(())
    }
}
