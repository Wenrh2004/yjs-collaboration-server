use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    Doc, GetString, ReadTxn, StateVector, Transact, Update,
};

/// Represents a collaborative document that multiple clients can edit simultaneously.
///
/// This entity encapsulates a Yjs document (via Yrs' `Doc`) and provides methods for
/// synchronizing changes between connected clients using CRDT operations.
///
/// This is the core domain entity of the collaboration system.
pub struct CollaborativeDocument {
    pub(crate) doc: Doc,
}

impl CollaborativeDocument {
    /// Creates a new, empty collaborative document.
    ///
    /// # Returns
    ///
    /// A new `CollaborativeDocument` instance with an initialized Yjs document.
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    /// Retrieves the document's current state vector.
    ///
    /// The state vector represents the logical clock of all changes incorporated
    /// into the document and is used for synchronization between clients.
    ///
    /// # Returns
    ///
    /// A binary-encoded state vector that can be sent to clients.
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = txn.state_vector();
        sv.encode_v1()
    }

    /// Applies an update to the document.
    ///
    /// This method integrates changes from a client into the document.
    ///
    /// # Arguments
    ///
    /// * `update` - A binary-encoded update from a client
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<u8>)` - The document's new state vector after applying the update
    /// * `Err(String)` - An error message if the update couldn't be applied
    pub fn apply_update(&mut self, update: &[u8]) -> Result<Vec<u8>, String> {
        if let Ok(update) = Update::decode_v1(update) {
            let mut txn = self.doc.transact_mut();

            // Apply update and handle potential errors
            let result = txn.apply_update(update);
            if let Err(e) = result {
                return Err(e.to_string());
            }

            // Get the updated state vector
            Ok(self.get_state_vector())
        } else {
            Err("Failed to decode update".to_string())
        }
    }

    /// Retrieves updates that a client is missing based on its state vector.
    ///
    /// This method computes the difference between the document's current state
    /// and the client's state vector, returning the updates the client needs.
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
        if let Ok(sv) = StateVector::decode_v1(client_state) {
            let txn = self.doc.transact();
            let updates = txn.encode_state_as_update_v2(&sv);
            Ok(updates)
        } else {
            Err("Failed to decode state vector".to_string())
        }
    }

    /// Retrieves the text content of the document.
    ///
    /// This method extracts text content from the Yjs document using the correct Yrs API.
    /// It uses the GetString trait to extract actual text content from TextRef objects.
    ///
    /// # Returns
    ///
    /// The current text content of the document as a String.
    pub fn get_text_content(&self) -> String {
        let txn = self.doc.transact();

        // Try to get the default text object (commonly used root text)
        let text_ref = self.doc.get_or_insert_text("");
        let content = text_ref.get_string(&txn);

        // If the default text is not empty, return it
        if !content.is_empty() {
            return content;
        }

        // If no default text exists, try common field names
        for field_name in ["content", "text", "body", "document"] {
            let text_ref = self.doc.get_or_insert_text(field_name);
            let content = text_ref.get_string(&txn);
            if !content.is_empty() {
                return content;
            }
        }

        // If no text content found, return empty string
        String::new()
    }

    /// Retrieves a simple text representation of the document.
    ///
    /// This method provides a basic text extraction from the Yjs document,
    /// suitable for display or basic content operations.
    ///
    /// # Returns
    ///
    /// A string representing the document's text content.
    pub fn get_content_as_string(&self) -> String {
        // For now, use the text content method
        // In a real implementation, this might extract content from various shared types
        let text_content = self.get_text_content();

        if text_content.is_empty() {
            // If no text content, provide a basic representation
            format!(
                "Document (State Vector Length: {})",
                self.get_state_vector().len()
            )
        } else {
            text_content
        }
    }
}
