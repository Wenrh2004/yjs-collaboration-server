use tokio::sync::broadcast;

use crate::domain::entities::document::CollaborativeDocument;

/// A domain service that manages a single collaborative document and its associated updates.
///
/// This service wraps a `CollaborativeDocument` entity and provides the following capabilities:
/// - Maintains the document state
/// - Handles document updates
/// - Broadcasts updates to connected clients via a publish-subscribe channel
/// - Provides synchronization mechanisms for clients to catch up
///
/// The service acts as an intermediary between the repository layer and the document entity.
pub struct DocumentService {
    document: CollaborativeDocument,
    update_broadcaster: broadcast::Sender<Vec<u8>>,
}

impl DocumentService {
    /// Creates a new document service with an empty document and broadcast channel.
    ///
    /// # Returns
    ///
    /// A new `DocumentService` instance with an initialized document and broadcast channel.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100); // buffer size for 100 updates
        Self {
            document: CollaborativeDocument::new(),
            update_broadcaster: tx,
        }
    }

    /// Retrieves the document's current state vector.
    ///
    /// # Returns
    ///
    /// A binary-encoded state vector that represents the current document state.
    pub fn get_state_vector(&self) -> Vec<u8> {
        self.document.get_state_vector()
    }

    /// Applies an update to the document and broadcasts it to all connected clients.
    ///
    /// # Arguments
    ///
    /// * `update` - A binary-encoded update to apply to the document
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the update was successfully applied and broadcasted
    /// * `Err(String)` - An error message if the update couldn't be applied
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

    /// Retrieves updates that a client is missing based on its state vector.
    ///
    /// # Arguments
    ///
    /// * `client_state` - A binary-encoded state vector from the client
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - Binary-encoded updates the client needs to apply
    /// * `Err(String)` - An error message if the client state couldn't be processed
    pub fn get_missing_updates(&self, client_state: &[u8]) -> Result<Vec<u8>, String> {
        self.document.get_missing_updates(client_state)
    }

    /// Creates a subscription to document updates.
    ///
    /// Clients can use this to receive real-time notifications when the document changes.
    ///
    /// # Returns
    ///
    /// A broadcast receiver that will receive document state vector updates.
    pub fn subscribe(&self) -> broadcast::Receiver<Vec<u8>> {
        self.update_broadcaster.subscribe()
    }
}
