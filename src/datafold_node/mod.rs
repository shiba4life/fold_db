pub mod config;
pub mod error;
pub mod loader;
pub mod node;
pub mod tests;

// Re-export the DataFoldNode struct for easier imports
pub use node::DataFoldNode;
pub use config::NodeConfig;
pub use loader::load_schema_from_file;
