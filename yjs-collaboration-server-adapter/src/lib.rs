// Adapter layer for the Yjs Collaboration Server
//
// This crate contains HTTP and RPC adapters that provide external interfaces
// to the application's functionality, translating between external formats and
// the application's internal models.

pub mod http;
pub mod rpc;