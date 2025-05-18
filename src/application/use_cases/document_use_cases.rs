use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

use crate::domain::{
    repositories::document_repository::DocumentRepository, value_objects::message::ServerMessage,
};

// Document use case service, contains all business logic related to document interaction
pub struct DocumentUseCases<R: DocumentRepository> {
    document_repository: R,
}

impl<R: DocumentRepository> DocumentUseCases<R> {
    pub fn new(document_repository: R) -> Self {
        Self {
            document_repository,
        }
    }

    // Handle client's sync request
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

    // Handle client's update request
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

    // Handle client's state vector request
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

    // Handle binary update request
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
