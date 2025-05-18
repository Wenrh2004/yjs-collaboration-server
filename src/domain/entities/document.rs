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

    /// 获取文档的状态向量
    pub fn get_state_vector(&self) -> Vec<u8> {
        let txn = self.doc.transact();
        let sv = txn.state_vector();
        sv.encode_v1()
    }

    /// 应用更新到文档
    pub fn apply_update(&mut self, update: &[u8]) -> Result<Vec<u8>, String> {
        if let Ok(update) = Update::decode_v1(update) {
            let mut txn = self.doc.transact_mut();

            // 应用更新并处理可能的错误
            if let Err(e) = txn.apply_update(update) {
                return Err(format!("Failed to apply update: {}", e));
            }

            // 获取更新后的状态向量
            let sv = txn.state_vector().encode_v1();
            Ok(sv)
        } else {
            Err("Failed to decode update".to_string())
        }
    }

    /// 获取客户端缺失的更新
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
