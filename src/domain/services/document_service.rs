use tokio::sync::broadcast;

use crate::domain::{
    entities::document::CollaborativeDocument,
    repositories::document_repository::DocumentRepository,
};

/// A domain service that manages collaborative documents and their operations.
///
/// This service provides comprehensive document collaboration capabilities:
/// - Manages multiple collaborative documents through a repository
/// - Handles document synchronization protocols
/// - Provides real-time update broadcasting
/// - Implements core business logic for collaborative editing
/// - Manages client synchronization and state vectors
///
/// This service represents the domain expertise around collaborative document operations
/// and encapsulates all business rules for document collaboration.
///
/// It uses the repository abstraction for data persistence, without knowing
/// about the concrete implementation details.
pub struct DocumentService<R: DocumentRepository> {
    document_repository: R,
}

impl<R: DocumentRepository> DocumentService<R> {
    /// Creates a new document service with the provided repository.
    ///
    /// # Arguments
    ///
    /// * `document_repository` - A repository implementation for document storage
    ///
    /// # Returns
    ///
    /// A new `DocumentService` instance.
    pub fn new(document_repository: R) -> Self {
        Self {
            document_repository,
        }
    }

    /// Establishes a synchronization session for a document.
    ///
    /// This is the core business logic for initiating collaboration on a document.
    /// It ensures the document exists and sets up the necessary channels for
    /// real-time collaboration.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * The document's current state vector as binary data
    /// * A broadcast receiver for future document updates
    pub async fn establish_sync_session(
        &self,
        doc_id: &str,
    ) -> (Vec<u8>, broadcast::Receiver<Vec<u8>>) {
        // Use repository abstraction - domain doesn't know about storage details
        let doc_service = self.document_repository.get_or_create(doc_id);

        // Get document state and subscribe to updates
        let state = doc_service.lock().await;
        let state_vector = state.get_state_vector();
        let update_receiver = state.subscribe();

        (state_vector, update_receiver)
    }

    /// Applies a document update using the collaborative editing protocol.
    ///
    /// This method encapsulates the business rules for applying updates to
    /// collaborative documents, ensuring data consistency and proper
    /// synchronization across all clients.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to update
    /// * `update_data` - The binary update data to apply
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub async fn apply_document_update(
        &self,
        doc_id: &str,
        update_data: &[u8],
    ) -> Result<(), String> {
        // Use repository abstraction for document access
        let doc_service = self.document_repository.get_or_create(doc_id);
        let mut state = doc_service.lock().await;
        state.apply_update(update_data)
    }

