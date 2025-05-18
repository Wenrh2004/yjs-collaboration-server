use serde::{Deserialize, Serialize};
use sonic_rs::Value;

// Message sent by the client
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientMessage {
    pub doc_id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: Option<Value>,
    pub update: Option<String>, // base64-encoded update
}

// Message sent by the server
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerMessage {
    #[serde(rename = "type")]
    pub message_type: String,
    pub data: Option<Value>,
    pub update: Option<String>, // base64-encoded update
}
