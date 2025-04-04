// Re-export everything from fold_node
pub use fold_node::*;

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
