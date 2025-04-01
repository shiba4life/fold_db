pub mod atom;
pub mod db_operations;
pub mod error;
pub mod fees;
pub mod network;
pub mod permissions;
pub mod schema;
pub mod datafold_node;
pub mod fold_db_core;
pub mod testing;

// Import the SDK crate
pub use datafold_sdk;

// Re-export main types for convenience
pub use datafold_node::DataFoldNode;
pub use datafold_node::config::NodeConfig;
pub use datafold_node::loader::load_schema_from_file;
pub use error::{FoldDbError, FoldDbResult};
pub use fold_db_core::FoldDB;
pub use network::{NetworkCore, NetworkConfig, PeerId, NetworkError, NetworkResult, SchemaService};

// Re-export SDK types for backward compatibility
pub use datafold_sdk::{
    DataFoldClient, SocialAppContainer, ContainerConfig, AppSdkError, AppSdkResult,
    NetworkIsolation, MicroVMConfig, MicroVMType, LinuxContainerConfig, WasmSandboxConfig,
    AppPermissions, FieldPermissions, QueryBuilder, MutationBuilder, NetworkManager, SchemaDiscovery
};

// Re-export additional types needed for the SDK
pub use datafold_sdk::types::{
    QueryFilter, QueryResult, MutationResult, NodeInfo, RemoteNodeInfo
};
pub use datafold_sdk::mutation_builder::MutationType as AppSdkMutationType;

// Re-export schema types needed for CLI
pub use schema::Schema;
pub use schema::types::operations::MutationType;
pub use schema::types::operation::Operation;
