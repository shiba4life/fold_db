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
pub mod error;
pub mod http_server;
pub mod schema_routes;
pub mod query_routes;
pub mod network_routes;
pub mod system_routes;
pub mod log_routes;
mod db;
mod permissions;
mod transform_queue;
pub mod node;
pub mod tcp_server;
pub mod tcp_protocol;
pub mod tcp_connections;
pub mod tcp_command_router;
pub mod tests;

// Re-export the DataFoldNode struct for easier imports
pub use config::NodeConfig;
pub use config::load_node_config;
pub use http_server::DataFoldHttpServer;
pub use node::DataFoldNode;
pub use tcp_server::TcpServer;
