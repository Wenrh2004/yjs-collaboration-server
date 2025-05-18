use std::{future::Future, pin::Pin, sync::Arc};

use base64::Engine;
use futures_util::{sink::SinkExt, stream::StreamExt};
use sonic_rs::{from_str, to_string};
use tracing::{info, warn};
use uuid::Uuid;
use volo_http::{
    response::ServerResponse,
    server::utils::{Message, WebSocket, WebSocketUpgrade},
};

use crate::{
    application::use_cases::document_use_cases::DocumentUseCases,
    domain::{
        repositories::document_repository::DocumentRepository,
        value_objects::message::{ClientMessage, ServerMessage},
    },
};

// Standalone WebSocket handler function for the routing system
pub async fn handle_websocket_upgrade<R>(
    ws: WebSocketUpgrade,
    document_use_cases: Arc<DocumentUseCases<R>>,
) -> ServerResponse
where
    R: DocumentRepository + Send + Sync + 'static,
{
    ws.on_upgrade(move |socket| {
        Box::pin(WebSocketHandler::<R>::handle_socket(
            socket,
            document_use_cases,
        )) as Pin<Box<dyn Future<Output = ()> + Send>>
    })
}

// WebSocket connection handler
#[derive(Clone)]
pub struct WebSocketHandler<R: DocumentRepository> {
    document_use_cases: Arc<DocumentUseCases<R>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> WebSocketHandler<R> {
    pub fn new(document_use_cases: Arc<DocumentUseCases<R>>) -> Self {
        Self { document_use_cases }
    }

    // Handle WebSocket upgrade request
    pub fn handle_upgrade(&self, ws: WebSocketUpgrade) -> ServerResponse {
        let document_use_cases = self.document_use_cases.clone();
        ws.on_upgrade(move |socket| {
            Box::pin(Self::handle_socket(socket, document_use_cases))
                as Pin<Box<dyn Future<Output = ()> + Send>>
        })
    }

    // Handle WebSocket connection
    async fn handle_socket(mut socket: WebSocket, document_use_cases: Arc<DocumentUseCases<R>>) {
        let client_id = Uuid::new_v4().to_string();
        let mut active_doc_id: Option<String> = None;
        let mut update_receiver = None;

        info!("Client {} connected", client_id);

        // Main WebSocket message processing loop
        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Text(ref text) => {
                    // Attempt to parse client message
                    if let Ok(client_msg) = from_str::<ClientMessage>(text) {
                        // Handle based on message type
                        match client_msg.message_type.as_str() {
                            "sync" => {
                                // Client requests document synchronization
                                let doc_id = client_msg.doc_id.clone();
                                active_doc_id = Some(doc_id.clone());

                                // Handle sync request
                                let (response, receiver) =
                                    document_use_cases.handle_sync_request(&doc_id).await;
                                update_receiver = Some(receiver);

                                // Send response
                                if let Ok(json) = to_string(&response) {
                                    let _ = socket.send(Message::Text(json)).await;
                                }
                            }
                            "update" => {
                                // Client sends an update
                                if let (Some(doc_id), Some(update_b64)) =
                                    (&active_doc_id, &client_msg.update)
                                {
                                    let _ = document_use_cases
                                        .handle_update_request(doc_id, update_b64)
                                        .await;
                                }
                            }
                            "sv" => {
                                // Client sends state vector to retrieve missing updates
                                if let (Some(doc_id), Some(sv_b64)) =
                                    (&active_doc_id, &client_msg.update)
                                {
                                    if let Ok(Some(response)) = document_use_cases
                                        .handle_state_vector_request(doc_id, sv_b64)
                                        .await
                                    {
                                        if let Ok(json) = to_string(&response) {
                                            let _ = socket.send(Message::Text(json)).await;
                                        }
                                    }
                                }
                            }
                            _ => {
                                warn!("Unknown message type: {}", client_msg.message_type);
                            }
                        }
                    } else {
                        // Non-Yjs message, return as-is
                        socket.send(msg.clone()).await.unwrap();
                    }
                }
                Message::Binary(bin_data) => {
                    // Process binary message (possibly raw update)
                    if let Some(doc_id) = &active_doc_id {
                        let _ = document_use_cases
                            .handle_binary_update(doc_id, &bin_data)
                            .await;
                    }
                }
                _ => {}
            }

            // Check for updates from other clients
            if let Some(receiver) = &mut update_receiver {
                if let Ok(update) = receiver.try_recv() {
                    // Create update message
                    let response = ServerMessage {
                        message_type: "update".to_string(),
                        data: None,
                        update: Some(base64::engine::general_purpose::STANDARD.encode(&update)),
                    };

                    // Send update to client
                    if let Ok(json) = to_string(&response) {
                        let _ = socket.send(Message::Text(json)).await;
                    }
                }
            }
        }

        // Clean up when client disconnects
        info!("Client {} disconnected", client_id);
    }
}
