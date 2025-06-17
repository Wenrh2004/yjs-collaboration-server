use std::{net::SocketAddr, sync::Arc};

use tracing::info;
use volo_grpc::server::{Server, ServiceBuilder};
use yjs_collaboration_server_adapter::rpc::collaboration_service::CollaborationServiceImpl;
use yjs_collaboration_server_common::volo_gen;
use yjs_collaboration_server_domain::services::document_service::DocumentService;
use yjs_collaboration_server_infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository;

/// RPC server application service
/// Responsible for starting and managing the lifecycle of the gRPC server
pub struct RpcServer {
    addr: SocketAddr,
    document_service: Arc<DocumentService<InMemoryDocumentRepository>>,
}

impl RpcServer {
    pub fn new(
        addr: SocketAddr,
        document_service: Arc<DocumentService<InMemoryDocumentRepository>>,
    ) -> Self {
        Self {
            addr,
            document_service,
        }
    }

    /// Start the gRPC server
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting gRPC server on {}", self.addr);

        // Create collaboration service
        let collaboration_service = CollaborationServiceImpl::new(self.document_service.clone());

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
