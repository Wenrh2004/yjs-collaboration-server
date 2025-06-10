use std::sync::Arc;

use base64::Engine;
use tokio::sync::{broadcast, Mutex};

use crate::{
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

    /// Handles a sync request from a client.
    ///
    /// This method processes client synchronization requests and returns the current
    /// document state along with a receiver for future updates.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A SyncResponse with the document's current state
    /// * A broadcast receiver for future document updates
    pub async fn handle_sync_request(
        &self,
        doc_id: &str,
    ) -> (SyncResponse, broadcast::Receiver<UpdateNotification>) {
        let (state_vector, receiver) = self.establish_sync_session(doc_id).await;

        let response = SyncResponse {
            update: None,
            state_vector,
        };

        (response, receiver)
    }

    /// Handles an update request from a client.
    ///
    /// This method processes document updates sent by clients in Base64 format.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to update
    /// * `update_base64` - The Base64-encoded update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub async fn handle_update_request(
        &self,
        doc_id: &str,
        update_base64: &str,
    ) -> Result<(), String> {
        // Decode Base64 update data
        let update_data = base64::engine::general_purpose::STANDARD
            .decode(update_base64)
            .map_err(|e| format!("Failed to decode Base64 update: {}", e))?;

        // Apply the update using existing method
        self.apply_document_update(doc_id, &update_data).await
    }

    /// Handles a synchronization step with a state vector from a client.
    ///
    /// This method processes client state vectors and returns the necessary updates
    /// to bring the client up to date.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize
    /// * `state_vector_base64` - The Base64-encoded client state vector
    ///
    /// # Returns
    ///
    /// A result containing:
    /// * `Ok((SyncResponse, receiver))` - Response with updates and receiver for future updates
    /// * `Err(String)` - An error message if synchronization couldn't be processed
    pub async fn handle_sync_step(
        &self,
        doc_id: &str,
        state_vector_base64: &str,
    ) -> Result<(SyncResponse, broadcast::Receiver<UpdateNotification>), String> {
        // Decode Base64 state vector
        let state_vector = base64::engine::general_purpose::STANDARD
            .decode(state_vector_base64)
            .map_err(|e| format!("Failed to decode Base64 state vector: {}", e))?;

        // Sync with the provided state vector
        let (update, receiver) = self.sync_document(doc_id, Some(&state_vector)).await;

        let response = SyncResponse {
            update: if update.is_empty() {
                None
            } else {
                Some(base64::engine::general_purpose::STANDARD.encode(&update))
            },
            state_vector: vec![], // The state vector should be obtained separately
        };

        Ok((response, receiver))
    }

    /// Handles a binary update from a client.
    ///
    /// This method processes binary document updates directly without Base64 encoding.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to update
    /// * `update_data` - The binary update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub async fn handle_binary_update(
        &self,
        doc_id: &str,
        update_data: &[u8],
    ) -> Result<(), String> {
        // Apply the update using existing method
        self.apply_document_update(doc_id, update_data).await
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
    ) -> (Vec<u8>, broadcast::Receiver<UpdateNotification>) {
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
        let state = doc_service.lock().await;
        state.apply_update(update_data).await
    }

    /// Computes missing updates for client synchronization.
    ///
    /// This implements the core synchronization algorithm that determines
    /// what updates a client needs based on their current state vector.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize
    /// * `client_state_vector` - The client's current state vector
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * The binary update data the client needs
    /// * A broadcast receiver for future document updates
    pub async fn sync_document(
        &self,
        doc_id: &str,
        client_state_vector: Option<&[u8]>,
    ) -> (Vec<u8>, broadcast::Receiver<UpdateNotification>) {
        let doc_service = self.document_repository.get_or_create(doc_id);

        // Use read lock for sync operation as it primarily reads the document state
        let state = doc_service.lock().await;

        // Generate update based on client's state vector
        let update = match client_state_vector {
            Some(sv) => state.diff_update(sv),
            None => state.get_state_vector(),
        };

        let receiver = state.subscribe();
        (update, receiver)
    }

    /// Gets the complete content of a document.
    ///
    /// This method provides access to the document's full content,
    /// typically for initial loading or full state recovery.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to retrieve
    ///
    /// # Returns
    ///
    /// * `Some(String)` - The document content if the document exists
    /// * `None` - If the document doesn't exist
    pub async fn get_document_content(&self, doc_id: &str) -> Option<String> {
        let doc_service = self.document_repository.get_document(doc_id)?;

        // Use read lock as this operation only reads the document
        let state = doc_service.lock().await;
        Some(state.get_content())
    }
}

/// Response to a sync request
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SyncResponse {
    /// The binary update to apply
    pub update: Option<String>,
    /// Binary state vector
    pub state_vector: Vec<u8>,
}

/// Notification of a document update
#[derive(Clone, Debug)]
pub struct UpdateNotification {
    /// The binary update data
    pub update: Vec<u8>,
    /// Source of the update
    pub source: String,
}

/// Concrete implementation of a single document service using Yjs CRDT
pub struct SingleDocumentServiceImpl {
    /// The collaborative document instance
    document: Arc<Mutex<CollaborativeDocument>>,
    /// Broadcast channel for sending updates to subscribers
    update_sender: broadcast::Sender<UpdateNotification>,
}

impl SingleDocumentServiceImpl {
    /// Creates a new document service instance
    pub fn new() -> Self {
        let (update_sender, _) = broadcast::channel(1024);

        Self {
            document: Arc::new(Mutex::new(CollaborativeDocument::new())),
            update_sender,
        }
    }

    /// Get the current state of the document
    pub async fn get_state(&self) -> SyncResponse {
        let doc = self.document.lock().await;
        SyncResponse {
            update: None,
            state_vector: doc.get_state_vector(),
        }
    }

    /// Apply an update to the document
    pub async fn apply_update(&self, update_data: &[u8]) -> Result<(), String> {
        let mut doc = self.document.lock().await;
        doc.apply_update(update_data)?;

        // Broadcast the update to subscribers
        let notification = UpdateNotification {
            update: update_data.to_vec(),
            source: "server".to_string(),
        };

        let _ = self.update_sender.send(notification);
        Ok(())
    }

    /// Subscribe to updates to the document
    pub fn subscribe(&self) -> broadcast::Receiver<UpdateNotification> {
        self.update_sender.subscribe()
    }

    /// Get the current content of the document
    pub fn get_content(&self) -> String {
        "".to_string() // Placeholder implementation
    }

    /// Get the current state vector of the document
    pub fn get_state_vector(&self) -> Vec<u8> {
        vec![] // Placeholder implementation
    }

    /// Get a diff update based on the provided state vector
    pub fn diff_update(&self, _state_vector: &[u8]) -> Vec<u8> {
        vec![] // Placeholder implementation
    }
}

impl Default for SingleDocumentServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}
