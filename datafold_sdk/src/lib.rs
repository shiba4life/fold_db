//! DataFold SDK for interacting with DataFold nodes
//!
//! This crate provides a client SDK for applications to interact with DataFold nodes
//! while maintaining security and privacy.

pub mod client;
pub mod container;
pub mod error;
pub mod isolation;
pub mod permissions;
pub mod query_builder;
pub mod mutation_builder;
pub mod network_manager;
pub mod network_utils;
#[cfg(any(test, feature = "mock"))]
pub mod network_mock;
pub mod schema;
pub mod schema_builder;
pub mod schema_discovery;
pub mod types;

// Re-export main types for convenience
pub use client::DataFoldClient;
pub use container::{SocialAppContainer, ContainerConfig};
pub use error::{AppSdkError, AppSdkResult};
pub use isolation::{NetworkIsolation, MicroVMConfig, MicroVMType, LinuxContainerConfig, WasmSandboxConfig};
pub use permissions::{AppPermissions, FieldPermissions};
pub use query_builder::QueryBuilder;
pub use mutation_builder::MutationBuilder;
pub use network_manager::NetworkManager;
pub use network_utils::NetworkUtils;
#[cfg(any(test, feature = "mock"))]
pub use network_mock::NetworkMock;
pub use schema::*;
pub use schema_builder::SchemaBuilder;
pub use schema_discovery::SchemaDiscovery;
pub use types::{
    AppRequest, AppChannel, NodeConnection, AuthCredentials, SchemaCache, 
    NodeInfo, QueryFilter, QueryResult, MutationResult, RemoteNodeInfo
};
