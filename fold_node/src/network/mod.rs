mod config;
mod core;
mod error;
mod schema_protocol;
mod schema_service;

pub use config::NetworkConfig;
pub use core::NetworkCore;
pub use error::{NetworkError, NetworkResult};
pub use schema_service::SchemaService;

// Re-export types needed for public API
pub use libp2p::PeerId;
