use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use chrono::Utc;
use dashmap::DashMap;
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use volo_grpc::{BoxStream, RecvStream, Request, Response, Status};
use yjs_collaboration_server_common::volo_gen::collaboration::{
    client_message, server_message, ActiveUser, AwarenessUpdate, ClientMessage,
    CollaborationService, DocumentState, ErrorMessage, ErrorType, GetActiveUsersRequest,
    GetActiveUsersResponse, GetDocumentStateRequest, GetDocumentStateResponse, ServerMessage,
    SyncResponse as ProtoSyncResponse, UpdateMessage, UserJoined, UserLeft,
};
use yjs_collaboration_server_domain::{
    repositories::document_repository::DocumentRepository,
    services::document_service::DocumentService,
};

/// User session information for tracking active users
#[derive(Clone, Debug)]
struct UserSession {
    user_id: String,
    user_name: String,
    user_color: String,
    client_id: String,
    document_id: String,
    last_seen: i64,
    user_metadata: std::collections::HashMap<String, String>,
    sender: mpsc::Sender<Result<ServerMessage, Status>>,
}

/// Implementation of the Yjs collaboration gRPC service.
///
/// This struct handles client connections, manages active sessions,
/// and provides real-time collaboration features for documents including
/// synchronization, updates, and user presence notifications.
pub struct CollaborationServiceImpl<R: DocumentRepository> {
    /// Document service handling core business logic for documents
    document_service: Arc<DocumentService<R>>,
    /// Manages active connection sessions with session ID as key and message sender channel as
    /// value Using DashMap for improved concurrent performance compared to Mutex<HashMap>
    active_sessions: Arc<DashMap<String, mpsc::Sender<Result<ServerMessage, Status>>>>,
    /// Tracks active user sessions with detailed user information
    user_sessions: Arc<DashMap<String, UserSession>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> CollaborationServiceImpl<R> {
    /// Creates a new collaboration service instance.
    ///
    /// # Parameters
    ///
    /// * `document_service` - An Arc reference to document service
    ///
    /// # Returns
    ///
    /// A new instance of `CollaborationServiceImpl`
    pub fn new(document_service: Arc<DocumentService<R>>) -> Self {
        Self {
            document_service,
            active_sessions: Arc::new(DashMap::new()),
            user_sessions: Arc::new(DashMap::new()),
        }
    }

    /// Handles messages received from clients.
    ///
    /// Processes different message types such as sync requests, document updates,
    /// or users joining a document.
    ///
    /// # Parameters
    ///
    /// * `client_msg` - The message received from the client
    /// * `tx` - Channel for sending responses back to the client
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure with appropriate status
    ///
    /// # Errors
    ///
    /// Returns a gRPC Status error if message processing fails
    async fn handle_client_message(
        &self,
        client_msg: ClientMessage,
        tx: &mpsc::Sender<Result<ServerMessage, Status>>,
    ) -> Result<(), Status> {
        let client_id = client_msg.client_id.to_string();
        let document_id = client_msg.document_id.to_string();

        if let Some(message_type) = client_msg.message_type {
            match message_type {
                client_message::MessageType::SyncRequest(sync_req) => {
                    let (response, _) = self
                        .document_service
                        .handle_sync_request(&document_id, Some(&sync_req.state_vector.to_vec()))
                        .await;

                    let proto_response = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: Utc::now().timestamp(),
                        message_type: Some(server_message::MessageType::SyncResponse(
                            ProtoSyncResponse {
                                update_data: response.update.unwrap_or_default().into(),
                            },
                        )),
                    };

                    if tx.send(Ok(proto_response)).await.is_err() {
                        warn!("Failed to send sync response to client {}", client_id);
                    }
                }
                client_message::MessageType::Update(update) => {
                    if let Err(e) = self
                        .document_service
                        .handle_binary_update(&document_id, &update.update_data)
                        .await
                    {
                        error!("Failed to handle update: {}", e);
                        let error_msg = ServerMessage {
                            document_id: document_id.into(),
                            timestamp: Utc::now().timestamp(),
                            message_type: Some(server_message::MessageType::Error(ErrorMessage {
                                error_code: 400,
                                error_message: e.into(),
                                error_type: ErrorType::INVALID_UPDATE,
                            })),
                        };
                        let _ = tx.send(Ok(error_msg)).await;
                    } else {
                        // Broadcast update to other clients
                        self.broadcast_update(&document_id, &client_id, &update.update_data)
                            .await;
                    }
                }
                client_message::MessageType::JoinDocument(join) => {
                    info!("User {} joined document {}", join.user_id, document_id);

                    // Create user session
                    let session_id = format!("{}_{}", document_id, client_id);
                    let user_session = UserSession {
                        user_id: join.user_id.to_string(),
                        user_name: join.user_name.to_string(),
                        user_color: join.user_color.to_string(),
                        client_id: client_id.to_string(),
                        document_id: document_id.to_string(),
                        last_seen: Utc::now().timestamp(),
                        user_metadata: join
                            .user_metadata
                            .iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                        sender: tx.clone(),
                    };

                    self.update_user_session(session_id, user_session);

                    // Notify other users
                    let user_joined = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: Utc::now().timestamp(),
                        message_type: Some(server_message::MessageType::UserJoined(UserJoined {
                            user_id: join.user_id.clone(),
                            user_name: join.user_name.clone(),
                            user_color: join.user_color.clone(),
                            client_id: client_id.clone().into(),
                            user_metadata: join.user_metadata.clone(),
                        })),
                    };

                    self.broadcast_to_document(&document_id, user_joined, Some(&client_id))
                        .await;
                }
                client_message::MessageType::LeaveDocument(leave) => {
                    info!("User {} left document {}", leave.user_id, document_id);

                    // Remove user session
                    let session_id = format!("{}_{}", document_id, client_id);
                    self.remove_user_session(&session_id);

                    let user_left = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: Utc::now().timestamp(),
                        message_type: Some(server_message::MessageType::UserLeft(UserLeft {
                            user_id: leave.user_id,
                            client_id: client_id.clone().into(),
                        })),
                    };

                    self.broadcast_to_document(&document_id, user_left, Some(&client_id))
                        .await;
                }
                client_message::MessageType::Awareness(awareness) => {
                    // Broadcast awareness update
                    let awareness_msg = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: Utc::now().timestamp(),
                        // Handle heartbeat, update user activity status
                        message_type: Some(server_message::MessageType::Awareness(
                            AwarenessUpdate {
                                client_id: awareness.client_id.clone(),
                                user_info: awareness.user_info.clone(),
                                awareness_state: awareness.awareness_state.clone(),
                                timestamp: awareness.timestamp,
                            },
                        )),
                    };

                    self.broadcast_to_document(&document_id, awareness_msg, Some(&client_id))
                        .await;
                }
                client_message::MessageType::Heartbeat(_) => {
                    // 处理心跳，更新用户活跃状态
                }
            }
        }

        Ok(())
    }

    /// Broadcasts document update messages to other clients.
    ///
    /// # Parameters
    ///
    /// * `document_id` - Unique identifier for the document
    /// * `origin_client_id` - ID of the client that sent the update
    /// * `update_data` - The update data content
    async fn broadcast_update(
        &self,
        document_id: &str,
        origin_client_id: &str,
        update_data: &[u8],
    ) {
        let update_msg = ServerMessage {
            document_id: document_id.to_string().into(),
            timestamp: Utc::now().timestamp(),
            message_type: Some(server_message::MessageType::Update(UpdateMessage {
                // Sequence numbers can be implemented
                sequence_number: 0,
                update_data: update_data.to_vec().into(),
                origin_client_id: origin_client_id.to_string().into(),
            })),
        };
        self.broadcast_to_document(document_id, update_msg, Some(origin_client_id))
            .await;
    }

    /// Broadcasts a message to all active sessions for a document.
    ///
    /// # Parameters
    ///
    /// * `document_id` - Unique identifier for the document
    /// * `message` - The message to broadcast
    /// * `exclude_client` - Optional client ID to exclude from broadcast
    async fn broadcast_to_document(
        &self,
        document_id: &str,
        message: ServerMessage,
        exclude_client: Option<&str>,
    ) {
        // With DashMap, we can iterate over entries without locking the entire map
        for entry in self.active_sessions.iter() {
            let session_id = entry.key();
            let sender = entry.value();

            if let Some(exclude) = exclude_client {
                if session_id.contains(exclude) {
                    continue;
                }
            }

            if session_id.contains(document_id) {
                if let Err(_) = sender.send(Ok(message.clone())).await {
                    warn!("Failed to send message to session {}", session_id);
                }
            }
        }
    }

    /// Gets active users for a specific document.
    ///
    /// # Parameters
    ///
    /// * `document_id` - Unique identifier for the document
    ///
    /// # Returns
    ///
    /// Vector of ActiveUser structs representing users currently active in the document
    fn get_active_users_for_document(&self, document_id: &str) -> Vec<ActiveUser> {
        self.user_sessions
            .iter()
            .filter_map(|entry| {
                let user_session = entry.value();
                if user_session.document_id == document_id {
                    Some(ActiveUser {
                        user_id: user_session.user_id.clone().into(),
                        user_name: user_session.user_name.clone().into(),
                        user_color: user_session.user_color.clone().into(),
                        client_id: user_session.client_id.clone().into(),
                        last_seen: user_session.last_seen,
                        user_metadata: user_session
                            .user_metadata
                            .iter()
                            .map(|(k, v)| (k.clone().into(), v.clone().into()))
                            .collect(),
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Updates user session information.
    ///
    /// # Parameters
    ///
    /// * `session_id` - Unique session identifier
    /// * `user_session` - User session data to store
    fn update_user_session(&self, session_id: String, user_session: UserSession) {
        self.user_sessions.insert(session_id, user_session);
    }

    /// Removes a user session.
    ///
    /// # Parameters
    ///
    /// * `session_id` - Unique session identifier to remove
    fn remove_user_session(&self, session_id: &str) {
        self.user_sessions.remove(session_id);
    }
}

impl<R: DocumentRepository + Send + Sync + 'static> CollaborationService
    for CollaborationServiceImpl<R>
{
    /// Handles collaboration requests from clients.
    ///
    /// Establishes a bidirectional streaming connection for real-time collaboration.
    ///
    /// # Parameters
    ///
    /// * `request` - Request object containing a stream of client messages
    ///
    /// # Returns
    ///
    /// A response object containing a stream for sending server messages to the client
    ///
    /// # Errors
    ///
    /// Returns a gRPC Status error if the collaboration session cannot be established
    async fn collaborate(
        &self,
        request: Request<RecvStream<ClientMessage>>,
    ) -> Result<Response<BoxStream<'static, Result<ServerMessage, Status>>>, Status> {
        let mut stream = request.into_inner();
        let (tx, mut rx) = mpsc::channel(100);

        let service = self.clone();
        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(msg) => {
                        let session_id = format!("{}_{}", msg.document_id, msg.client_id);

                        // Register session - with DashMap, no explicit locking needed
                        service
                            .active_sessions
                            .insert(session_id.clone(), tx.clone());

                        if let Err(e) = service.handle_client_message(msg, &tx).await {
                            error!("Error handling client message: {:?}", e);
                            let _ = tx.send(Err(e)).await;
                        }
                    }
                    Err(e) => {
                        error!("Error receiving client message: {:?}", e);
                        let _ = tx.send(Err(Status::internal("Stream error"))).await;
                        break;
                    }
                }
            }
        });

        let output_stream = async_stream::stream! {
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        };

        Ok(Response::new(Box::pin(output_stream)))
    }

    /// Gets the current state of a document.
    ///
    /// # Parameters
    ///
    /// * `request` - Request containing the document ID
    ///
    /// # Returns
    ///
    /// A response containing the current document state
    ///
    /// # Errors
    ///
    /// Returns a gRPC Status error if document state retrieval fails
    async fn get_document_state(
        &self,
        request: Request<GetDocumentStateRequest>,
    ) -> Result<Response<GetDocumentStateResponse>, Status> {
        let req = request.into_inner();

        // 获取文档状态
        let (response, _) = self
            .document_service
            .handle_sync_request(&req.document_id, None)
            .await;

        let document_state = DocumentState {
            state_vector: response.state_vector.unwrap_or_default().into(),
            document_data: response
                .update
                .as_ref()
                .map(|u| STANDARD.decode(&u).unwrap_or_default())
                .unwrap_or_default()
                .into(),
            active_users: self.get_active_users_for_document(&req.document_id),
            last_modified: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(GetDocumentStateResponse {
            document_state: Some(document_state),
        }))
    }

    /// Gets the list of currently active users.
    ///
    /// # Parameters
    ///
    /// * `request` - Request containing query parameters
    ///
    /// # Returns
    ///
    /// A response containing the list of active users
    ///
    /// # Errors
    ///
    /// Returns a gRPC Status error if user information retrieval fails
    async fn get_active_users(
        &self,
        request: Request<GetActiveUsersRequest>,
    ) -> Result<Response<GetActiveUsersResponse>, Status> {
        let req = request.into_inner();

        let active_users = self.get_active_users_for_document(&req.document_id);

        Ok(Response::new(GetActiveUsersResponse { active_users }))
    }
}

/// Implementation of Clone for CollaborationServiceImpl
impl<R: DocumentRepository> Clone for CollaborationServiceImpl<R> {
    /// Creates a clone of this collaboration service instance.
    ///
    /// # Returns
    ///
    /// A new `CollaborationServiceImpl` instance with the same references
    fn clone(&self) -> Self {
        Self {
            document_service: Arc::clone(&self.document_service),
            active_sessions: Arc::clone(&self.active_sessions),
            user_sessions: Arc::clone(&self.user_sessions),
        }
    }
}
