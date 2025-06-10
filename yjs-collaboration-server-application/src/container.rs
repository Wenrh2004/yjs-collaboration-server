use std::sync::Arc;
use yjs_collaboration_server_domain::services::document_service::DocumentService;
use yjs_collaboration_server_infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository;

/// Dependency injection container
/// Follows DDD architecture, manages dependencies across layers
pub struct Container {
    // Application layer
    document_service: Arc<DocumentService<InMemoryDocumentRepository>>,
}

impl Container {
    /// Create and configure all dependencies
    pub fn new() -> Self {
        // Create infrastructure dependencies
        let document_repository = InMemoryDocumentRepository::new();

        // Application layer - create use case service
        let document_service = Arc::new(DocumentService::new(document_repository));

        Self { document_service }
    }

    /// Get document use case service
    pub fn get_document_service(&self) -> Arc<DocumentService<InMemoryDocumentRepository>> {
        self.document_service.clone()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
