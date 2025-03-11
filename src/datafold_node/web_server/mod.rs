#![recursion_limit = "256"]

pub mod types;
pub mod handlers;
pub mod server;
pub mod api_server;
pub mod unix_socket;
pub mod auth;

// Re-export the WebServer and ApiServer structs for easier imports
pub use server::WebServer;
pub use api_server::ApiServer;
pub use types::{ApiSuccessResponse, ApiErrorResponse, QueryRequest, NetworkInitRequest, ConnectToNodeRequest};
