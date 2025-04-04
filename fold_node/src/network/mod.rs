mod core;
mod schema_protocol;
mod schema_service;
mod error;
mod config;

pub use core::NetworkCore;
pub use schema_service::SchemaService;
pub use error::{NetworkError, NetworkResult};
pub use config::NetworkConfig;

// Re-export types needed for public API
pub use libp2p::PeerId;
