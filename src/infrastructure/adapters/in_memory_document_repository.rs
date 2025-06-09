use std::{
    collections::HashMap,
    sync::{Arc, Mutex as StdMutex},
};

use once_cell::sync::Lazy;
use tokio::sync::Mutex;

use crate::domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::DocumentService,
};

/// Global in-memory storage for collaborative documents.
///
/// This static collection maintains document instances across the application.
/// It uses a thread-safe map with document IDs as keys and document services as values.
/// The `Lazy` initialization ensures the storage is created only when first accessed.
static DOCUMENTS: Lazy<Arc<StdMutex<HashMap<String, Arc<Mutex<DocumentService>>>>>> =
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
    /// Retrieves a document by ID from memory or creates a new one if it doesn't exist.
    ///
    /// This implementation uses a global static `HashMap` to store documents,
    /// making them accessible across different parts of the application.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - A string identifier for the document
    ///
    /// # Returns
    ///
    /// A thread-safe reference to the document service for the requested document.
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<DocumentService>> {
        let mut docs = DOCUMENTS.lock().unwrap();

        if !docs.contains_key(doc_id) {
            let doc_service = Arc::new(Mutex::new(DocumentService::new()));
            docs.insert(doc_id.to_string(), doc_service.clone());
            doc_service
        } else {
            docs.get(doc_id).unwrap().clone()
        }
    }
}
