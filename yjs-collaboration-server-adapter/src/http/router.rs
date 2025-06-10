use std::sync::Arc;

use volo_http::{server::route::get, Router};
use yjs_collaboration_server_domain::repositories::document_repository::DocumentRepository;
use yjs_collaboration_server_domain::services::document_service::DocumentService;

use crate::http::websocket::ws_handler::handle_websocket_upgrade;

/// HTTP router configuration for the collaboration server.
///
/// This adapter configures and builds the HTTP routes for the collaboration server,
/// integrating the domain services with the HTTP interface.
///
/// It defines:
/// - A health check endpoint to verify server status
/// - A WebSocket endpoint for real-time collaboration
pub struct HttpRouter<R: DocumentRepository> {
    // 直接使用domain层的DocumentService
    document_service: Arc<DocumentService<R>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> HttpRouter<R> {
    /// Creates a new HTTP router with the provided document service.
    ///
    /// # Arguments
    ///
    /// * `document_service` - The domain document service to handle collaboration logic
    ///
    /// # Returns
    ///
    /// A new `HttpRouter` instance.
    pub fn new(document_service: Arc<DocumentService<R>>) -> Self {
        Self { document_service }
    }

    /// Health check handler that returns a simple status message.
    ///
    /// This endpoint can be used to verify that the server is running.
    ///
    /// # Returns
    ///
    /// A static string confirming the server is operational.
    async fn health_handler() -> &'static str {
        "Yjs Collaboration Server Is Health\n"
    }

    /// Builds and configures the HTTP router with all necessary routes.
    ///
    /// This method sets up:
    /// - A root route (`/`) for health checks
    /// - A WebSocket route (`/ws`) for real-time document collaboration
    ///
    /// # Returns
    ///
    /// A configured `Router` instance ready to be used by the HTTP server.
    pub fn build_router(&self) -> Router {
        let document_service = self.document_service.clone();

        Router::new().route("/", get(Self::health_handler)).route(
            "/ws",
            get(move |upgrade| handle_websocket_upgrade(upgrade, document_service.clone())),
        )
    }
}