    /// Computes missing updates for client synchronization.
    ///
    /// This implements the core synchronization algorithm that determines
    /// what updates a client needs based on their current state vector.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    /// * `client_state_vector` - The client's current state vector
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Vec<u8>))` - Binary updates if the client needs them
    /// * `Ok(None)` - If the client is already up-to-date
    /// * `Err(String)` - An error message if synchronization failed
    pub async fn compute_missing_updates(
        &self,
        doc_id: &str,
        client_state_vector: &[u8],
    ) -> Result<Option<Vec<u8>>, String> {
        // Use repository abstraction
        let doc_service = self.document_repository.get_or_create(doc_id);
        let state = doc_service.lock().await;

        match state.get_missing_updates(client_state_vector) {
            Ok(update) => {
                if update.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(update))
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Retrieves the current state vector for a document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document
    ///
    /// # Returns
    ///
    /// A binary-encoded state vector that represents the current document state.
    pub async fn get_document_state_vector(&self, doc_id: &str) -> Vec<u8> {
        let doc_service = self.document_repository.get_or_create(doc_id);
        let state = doc_service.lock().await;
        state.get_state_vector()
    }

    /// Creates a subscription to document updates for a specific document.
    ///
    /// Clients can use this to receive real-time notifications when the document changes.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to subscribe to
    ///
    /// # Returns
    ///
    /// A broadcast receiver that will receive document state vector updates.
    pub async fn subscribe_to_document(&self, doc_id: &str) -> broadcast::Receiver<Vec<u8>> {
        let doc_service = self.document_repository.get_or_create(doc_id);
        let state = doc_service.lock().await;
        state.subscribe()
    }

    /// Domain business logic: Create a new document with validation.
    ///
    /// This method includes business rules like document ID validation.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the new document
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the document was created successfully
    /// * `Err(String)` - If creation failed or business rules were violated
    pub async fn create_new_document(&self, doc_id: &str) -> Result<(), String> {
        // Business rule: validate document ID
        if doc_id.is_empty() {
            return Err("Document ID cannot be empty".to_string());
        }

        if doc_id.len() > 255 {
            return Err("Document ID cannot exceed 255 characters".to_string());
        }

        // Use repository abstraction
        match self.document_repository.create_document(doc_id) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    /// Domain business logic: Delete a document with cleanup.
    ///
    /// This method includes business rules and cleanup logic.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the document was deleted successfully
    /// * `Err(String)` - If deletion failed
    pub async fn delete_document_with_cleanup(&self, doc_id: &str) -> Result<(), String> {
        // Business rule: check if document exists first
        if !self.document_repository.exists(doc_id) {
            return Err(format!("Document '{}' does not exist", doc_id));
        }

        // Use repository abstraction for deletion
        self.document_repository.delete_document(doc_id)
    }

    /// Domain business logic: Get repository statistics.
    ///
    /// This method provides business intelligence about the document repository.
    ///
    /// # Returns
    ///
    /// A tuple containing (total_documents, document_list)
    pub fn get_repository_stats(&self) -> (usize, Vec<String>) {
        let count = self.document_repository.count();
        let documents = self.document_repository.list_documents();
        (count, documents)
    }
}

/// Individual document service for managing a single collaborative document.
///
/// This service is used internally by repositories and wraps a `CollaborativeDocument`
/// entity with broadcasting capabilities.
pub struct SingleDocumentService {
    document: CollaborativeDocument,
    update_broadcaster: broadcast::Sender<Vec<u8>>,
}

impl SingleDocumentService {
    /// Creates a new single document service with an empty document and broadcast channel.
    ///
    /// # Returns
    ///
    /// A new `SingleDocumentService` instance with an initialized document and broadcast channel.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100); // buffer size for 100 updates
        Self {
            document: CollaborativeDocument::new(),
            update_broadcaster: tx,
        }
    }

    /// Retrieves the document's current state vector.
    ///
    /// # Returns
    ///
    /// A binary-encoded state vector that represents the current document state.
    pub fn get_state_vector(&self) -> Vec<u8> {
        self.document.get_state_vector()
    }

    /// Applies an update to the document and broadcasts it to all connected clients.
    ///
    /// # Arguments
    ///
    /// * `update` - A binary-encoded update to apply to the document
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied and broadcasted
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), String> {
        // Apply update to the document
        self.document.apply_update(update)?;

        // Broadcast the update to all connected clients
        // If there are no active receivers, this will just drop the message
        let _ = self.update_broadcaster.send(update.to_vec());

        Ok(())
    }

    /// Computes what updates a client needs based on their state vector.
    ///
    /// # Arguments
    ///
    /// * `client_state` - The client's current state vector
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Binary-encoded updates the client needs
    /// * `Err(String)` - An error message if the operation failed
    pub fn get_missing_updates(&self, client_state: &[u8]) -> Result<Vec<u8>, String> {
        self.document.get_missing_updates(client_state)
    }

    /// Creates a new subscription to this document's updates.
    ///
    /// # Returns
    ///
    /// A broadcast receiver that will receive updates when the document changes.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.update_broadcaster.subscribe()
    }
}

impl Default for SingleDocumentService {
    fn default() -> Self {
        Self::new()
    }
}
