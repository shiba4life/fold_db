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

// Configuration and top-level modules
pub mod config;
pub mod error;
pub mod tests;

// Modular submodules
pub mod core;
pub mod crypto;
pub mod transport;
pub mod routes;
pub mod auth;
pub mod monitoring;

// Re-export key types for easier imports
pub use config::{load_node_config, NodeConfig};
pub use core::DataFoldNode;
pub use crypto::{
    get_crypto_init_status, initialize_database_crypto, is_crypto_init_needed, 
    CryptoInitContext, CryptoInitError, CryptoInitStatus,
    validate_crypto_config_comprehensive, validate_crypto_config_quick,
    validate_for_database_creation,
};
pub use routes::{DataFoldHttpServer, AppState};
pub use transport::TcpServer;
pub use auth::{SignatureAuthConfig, SignatureVerificationState, SignatureVerificationMiddleware};
pub use monitoring::{PerformanceMetrics, SystemHealthStatus};

// Re-export crypto modules for backward compatibility
pub use crypto::{encryption_at_rest, encryption_at_rest_async, key_cache_manager, crypto_init, crypto_routes};

// Re-export auth module for backward compatibility
pub use auth::signature_auth;

// Re-export routes for backward compatibility
pub use routes::{http_server, system_routes};

// Re-export transport modules for backward compatibility
pub use transport::tcp_protocol;

// Add missing CryptoInitResult re-export
pub use crypto::crypto_init::CryptoInitResult;
