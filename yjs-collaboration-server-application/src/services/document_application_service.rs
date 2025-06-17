use std::sync::Arc;

use yjs_collaboration_server_domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::DocumentService
    ,
};

/// Application service implementing complex document use cases and workflows.
///
/// This service acts as an orchestration layer that coordinates multiple domain services
/// to implement higher-level application features and workflows. Its responsibilities are:
/// - Coordinating calls to multiple domain services
/// - Implementing complex business processes that span multiple domain concepts
/// - Handling application-specific concerns like message formatting and transformation
/// - Managing transactions and consistency across multiple operations
///
/// Rather than being a required intermediary between adapters and domain, it provides
/// additional value by implementing complex workflows that adapters can choose to use
/// when appropriate.
pub struct DocumentUseCases<R: DocumentRepository> {
    document_service: Arc<DocumentService<R>>,
    // Other domain services could be added here for orchestration
}

impl<R: DocumentRepository + Send + Sync + 'static> DocumentUseCases<R> {
    /// Creates a new document application service with the provided document service.
    ///
    /// # Arguments
    ///
    /// * `document_service` - An Arc reference to a domain document service
    ///
    /// # Returns
    ///
    /// A new `DocumentUseCases` instance.
    pub fn new(document_service: Arc<DocumentService<R>>) -> Self {
        Self {
            document_service,
        }
    }

    /// Alternative constructor that creates both the domain service and the use cases.
    ///
    /// # Arguments
    ///
    /// * `document_repository` - A repository implementation for document storage
    ///
    /// # Returns
    ///
    /// A new `DocumentUseCases` instance with freshly created domain service
    pub fn with_repository(document_repository: R) -> Self {
        let document_service = Arc::new(DocumentService::new(document_repository));
        Self {
            document_service,
        }
    }

    /// Get direct access to the domain service.
    ///
    /// This method allows adapters to access the domain service directly
    /// when they don't need the additional orchestration provided by this layer.
    ///
    /// # Returns
    ///
    /// A reference to the domain document service
    pub fn get_document_service(&self) -> Arc<DocumentService<R>> {
        self.document_service.clone()
    }

}
