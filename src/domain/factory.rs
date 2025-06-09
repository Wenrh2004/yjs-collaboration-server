use std::sync::Arc;

use crate::{
    domain::repositories::document_repository::DocumentRepository,
    infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository,
};

/// Domain layer factory for creating infrastructure implementations.
///
/// This factory encapsulates the creation of infrastructure layer components,
/// following the dependency inversion principle where domain layer depends on infrastructure.
/// The factory pattern allows domain layer to control the creation of infrastructure
/// implementations while maintaining separation of concerns.
pub struct RepositoryFactory;

impl RepositoryFactory {
    /// Creates a new in-memory document repository instance.
    ///
    /// This method returns the default implementation (InMemoryDocumentRepository).
    /// In the future, this can be extended to support different implementations
    /// based on configuration.
    ///
    /// # Returns
    ///
    /// A new InMemoryDocumentRepository instance.
    pub fn create_document_repository() -> InMemoryDocumentRepository {
        InMemoryDocumentRepository::new()
    }

    /// Creates an Arc-wrapped document repository for shared ownership.
    ///
    /// # Returns
    ///
    /// An Arc-wrapped document repository implementation for concurrent access.
    pub fn create_shared_document_repository() -> Arc<dyn DocumentRepository + Send + Sync> {
        Arc::new(InMemoryDocumentRepository::new())
    }
}
