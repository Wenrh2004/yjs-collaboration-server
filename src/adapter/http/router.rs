use std::sync::Arc;

use volo_http::{Router, server::route::get};

use crate::{
    adapter::http::websocket::ws_handler::handle_websocket_upgrade,
    application::services::document_application_service::DocumentUseCases,
    domain::repositories::document_repository::DocumentRepository,
};

/// HTTP router configuration for the collaboration server.
///
/// This adapter configures and builds the HTTP routes for the collaboration server,
/// integrating the application's use cases with the HTTP interface.
///
/// It defines:
/// - A health check endpoint to verify server status
/// - A WebSocket endpoint for real-time collaboration
pub struct HttpRouter<R: DocumentRepository> {
    document_use_cases: Arc<DocumentUseCases<R>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> HttpRouter<R> {
    /// Creates a new HTTP router with the provided document use cases.
    ///
    /// # Arguments
    ///
    /// * `document_use_cases` - The document use cases service to handle collaboration logic
    ///
    /// # Returns
    ///
    /// A new `HttpRouter` instance.
    pub fn new(document_use_cases: Arc<DocumentUseCases<R>>) -> Self {
        Self { document_use_cases }
    }

    /// Health check handler that returns a simple status message.
    ///
    /// This endpoint can be used to verify that the server is running.
    ///
    /// # Returns
    ///
    /// A static string confirming the server is operational.
    async fn health_handler() -> &'static str {
        "Yjs Collaboration Server Is Hearth\n"
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
        let document_use_cases = self.document_use_cases.clone();

        Router::new().route("/", get(Self::health_handler)).route(
            "/ws",
            get(move |upgrade| handle_websocket_upgrade(upgrade, document_use_cases.clone())),
        )
    }
}
