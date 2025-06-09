use std::sync::Arc;

use tokio::sync::Mutex;

use crate::domain::services::document_service::DocumentService;

/// Repository interface for document storage and retrieval operations.
///
/// This trait defines the contract for accessing and manipulating collaborative documents.
/// It abstracts the storage mechanism for documents, allowing for different implementations
/// (in-memory, persistent storage, etc.) while maintaining a consistent interface.
///
/// Implementations must be thread-safe as they will be accessed concurrently.
pub trait DocumentRepository {
    /// Retrieves an existing document by ID or creates a new one if it doesn't exist.
    ///
    /// This method follows the "get or create" pattern, ensuring that a document
    /// with the specified ID will always be available after this call.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// A thread-safe reference to the document service for the requested document.
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<DocumentService>>;
}
