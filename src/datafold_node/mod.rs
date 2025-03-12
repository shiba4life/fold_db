pub mod config;
pub mod error;
pub mod loader;
pub mod node;
pub mod tests;
pub mod web_server_compat;
pub mod ui_server;
pub mod app_server;
pub mod network;
pub mod web_server;

// Re-export the DataFoldNode struct for easier imports
pub use node::DataFoldNode;
pub use config::NodeConfig;
pub use ui_server::UiServer;
pub use app_server::AppServer;
pub use loader::load_schema_from_file;
