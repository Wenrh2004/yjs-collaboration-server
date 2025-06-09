use yrs::{
    updates::{decoder::Decode, encoder::Encode},
    Doc, ReadTxn, StateVector, Transact, Update,
};

// Core domain entity: collaborative document
pub struct CollaborativeDocument {
    pub(crate) doc: Doc,
}

impl CollaborativeDocument {
    pub fn new() -> Self {
        Self { doc: Doc::new() }
    }

    /// Get the document's state vector
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = txn.state_vector();
        sv.encode_v1()
    }

    /// Apply updates to the document
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

    /// Get missing updates for the client
    pub fn get_missing_updates(&self, client_state: &[u8]) -> Result<Vec<u8>, String> {
        if let Ok(sv) = StateVector::decode_v1(client_state) {
            let txn = self.doc.transact();
            let updates = txn.encode_state_as_update_v1(&sv);
            Ok(updates)
        } else {
            Err("Failed to decode state vector".to_string())
        }
    }
}
