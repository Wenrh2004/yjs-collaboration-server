use std::{collections::HashMap, sync::Arc};

use futures::StreamExt;
use tokio::sync::{Mutex, mpsc};
use tracing::{error, info, warn};
use volo_gen::collaboration::{
    AwarenessUpdate, ClientMessage, CollaborationService, DocumentState, ErrorMessage, ErrorType,
    GetActiveUsersRequest, GetActiveUsersResponse, GetDocumentStateRequest,
    GetDocumentStateResponse, JoinDocument, LeaveDocument, ServerMessage, SyncResponse,
    UpdateMessage, UserJoined, UserLeft, client_message, server_message,
};
use volo_grpc::{BoxStream, RecvStream, Request, Response, Status};

use crate::{DocumentRepository, DocumentUseCases};

pub struct CollaborationServiceImpl<R: DocumentRepository> {
    document_use_cases: Arc<DocumentUseCases<R>>,
    // 管理活跃连接的会话
    active_sessions: Arc<Mutex<HashMap<String, mpsc::Sender<Result<ServerMessage, Status>>>>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> CollaborationServiceImpl<R> {
    pub fn new(document_use_cases: Arc<DocumentUseCases<R>>) -> Self {
        Self {
            document_use_cases,
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

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
                        .document_use_cases
                        .handle_sync_request(&document_id)
                        .await;

                    let proto_response = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: chrono::Utc::now().timestamp(),
                        message_type: Some(server_message::MessageType::SyncResponse(
                            SyncResponse {
                                update_data: response
                                    .update
                                    .map(|u| base64::decode(&u).unwrap_or_default())
                                    .unwrap_or_default()
                                    .into(),
                                state_vector: sync_req.state_vector,
                            },
                        )),
                    };

                    if let Err(_) = tx.send(Ok(proto_response)).await {
                        warn!("Failed to send sync response to client {}", client_id);
                    }
                }
                client_message::MessageType::Update(update) => {
                    if let Err(e) = self
                        .document_use_cases
                        .handle_binary_update(&document_id, &update.update_data)
                        .await
                    {
                        error!("Failed to handle update: {}", e);
                        let error_msg = ServerMessage {
                            document_id: document_id.into(),
                            timestamp: chrono::Utc::now().timestamp(),
                            message_type: Some(server_message::MessageType::Error(ErrorMessage {
                                error_code: 400,
                                error_message: e.into(),
                                error_type: ErrorType::INVALID_UPDATE,
                            })),
                        };
                        let _ = tx.send(Ok(error_msg)).await;
                    } else {
                        // 广播更新给其他客户端
                        self.broadcast_update(&document_id, &client_id, &update.update_data)
                            .await;
                    }
                }
                client_message::MessageType::JoinDocument(join) => {
                    info!("User {} joined document {}", join.user_id, document_id);

                    // 通知其他用户
                    let user_joined = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: chrono::Utc::now().timestamp(),
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

                    let user_left = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: chrono::Utc::now().timestamp(),
                        message_type: Some(server_message::MessageType::UserLeft(UserLeft {
                            user_id: leave.user_id.clone(),
                            client_id: client_id.clone().into(),
                        })),
                    };

                    self.broadcast_to_document(&document_id, user_left, Some(&client_id))
                        .await;
                }
                client_message::MessageType::Awareness(awareness) => {
                    // 广播感知更新
                    let awareness_msg = ServerMessage {
                        document_id: document_id.clone().into(),
                        timestamp: chrono::Utc::now().timestamp(),
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
                    // 处理心跳，可以更新用户活跃状态
                }
            }
        }

        Ok(())
    }

    async fn broadcast_update(
        &self,
        document_id: &str,
        origin_client_id: &str,
        update_data: &[u8],
    ) {
        let update_msg = ServerMessage {
            document_id: document_id.to_string().into(),
            timestamp: chrono::Utc::now().timestamp(),
            message_type: Some(server_message::MessageType::Update(UpdateMessage {
                update_data: update_data.to_vec().into(),
                origin_client_id: origin_client_id.to_string().into(),
                sequence_number: 0, // 可以实现序列号管理
            })),
        };

        self.broadcast_to_document(document_id, update_msg, Some(origin_client_id))
            .await;
    }

    async fn broadcast_to_document(
        &self,
        document_id: &str,
        message: ServerMessage,
        exclude_client: Option<&str>,
    ) {
        let sessions = self.active_sessions.lock().await;
        for (session_id, sender) in sessions.iter() {
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
}

impl<R: DocumentRepository + Send + Sync + 'static> CollaborationService
    for CollaborationServiceImpl<R>
{
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

                        // 注册会话
                        {
                            let mut sessions = service.active_sessions.lock().await;
                            sessions.insert(session_id.clone(), tx.clone());
                        }

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

    async fn get_document_state(
        &self,
        request: Request<GetDocumentStateRequest>,
    ) -> Result<Response<GetDocumentStateResponse>, Status> {
        let req = request.into_inner();

        // 获取文档状态
        let (response, _) = self
            .document_use_cases
            .handle_sync_request(&req.document_id)
            .await;

        let document_state = DocumentState {
            state_vector: vec![].into(), // TODO: 从response中提取实际的state vector
            document_data: response
                .update
                .map(|u| base64::decode(&u).unwrap_or_default())
                .unwrap_or_default()
                .into(),
            active_users: vec![], // TODO: 实现活跃用户管理
            last_modified: chrono::Utc::now().timestamp(),
        };

        Ok(Response::new(GetDocumentStateResponse {
            document_state: Some(document_state),
        }))
    }

    async fn get_active_users(
        &self,
        request: Request<GetActiveUsersRequest>,
    ) -> Result<Response<GetActiveUsersResponse>, Status> {
        let _req = request.into_inner();

        // TODO: 实现从会话管理中获取活跃用户
        let active_users = vec![];

        Ok(Response::new(GetActiveUsersResponse { active_users }))
    }
}

impl<R: DocumentRepository> Clone for CollaborationServiceImpl<R> {
    fn clone(&self) -> Self {
        Self {
            document_use_cases: Arc::clone(&self.document_use_cases),
            active_sessions: Arc::clone(&self.active_sessions),
        }
    }
}
