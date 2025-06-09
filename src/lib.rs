//! # Yjs Collaboration Server
//!
//! This crate provides a collaborative editing server built with Volo HTTP and Yrs (the Rust
//! implementation of Yjs). Organized according to Domain-Driven Design (DDD) and Clean Architecture
//! principles.
//!
//! ## Architecture
//!
//! - `adapter`: HTTP and RPC adapters for external communication
//! - `application`: Business use cases and application services
//! - `domain`: Core domain logic, entities and interfaces
//! - `infrastructure`: Implementation details like repositories

// Export all modules
pub mod adapter;
pub mod application;
pub mod domain;
pub mod infrastructure;
