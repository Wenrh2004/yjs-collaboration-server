use serde::{Deserialize, Serialize};
use sonic_rs::Value;

/// Message sent from a client to the server.
///
/// This value object represents the structure of messages that clients
/// send to the collaboration server. These messages contain synchronization
/// information, document updates, or other commands.
///
/// The message includes:
/// - A document identifier to specify which document it relates to
/// - A message type to indicate the operation being performed
/// - Optional JSON data for custom information
/// - Optional Base64-encoded binary update data for document changes
#[derive(Debug, Deserialize, Serialize)]
pub struct ClientMessage {
    /// Identifier of the document this message relates to
    pub doc_id: String,

    /// Type of message being sent (e.g., "sync", "update", "sv")
    #[serde(rename = "type")]
    pub message_type: String,

    /// Optional JSON data for additional information or parameters
    pub data: Option<Value>,

    /// Base64-encoded binary update for document modifications
    pub update: Option<String>,
}

/// Message sent from the server to a client.
///
/// This value object represents the structure of messages that the server
/// sends to clients in response to their requests or as part of real-time updates.
///
/// The message includes:
/// - A message type to indicate the kind of response
/// - Optional JSON data for custom information
/// - Optional Base64-encoded binary update data for document changes
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerMessage {
    /// Type of message being sent (e.g., "sv", "update", "error")
    #[serde(rename = "type")]
    pub message_type: String,

    /// Optional JSON data for additional information
    pub data: Option<Value>,

    /// Base64-encoded binary update or state vector
    pub update: Option<String>,
}
