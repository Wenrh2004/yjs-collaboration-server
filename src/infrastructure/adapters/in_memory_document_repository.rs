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

/// Global singleton for document storage
static DOCUMENTS: Lazy<Arc<StdMutex<HashMap<String, Arc<Mutex<DocumentService>>>>>> =
    Lazy::new(|| Arc::new(StdMutex::new(HashMap::new())));

/// In-memory document repository implementation
pub struct InMemoryDocumentRepository;

impl InMemoryDocumentRepository {
    pub fn new() -> Self {
        Self {}
    }
}

impl DocumentRepository for InMemoryDocumentRepository {
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
