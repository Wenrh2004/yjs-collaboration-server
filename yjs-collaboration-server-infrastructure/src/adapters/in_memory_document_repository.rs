use std::sync::Arc;

use dashmap::DashMap;
use once_cell::sync::Lazy;
use tokio::sync::Mutex;
use yjs_collaboration_server_domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::SingleDocumentServiceImpl,
};

/// Global in-memory storage for collaborative documents.
///
/// This static collection maintains document instances across the application.
/// It uses a thread-safe concurrent map with document IDs as keys and document services as values.
/// DashMap provides high-performance concurrent access without global locking.
/// The `Lazy` initialization ensures the storage is created only when first accessed.
static DOCUMENTS: Lazy<DashMap<String, Arc<Mutex<SingleDocumentServiceImpl>>>> =
    Lazy::new(|| DashMap::new());

/// An in-memory implementation of the document repository interface.
///
/// This repository stores all documents in memory using a static `DashMap`.
/// It provides a simple, non-persistent storage solution suitable for:
/// - Development and testing
/// - Small-scale deployments
/// - Scenarios where persistence is not required
/// - High-concurrency access patterns
///
/// Note: All documents are lost when the server restarts.
///
/// This implementation contains all the concrete CRUD logic that the domain
/// layer abstracts through the DocumentRepository trait.
pub struct InMemoryDocumentRepository;

impl InMemoryDocumentRepository {
    /// Creates a new in-memory document repository instance.
    ///
    /// # Returns
    ///
    /// A new `InMemoryDocumentRepository` instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl DocumentRepository for InMemoryDocumentRepository {
    /// Creates a new document with the given ID.
    ///
    /// This is the concrete implementation of document creation logic.
    fn create_document(
        &self,
        doc_id: &str,
    ) -> Result<Arc<Mutex<SingleDocumentServiceImpl>>, String> {
        // With DashMap, we can check for existence and insert atomically
        if DOCUMENTS.contains_key(doc_id) {
            return Err(format!("Document with ID '{}' already exists", doc_id));
        }

        let doc_service = Arc::new(Mutex::new(SingleDocumentServiceImpl::new()));
        DOCUMENTS.insert(doc_id.to_string(), doc_service.clone());

        Ok(doc_service)
    }

    /// Retrieves an existing document by ID.
    ///
    /// This is the concrete implementation of document retrieval logic.
    fn get_document(&self, doc_id: &str) -> Option<Arc<Mutex<SingleDocumentServiceImpl>>> {
        // With DashMap, we can directly get values without locking the entire map
        DOCUMENTS.get(doc_id).map(|entry| entry.value().clone())
    }

    /// Retrieves an existing document by ID or creates a new one if it doesn't exist.
    ///
    /// This is the concrete implementation that combines get and create operations.
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<SingleDocumentServiceImpl>> {
        // Use entry API for atomic get-or-insert operations
        DOCUMENTS
            .entry(doc_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(SingleDocumentServiceImpl::new())))
            .value()
            .clone()
    }

    /// Updates an existing document.
    ///
    /// This is the concrete implementation of document update logic.
    fn update_document(
        &self,
        doc_id: &str,
        document: Arc<Mutex<SingleDocumentServiceImpl>>,
    ) -> Result<(), String> {
        if !DOCUMENTS.contains_key(doc_id) {
            return Err(format!("Document with ID '{}' does not exist", doc_id));
        }

        DOCUMENTS.insert(doc_id.to_string(), document);
        Ok(())
    }

    /// Deletes a document by ID.
    ///
    /// This is the concrete implementation of document deletion logic.
    fn delete_document(&self, doc_id: &str) -> Result<(), String> {
        if DOCUMENTS.remove(doc_id).is_some() {
            Ok(())
        } else {
            Err(format!("Document with ID '{}' does not exist", doc_id))
        }
    }

    /// Lists all document IDs in the repository.
    ///
    /// This is the concrete implementation of document listing logic.
    fn list_documents(&self) -> Vec<String> {
        // Collect keys from DashMap
        DOCUMENTS.iter().map(|entry| entry.key().clone()).collect()
    }

    /// Checks if a document exists.
    ///
    /// This is the concrete implementation that checks document existence.
    fn exists(&self, doc_id: &str) -> bool {
        DOCUMENTS.contains_key(doc_id)
    }

    /// Gets the total number of documents in the repository.
    ///
    /// This is the concrete implementation that counts documents.
    fn count(&self) -> usize {
        DOCUMENTS.len()
    }

    /// Clears all documents from the repository.
    ///
    /// This is the concrete implementation of repository clearing logic.
    fn clear(&self) -> Result<(), String> {
        DOCUMENTS.clear();
        Ok(())
    }
}

impl Default for InMemoryDocumentRepository {
    fn default() -> Self {
        Self::new()
    }
}
