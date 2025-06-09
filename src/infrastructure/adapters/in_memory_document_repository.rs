use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::SingleDocumentService,
};

/// Global in-memory storage for collaborative documents.
///
/// This static collection maintains document instances across the application.
/// It uses a thread-safe map with document IDs as keys and document services as values.
/// The `Lazy` initialization ensures the storage is created only when first accessed.
static DOCUMENTS: Lazy<Arc<StdMutex<HashMap<String, Arc<Mutex<SingleDocumentService>>>>>> =
    Lazy::new(|| Arc::new(StdMutex::new(HashMap::new())));

/// An in-memory implementation of the document repository interface.
///
/// This repository stores all documents in memory using a static `HashMap`.
/// It provides a simple, non-persistent storage solution suitable for:
/// - Development and testing
/// - Small-scale deployments
/// - Scenarios where persistence is not required
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
    fn create_document(&self, doc_id: &str) -> Result<Arc<Mutex<SingleDocumentService>>, String> {
        let mut docs = DOCUMENTS.lock().unwrap();

        if docs.contains_key(doc_id) {
            return Err(format!("Document with ID '{}' already exists", doc_id));
        }

        let doc_service = Arc::new(Mutex::new(SingleDocumentService::new()));
        docs.insert(doc_id.to_string(), doc_service.clone());

        Ok(doc_service)
    }

    /// Retrieves an existing document by ID.
    ///
    /// This is the concrete implementation of document retrieval logic.
    fn get_document(&self, doc_id: &str) -> Option<Arc<Mutex<SingleDocumentService>>> {
        let docs = DOCUMENTS.lock().unwrap();
        docs.get(doc_id).cloned()
    }

    /// Retrieves an existing document by ID or creates a new one if it doesn't exist.
    ///
    /// This is the concrete implementation that combines get and create operations.
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<SingleDocumentService>> {
        let mut docs = DOCUMENTS.lock().unwrap();

        if !docs.contains_key(doc_id) {
            let doc_service = Arc::new(Mutex::new(SingleDocumentService::new()));
            docs.insert(doc_id.to_string(), doc_service.clone());
            doc_service
        } else {
            docs.get(doc_id).unwrap().clone()
        }
    }

    /// Updates an existing document.
    ///
    /// This is the concrete implementation of document update logic.
    fn update_document(
        &self,
        doc_id: &str,
        document: Arc<Mutex<SingleDocumentService>>,
    ) -> Result<(), String> {
        let mut docs = DOCUMENTS.lock().unwrap();

        if !docs.contains_key(doc_id) {
            return Err(format!("Document with ID '{}' does not exist", doc_id));
        }

        docs.insert(doc_id.to_string(), document);
        Ok(())
    }

    /// Deletes a document by ID.
    ///
    /// This is the concrete implementation of document deletion logic.
    fn delete_document(&self, doc_id: &str) -> Result<(), String> {
        let mut docs = DOCUMENTS.lock().unwrap();

        if docs.remove(doc_id).is_some() {
            Ok(())
        } else {
            Err(format!("Document with ID '{}' does not exist", doc_id))
        }
    }

    /// Lists all document IDs in the repository.
    ///
    /// This is the concrete implementation of document listing logic.
    fn list_documents(&self) -> Vec<String> {
        let docs = DOCUMENTS.lock().unwrap();
        docs.keys().cloned().collect()
    }

    /// Checks if a document exists.
    ///
    /// This is the concrete implementation of document existence check logic.
    fn exists(&self, doc_id: &str) -> bool {
        let docs = DOCUMENTS.lock().unwrap();
        docs.contains_key(doc_id)
    }

    /// Gets the total number of documents in the repository.
    ///
    /// This is the concrete implementation of document counting logic.
    fn count(&self) -> usize {
        let docs = DOCUMENTS.lock().unwrap();
        docs.len()
    }

    /// Clears all documents from the repository.
    ///
    /// This is the concrete implementation of repository clearing logic.
    fn clear(&self) -> Result<(), String> {
        let mut docs = DOCUMENTS.lock().unwrap();
        docs.clear();
        Ok(())
    }
}

impl Default for InMemoryDocumentRepository {
    fn default() -> Self {
        Self::new()
    }
}
