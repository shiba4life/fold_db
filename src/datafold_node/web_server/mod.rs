pub mod types;
pub mod handlers;
pub mod server;
pub mod unix_socket;
pub mod auth;

// Re-export the WebServer struct for easier imports
pub use server::WebServer;
pub use types::{ApiSuccessResponse, ApiErrorResponse, QueryRequest, NetworkInitRequest, ConnectToNodeRequest};
