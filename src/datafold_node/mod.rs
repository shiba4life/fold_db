//! DataFoldNode module provides the core node functionality for the FoldDB database system.
//! 
//! A DataFoldNode represents a single node in the distributed database system, responsible for:
//! - Managing local data storage
//! - Handling schema operations
//! - Processing queries and mutations
//! - Managing trust relationships between nodes
//! 
//! # Example
//! ```no_run
//! use fold_db::{DataFoldNode, NodeConfig, datafold_node::DockerConfig};
//! use std::path::PathBuf;
//! 
//! let config = NodeConfig {
//!     storage_path: PathBuf::from("/tmp/db"),
//!     default_trust_distance: 1,
//!     docker: DockerConfig::default(),
//! };
//! 
//! let node = DataFoldNode::new(config).expect("Failed to create node");
//! ```

mod config;
mod docker;
mod error;
mod node;
pub mod web_server;
mod loader;
#[cfg(test)]
mod tests;

pub use config::{DockerConfig, DockerNetworkConfig, NodeConfig};
pub use docker::{ContainerState, ContainerStatus};
pub use error::{NodeError, NodeResult};
pub use node::DataFoldNode;
pub use web_server::{WebServer, ApiSuccessResponse, ApiErrorResponse, handle_schema, with_node};
pub use loader::load_schema_from_file;
