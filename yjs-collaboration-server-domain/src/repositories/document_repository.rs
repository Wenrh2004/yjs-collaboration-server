use std::sync::Arc;

use tokio::sync::Mutex;

use crate::services::document_service::SingleDocumentServiceImpl;

/// Repository interface for document storage and retrieval operations.
///
/// This trait defines the contract for accessing and manipulating collaborative documents.
/// It abstracts the storage mechanism for documents, allowing for different implementations
/// (in-memory, persistent storage, etc.) while maintaining a consistent interface.
///
/// All methods in this trait are pure abstractions - the actual CRUD logic
/// is implemented in the infrastructure layer.
///
/// Implementations must be thread-safe as they will be accessed concurrently.
pub trait DocumentRepository: Send + Sync {
    /// Creates a new document with the given ID.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// * `Ok(Arc<Mutex<SingleDocumentServiceImpl>>)` - If the document was created successfully
    /// * `Err(String)` - If the document already exists or creation failed
    fn create_document(
        &self,
        doc_id: &str,
    ) -> Result<Arc<Mutex<SingleDocumentServiceImpl>>, String>;

    /// Retrieves an existing document by ID.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// * `Some(Arc<Mutex<SingleDocumentServiceImpl>>)` - If the document exists
    /// * `None` - If the document does not exist
    fn get_document(&self, doc_id: &str) -> Option<Arc<Mutex<SingleDocumentServiceImpl>>>;

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
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<SingleDocumentServiceImpl>>;

    /// Updates an existing document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    /// * `document` - The updated document service
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the document was updated successfully
    /// * `Err(String)` - If the document does not exist or update failed
    fn update_document(
        &self,
        doc_id: &str,
        document: Arc<Mutex<SingleDocumentServiceImpl>>,
    ) -> Result<(), String>;

    /// Deletes a document by ID.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the document was deleted successfully
    /// * `Err(String)` - If the document does not exist or deletion failed
    fn delete_document(&self, doc_id: &str) -> Result<(), String>;

    /// Lists all document IDs in the repository.
    ///
    /// # Returns
    ///
    /// A vector of all document IDs currently stored in the repository.
    fn list_documents(&self) -> Vec<String>;

    /// Checks if a document exists.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// * `true` - If the document exists
    /// * `false` - If the document does not exist
    fn exists(&self, doc_id: &str) -> bool;

    /// Gets the total number of documents in the repository.
    ///
    /// # Returns
    ///
    /// The number of documents currently stored in the repository.
    fn count(&self) -> usize;

    /// Clears all documents from the repository.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If all documents were cleared successfully
    /// * `Err(String)` - If the operation failed
    fn clear(&self) -> Result<(), String>;
}
