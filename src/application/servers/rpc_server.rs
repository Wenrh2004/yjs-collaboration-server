use std::{net::SocketAddr, sync::Arc};

use tracing::info;
use volo_grpc::server::{Server, ServiceBuilder};

use crate::{DocumentUseCases, InMemoryDocumentRepository, adapter::rpc::CollaborationServiceImpl};

/// RPC服务器应用服务
/// 负责gRPC服务器的启动和生命周期管理
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

    /// 启动gRPC服务器
    pub async fn start(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting gRPC server on {}", self.addr);

        // 创建协同服务
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
