// Application layer for the Yjs Collaboration Server
//
// This crate contains application services, use cases, and orchestration logic
// that coordinates the domain objects and infrastructure services.

pub mod bootstrap;
pub mod config;
pub mod container;
pub mod servers;
pub mod services;

// Re-export commonly used application types
pub use bootstrap::ApplicationBootstrap;
pub use config::AppConfig;
pub use services::document_application_service::DocumentUseCases;
