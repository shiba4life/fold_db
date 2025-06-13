//! A DataFold node is a self-contained instance that can store data, process
//! queries and mutations, and communicate with other nodes. Each node has:
//!
//! 1. A local database for storing data
//! 2. A schema system for defining data structure
//! 3. A network layer for communicating with other nodes
//! 4. A TCP server for external client connections
//!
//! Nodes can operate independently or as part of a network, with trust
//! relationships defining how they share and access data.

pub mod config;
pub mod crypto_init;
pub mod crypto_routes;
pub mod crypto_validation;
mod db;
pub mod encryption_at_rest;
pub mod encryption_at_rest_async;
pub mod error;
pub mod http_server;
pub mod key_cache_manager;
pub mod key_rotation_compliance;
pub mod key_rotation_routes;
pub mod log_routes;
pub mod network_routes;
pub mod node;
mod permissions;
pub mod query_routes;
pub mod schema_routes;
pub mod signature_auth;
pub mod system_routes;
pub mod tcp_command_router;
pub mod tcp_connections;
pub mod tcp_protocol;
pub mod tcp_server;
pub mod tests;
mod transform_queue;

// Re-export the DataFoldNode struct for easier imports
pub use config::load_node_config;
pub use config::NodeConfig;
pub use crypto_init::{
    get_crypto_init_status, initialize_database_crypto, is_crypto_init_needed, CryptoInitContext,
    CryptoInitError, CryptoInitResult, CryptoInitStatus,
};
pub use crypto_validation::{
    validate_crypto_config_comprehensive, validate_crypto_config_quick,
    validate_for_database_creation,
};
pub use http_server::DataFoldHttpServer;
pub use node::DataFoldNode;
pub use tcp_server::TcpServer;
