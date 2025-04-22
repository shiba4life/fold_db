pub mod config;
pub mod error;
pub mod loader;
pub mod node;
pub mod tcp_server;
pub mod tests;

// Re-export the DataFoldNode struct for easier imports
pub use config::NodeConfig;
pub use loader::load_schema_from_file;
pub use node::DataFoldNode;
pub use tcp_server::TcpServer;
