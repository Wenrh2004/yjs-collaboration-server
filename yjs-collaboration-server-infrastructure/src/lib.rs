// Infrastructure layer for the Yjs Collaboration Server
//
// This crate implements the repository interfaces defined in the domain layer,
// providing concrete storage mechanisms and infrastructure services.

pub mod adapters;

// Re-export commonly used infrastructure implementations
pub use adapters::in_memory_document_repository::InMemoryDocumentRepository;
