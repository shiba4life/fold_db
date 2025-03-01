pub mod types;
pub mod handlers;
pub mod server;

// Re-export the WebServer struct for easier imports
pub use server::WebServer;
pub use types::{ApiSuccessResponse, ApiErrorResponse, QueryRequest, NetworkInitRequest, ConnectToNodeRequest};
