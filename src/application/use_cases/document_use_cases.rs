use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::domain::{
    repositories::document_repository::DocumentRepository, value_objects::message::ServerMessage,
};

/// Application service implementing use cases for collaborative document operations.
///
/// This service acts as the primary interface between external adapters (like HTTP handlers)
/// and the domain layer. It orchestrates document operations by:
/// - Translating between external formats (like Base64) and domain formats
/// - Coordinating repository access
/// - Managing document synchronization flows
///
/// The service accepts a generic repository implementation conforming to the `DocumentRepository`
/// trait, allowing for different storage strategies.
pub struct DocumentUseCases<R: DocumentRepository> {
    document_repository: R,
}

impl<R: DocumentRepository> DocumentUseCases<R> {
    /// Creates a new document use case service with the provided repository.
    ///
    /// # Arguments
    ///
    /// * `document_repository` - A repository implementation for document storage
    ///
    /// # Returns
    ///
    /// A new `DocumentUseCases` instance.
    pub fn new(document_repository: R) -> Self {
        Self {
            document_repository,
        }
    }

    /// Handles a client's initial synchronization request.
    ///
    /// This use case handles a client connecting to a document for the first time
    /// or reconnecting after a disconnection. It:
    /// 1. Gets or creates the requested document
    /// 2. Returns its current state vector
    /// 3. Sets up a subscription for real-time updates
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A `ServerMessage` with the document's current state vector
    /// * A broadcast receiver for future document updates
    pub async fn handle_sync_request(
        &self,
        doc_id: &str,
    ) -> (ServerMessage, tokio::sync::broadcast::Receiver<Vec<u8>>) {
        let doc_service = self.document_repository.get_or_create(doc_id);

        // Get document state and subscribe to updates
        let (state_vector, update_receiver) = {
            let state = doc_service.lock().await;
            let sv = state.get_state_vector();
            let receiver = state.subscribe();
            (sv, receiver)
        };

        // Create response message
        let response = ServerMessage {
            message_type: "sv".to_string(),
            data: None,
            update: Some(BASE64.encode(&state_vector)),
        };

        (response, update_receiver)
    }

    /// Handles a client's update to a document.
    ///
    /// This use case processes an update sent from a client and applies it to
    /// the specified document. The update is provided in Base64 format and gets
    /// decoded before being applied to the document.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to update
    /// * `update_base64` - The document update encoded in Base64
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
        match BASE64.decode(update_base64.as_bytes()) {
            Ok(update) => {
                let doc_service = self.document_repository.get_or_create(doc_id);
                let mut state = doc_service.lock().await;
                state.apply_update(&update)
            }
            Err(_) => Err("Failed to decode base64 update data".to_string()),
        }
    }

    /// Handles a client's request to synchronize with a document using its state vector.
    ///
    /// This use case allows a client to receive only the updates it's missing by:
    /// 1. Comparing the client's state vector with the document's current state
    /// 2. Generating an update containing only the missing changes
    /// 3. Returning this update to bring the client up-to-date
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    /// * `sv_base64` - The client's state vector encoded in Base64
    ///
    /// # Returns
    ///
    /// * `Ok(Some(ServerMessage))` - A message containing the updates if there are any
    /// * `Ok(None)` - If the client is already up-to-date
    /// * `Err(String)` - An error message if synchronization failed
    pub async fn handle_state_vector_request(
        &self,
        doc_id: &str,
        sv_base64: &str,
    ) -> Result<Option<ServerMessage>, String> {
        match BASE64.decode(sv_base64.as_bytes()) {
            Ok(sv) => {
                let doc_service = self.document_repository.get_or_create(doc_id);
                let state = doc_service.lock().await;

                match state.get_missing_updates(&sv) {
                    Ok(update) => {
                        if update.is_empty() {
                            // No updates to sync
                            Ok(None)
                        } else {
                            // Create response with updates
                            let response = ServerMessage {
                                message_type: "update".to_string(),
                                data: None,
                                update: Some(BASE64.encode(&update)),
                            };
                            Ok(Some(response))
                        }
                    }
                    Err(e) => Err(e),
                }
            }
            Err(_) => Err("Failed to decode base64 state vector data".to_string()),
        }
    }

    /// Handles a binary update directly (without Base64 decoding).
    ///
    /// This use case is similar to `handle_update_request` but accepts raw binary data
    /// instead of Base64-encoded data, which can be more efficient when working with
    /// binary protocols like WebSockets.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to update
    /// * `binary_data` - The raw binary update data
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub async fn handle_binary_update(
        &self,
        doc_id: &str,
        binary_data: &[u8],
    ) -> Result<(), String> {
        let doc_service = self.document_repository.get_or_create(doc_id);
        let mut state = doc_service.lock().await;
        state.apply_update(binary_data)
    }
}
