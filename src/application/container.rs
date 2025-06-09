use std::sync::Arc;

use crate::{
    application::services::document_application_service::DocumentUseCases,
    domain::factory::RepositoryFactory,
    infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository,
};

/// Dependency injection container
/// Follows DDD architecture, manages dependencies across layers
pub struct Container {
    // Application layer
    pub document_use_cases: Arc<DocumentUseCases<InMemoryDocumentRepository>>,
}

impl Container {
    /// Create and configure all dependencies using domain factory
    pub fn new() -> Self {
        // Use domain factory to create infrastructure dependencies
        let document_repository = RepositoryFactory::create_document_repository();

        // Application layer - create use case service
        let document_use_cases = Arc::new(DocumentUseCases::new(document_repository));

        Self { document_use_cases }
    }

    /// Get document use case service
    pub fn get_document_use_cases(&self) -> Arc<DocumentUseCases<InMemoryDocumentRepository>> {
        self.document_use_cases.clone()
    }
}

impl Default for Container {
    fn default() -> Self {
        Self::new()
    }
}
