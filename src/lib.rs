//! # DataFold Node Library
//!
//! This library implements the core functionality of the DataFold distributed data platform.
//! It provides a schema-based data storage and query system with distributed networking capabilities.
//!
//! ## Core Components
//!
//! * `atom` - Atomic data storage units that form the foundation of the database
//! * `datafold_node` - Node implementation with TCP server and configuration
//! * `db_operations` - Database operation handlers
//! * `error` - Error types and handling
//! * `fees` - Payment and fee calculation system
//! * `fold_db_core` - Core database functionality
//! * `network` - P2P networking layer for node communication
//! * `permissions` - Access control and permission management
//! * `schema` - Schema definition, validation, and execution
//!
//! ## Architecture
//!
//! DataFold uses a distributed architecture where each node can store and process data
//! according to defined schemas. Nodes can communicate with each other to share and
//! replicate data, with permissions controlling access to different schemas and operations.
//!
//! The system is built around the concept of schemas that define the structure of data
//! and the operations that can be performed on it. Each schema has fields with associated
//! permissions and payment requirements.

pub mod atom;
pub mod cli;
pub mod config;
pub mod config_utils;
pub mod crypto;
pub mod datafold_node;
pub mod db_operations;
pub mod error;
pub mod error_handling;
pub mod events;
pub mod fees;
pub mod fold_db_core;
pub mod ingestion;
pub mod logging;
pub mod network;
pub mod permissions;
pub mod schema;
pub mod testing_utils;
pub mod transform;
pub mod validation_utils;
pub mod web_logger;

pub mod tests;

// Re-export main types for convenience
pub use datafold_node::config::load_node_config;
pub use datafold_node::config::NodeConfig;
pub use datafold_node::DataFoldNode;
pub use error::{FoldDbError, FoldDbResult};
pub use fold_db_core::FoldDB;
pub use network::{NetworkConfig, NetworkCore, NetworkError, NetworkResult, PeerId, SchemaService};

// Re-export schema types needed for CLI
pub use schema::core::SchemaState;
pub use schema::types::operation::Operation;
pub use schema::types::operations::MutationType;
pub use schema::Schema;

// Re-export ingestion types
pub use ingestion::{IngestionConfig, IngestionCore, IngestionError, IngestionResponse};

// Re-export crypto types
pub use crypto::{
    derive_master_keypair, derive_master_keypair_default, generate_master_keypair, generate_salt,
    generate_salt_and_derive_keypair, Argon2Params, CryptoError, CryptoResult, DerivedKey,
    MasterKeyPair, PublicKey, Salt,
};

// Re-export config types
pub use config::crypto::{
    ConfigError, CryptoConfig, KeyDerivationConfig, MasterKeyConfig, SecurityLevel,
};

// Re-export database crypto metadata types
pub use db_operations::crypto_metadata::CryptoMetadata;

// Re-export crypto initialization types
pub use datafold_node::{
    get_crypto_init_status, initialize_database_crypto, is_crypto_init_needed,
    validate_crypto_config_comprehensive, validate_crypto_config_quick,
    validate_for_database_creation, CryptoInitContext, CryptoInitError, CryptoInitResult,
    CryptoInitStatus,
};

// Re-export CLI types for authentication and signing
pub use cli::{
    auth::{CliAuthProfile, CliAuthStatus, CliRequestSigner, CliSigningConfig},
    config::{CliConfig, CliConfigManager, CliSettings, ServerConfig},
    http_client::{AuthenticatedHttpClient, HttpClientBuilder, RetryConfig},
    signing_config::{
        AutoSigningConfig, CommandSigningContext, EnhancedSigningConfig, SigningDebugConfig,
        SigningMode, SigningPerformanceConfig,
    },
};
