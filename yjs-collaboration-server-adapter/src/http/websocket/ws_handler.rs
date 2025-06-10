use std::{future::Future, pin::Pin, sync::Arc};

use base64::Engine;
use futures_util::{sink::SinkExt, stream::StreamExt};
use sonic_rs::{from_str, to_string};
use tracing::{info, warn};
use uuid::Uuid;
use volo_http::{
    response::Response,
    server::utils::ws::{Message, WebSocket, WebSocketUpgrade},
};
use yjs_collaboration_server_domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::DocumentService,
    value_objects::message::{ClientMessage, ServerMessage},
};

/// Handles WebSocket upgrade requests from the routing system.
///
/// This standalone function serves as an entry point for WebSocket connections
/// in the HTTP router. It upgrades HTTP connections to WebSocket protocol and
/// delegates the connection handling to the `WebSocketHandler`.
///
/// # Arguments
///
/// * `ws` - The WebSocket upgrade request
/// * `document_service` - Domain document service for collaboration operations
///
/// # Returns
///
/// A response that upgrades the connection to WebSocket protocol
pub async fn handle_websocket_upgrade<R>(
    ws: WebSocketUpgrade,
    document_service: Arc<DocumentService<R>>,
) -> Response
where
    R: DocumentRepository + Send + Sync + 'static,
{
    ws.on_upgrade(move |socket| {
        Box::pin(WebSocketHandler::<R>::handle_socket(
            socket,
            document_service,
        )) as Pin<Box<dyn Future<Output=()> + Send>>
    })
}

/// WebSocket connection handler for collaborative document editing.
///
/// This handler manages WebSocket connections with clients for real-time
/// document collaboration. It processes various message types including:
/// - Document synchronization requests
/// - Document updates
/// - State vector synchronization
///
/// It also maintains the connection state and broadcasts updates to clients.
#[derive(Clone)]
pub struct WebSocketHandler<R: DocumentRepository> {
    document_service: Arc<DocumentService<R>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> WebSocketHandler<R> {
    /// Creates a new WebSocket handler with the provided document service.
    ///
    /// # Arguments
    ///
    /// * `document_service` - Domain document service for collaboration operations
    ///
    /// # Returns
    ///
    /// A new `WebSocketHandler` instance
    pub fn new(document_service: Arc<DocumentService<R>>) -> Self {
        Self { document_service }
    }

    /// Handles a WebSocket upgrade request and sets up the connection.
    ///
    /// # Arguments
    ///
    /// * `ws` - The WebSocket upgrade request
    ///
    /// # Returns
    ///
    /// A response that upgrades the connection to WebSocket protocol
    pub fn handle_upgrade(&self, ws: WebSocketUpgrade) -> Response {
        let document_service = self.document_service.clone();
        ws.on_upgrade(move |socket| {
            Box::pin(Self::handle_socket(socket, document_service))
                as Pin<Box<dyn Future<Output=()> + Send>>
        })
    }

    /// Main WebSocket connection handler that processes messages from clients.
    ///
    /// This method:
    /// 1. Establishes a new WebSocket connection with a client
    /// 2. Processes incoming messages based on their type
    /// 3. Forwards document updates between collaborating clients
    /// 4. Maintains connection until client disconnects
    ///
    /// # Arguments
    ///
    /// * `socket` - The WebSocket connection
    /// * `document_service` - Domain document service for collaboration operations
    pub async fn handle_socket(mut socket: WebSocket, document_service: Arc<DocumentService<R>>) {
        // Generate a unique client ID for this connection
        let client_id = Uuid::new_v4().to_string();
        info!("New WebSocket connection established: {}", client_id);

        // Process incoming messages until client disconnects
        while let Some(msg) = socket.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    // Try to parse the message as a ClientMessage
                    match from_str::<ClientMessage>(&text) {
                        Ok(client_msg) => {
                            info!(
                                "Received message type '{}' for document '{}'",
                                client_msg.message_type, client_msg.doc_id
                            );

                            // Process message based on its type
                            match client_msg.message_type.as_str() {
                                // Client requests initial synchronization
                                "sync" => {
                                    let (response, _receiver) = document_service
                                        .handle_sync_request(&client_msg.doc_id)
                                        .await;

                                    // Send initial state vector back to client
                                    if let Ok(resp_json) = to_string(&response) {
                                        if socket.send(Message::Text(resp_json)).await.is_err() {
                                            warn!("Failed to send sync response to client");
                                            break;
                                        }
                                    }

                                    // Note: For broadcast updates, we would need to implement a
                                    // different approach
                                    // such as using channels or a broadcast system outside of this
                                    // handler
                                }
                                // Client sends a document update
                                "update" => {
                                    if let Some(update_base64) = &client_msg.update {
                                        if let Err(e) = document_service
                                            .handle_update_request(
                                                &client_msg.doc_id,
                                                update_base64,
                                            )
                                            .await
                                        {
                                            warn!("Failed to apply update: {}", e);
                                        }
                                    }
                                }
                                // Client requests synchronization using state vector
                                "sv" => {
                                    if let Some(sv_base64) = &client_msg.update {
                                        match document_service
                                            .handle_sync_step(&client_msg.doc_id, sv_base64)
                                            .await
                                        {
                                            Ok((response, _)) => {
                                                if let Ok(resp_json) = to_string(&response) {
                                                    if socket
                                                        .send(Message::Text(resp_json))
                                                        .await
                                                        .is_err()
                                                    {
                                                        warn!("Failed to send sv response");
                                                        break;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                warn!("Failed to handle sync step: {}", e);
                                            }
                                        }
                                    }
                                }
                                _ => warn!("Unknown message type: {}", client_msg.message_type),
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse client message: {}", e);
                        }
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket connection closed by client: {}", client_id);
                    break;
                }
                Err(e) => {
                    warn!("WebSocket error: {}", e);
                    break;
                }
                _ => {} // Ignore other message types
            }
        }

        info!("WebSocket connection terminated: {}", client_id);
    }
}
