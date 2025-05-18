use tokio::sync::broadcast;

use crate::domain::entities::document::CollaborativeDocument;

/// Document service, encapsulates business logic for a single document
pub struct DocumentService {
    document: CollaborativeDocument,
    update_broadcaster: broadcast::Sender<Vec<u8>>,
}

impl DocumentService {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100); // buffer size
        Self {
            document: CollaborativeDocument::new(),
            update_broadcaster: tx,
        }
    }

    /// Get the document's state vector
    pub fn get_state_vector(&self) -> Vec<u8> {
        self.document.get_state_vector()
    }

    /// Apply an update to the document and broadcast it to all connected clients
    pub fn apply_update(&mut self, update: &[u8]) -> Result<(), String> {
        match self.document.apply_update(update) {
            Ok(sv) => {
                // Broadcast the update to all connected clients
                let _ = self.update_broadcaster.send(sv);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Get the updates missing on the client
    pub fn get_missing_updates(&self, client_state: &[u8]) -> Result<Vec<u8>, String> {
        self.document.get_missing_updates(client_state)
    }

    /// Subscribe to document updates
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.update_broadcaster.subscribe()
    }
}
