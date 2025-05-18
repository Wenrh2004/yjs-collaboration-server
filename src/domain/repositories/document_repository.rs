use std::sync::Arc;

use tokio::sync::Mutex;

use crate::domain::services::document_service::DocumentService;

// Document repository interface, defines document storage and retrieval operations
pub trait DocumentRepository {
    // Get or create a document by ID
    fn get_or_create(&self, doc_id: &str) -> Arc<Mutex<DocumentService>>;
}
