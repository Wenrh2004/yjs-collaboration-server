//! # Volo HTTP Yjs Server
//!
//! This crate provides a collaborative editing server built with Volo HTTP and Yrs (the Rust implementation of Yjs).
//! Organized according to Domain-Driven Design (DDD) and Clean Architecture principles.

// Export all modules
pub mod adapter;
pub mod application;
pub mod domain;
pub mod infrastructure;

// Re-export main types for external use
use std::sync::Arc;

pub use adapter::http::router::HttpRouter;
pub use application::use_cases::document_use_cases::DocumentUseCases;
pub use domain::repositories::document_repository::DocumentRepository;
pub use infrastructure::adapters::in_memory_document_repository::InMemoryDocumentRepository;
use volo_http::Router;

/// Create the default router using the in-memory document repository
pub fn create_router() -> Router {
    // Create repository
    let repository = InMemoryDocumentRepository::new();

    // Create use case service
    let document_use_cases = Arc::new(DocumentUseCases::new(repository));

    // Create HTTP router
    let http_router = HttpRouter::new(document_use_cases);

    // Build and return the router
    http_router.build_router()
}
