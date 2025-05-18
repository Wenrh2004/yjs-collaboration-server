use std::sync::Arc;

use volo_http::{server::route::get, Router};

use crate::{
    adapter::websocket::ws_handler::handle_websocket_upgrade,
    application::use_cases::document_use_cases::DocumentUseCases,
    domain::repositories::document_repository::DocumentRepository,
};

// HTTP router configuration
pub struct HttpRouter<R: DocumentRepository> {
    document_use_cases: Arc<DocumentUseCases<R>>,
}

impl<R: DocumentRepository + Send + Sync + 'static> HttpRouter<R> {
    pub fn new(document_use_cases: Arc<DocumentUseCases<R>>) -> Self {
        Self { document_use_cases }
    }

    // Health check
    async fn health_handler() -> &'static str {
        "Yjs Collaboration Server Is Hearth\n"
    }

    // Build the router
    pub fn build_router(&self) -> Router {
        let document_use_cases = self.document_use_cases.clone();

        Router::new().route("/", get(Self::health_handler)).route(
            "/ws",
            get(move |upgrade| handle_websocket_upgrade(upgrade, document_use_cases.clone())),
        )
    }
}
