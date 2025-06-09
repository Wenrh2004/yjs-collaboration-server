use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

use crate::domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::DocumentService, value_objects::message::ServerMessage,
};

/// Application service implementing collaborative document operations.
///
/// This service acts as a thin coordination layer between external adapters (like HTTP handlers)
/// and the domain layer. Its responsibilities are:
/// - Translating between external formats (like Base64) and domain formats
/// - Coordinating calls to domain services
/// - Handling application-specific concerns like message formatting
///
/// The actual business logic is delegated to domain services.
pub struct DocumentApplicationService<R: DocumentRepository> {
    document_service: DocumentService<R>,
}

impl<R: DocumentRepository> DocumentApplicationService<R> {
    /// Creates a new document application service with the provided repository.
    ///
    /// # Arguments
    ///
    /// * `document_repository` - A repository implementation for document storage
    ///
    /// # Returns
    ///
    /// A new `DocumentApplicationService` instance.
    pub fn new(document_repository: R) -> Self {
        Self {
            document_service: DocumentService::new(document_repository),
        }
    }

    /// Handles a client's initial synchronization request.
    ///
    /// This use case orchestrates the sync request by:
    /// 1. Delegating to domain service to establish sync session
    /// 2. Converting binary state vector to Base64 format for external communication
    /// 3. Packaging the result in the expected message format
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Identifier for the document to synchronize with
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// * A `ServerMessage` with the document's current state vector in Base64
    /// * A broadcast receiver for future document updates
    pub async fn handle_sync_request(
        &self,
        doc_id: &str,
    ) -> (ServerMessage, tokio::sync::broadcast::Receiver<Vec<u8>>) {
        let (state_vector, update_receiver) =
            self.document_service.establish_sync_session(doc_id).await;

        // Convert binary state vector to Base64 for external communication
        let response = ServerMessage {
            message_type: "sv".to_string(),
            data: None,
            update: Some(BASE64.encode(&state_vector)),
        };

        (response, update_receiver)
    }

    /// Handles a client's update to a document.
    ///
    /// This use case handles format conversion and delegates to domain service:
    /// 1. Decodes Base64 update data to binary format
    /// 2. Delegates to domain service for actual business logic
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
                self.document_service
                    .apply_document_update(doc_id, &update)
                    .await
            }
            Err(_) => Err("Failed to decode base64 update data".to_string()),
        }
    }

    /// Handles a client's request to synchronize with a document using its state vector.
    ///
    /// This use case coordinates between external format and domain logic:
    /// 1. Decodes Base64 state vector to binary format
    /// 2. Delegates to domain service to compute missing updates
    /// 3. Converts result back to Base64 and packages in message format
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
            Ok(client_state_vector) => {
                match self
                    .document_service
                    .compute_missing_updates(doc_id, &client_state_vector)
                    .await?
                {
                    Some(update) => {
                        // Convert binary update to Base64 and package in message format
                        let response = ServerMessage {
                            message_type: "update".to_string(),
                            data: None,
                            update: Some(BASE64.encode(&update)),
                        };
                        Ok(Some(response))
                    }
                    None => Ok(None), // No updates to sync
                }
            }
            Err(_) => Err("Failed to decode base64 state vector data".to_string()),
        }
    }

    /// Handles a binary update directly (without Base64 decoding).
    ///
    /// This use case is optimized for binary protocols and directly delegates
    /// to the domain service without format conversion.
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
        self.document_service
            .apply_document_update(doc_id, binary_data)
            .await
    }
}

// Type alias for backward compatibility
pub type DocumentUseCases<R> = DocumentApplicationService<R>;
