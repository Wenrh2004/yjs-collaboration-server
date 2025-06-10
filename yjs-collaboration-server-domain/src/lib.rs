// Domain layer for the Yjs Collaboration Server
//
// This crate contains the core business logic, domain entities, value objects,
// and repository interfaces of the Yjs Collaboration Server.

pub mod entities;
pub mod repositories;
pub mod services;
pub mod value_objects;

// Re-export commonly used domain types
pub use entities::document::CollaborativeDocument;
pub use services::document_service::SingleDocumentServiceImpl;
