pub mod types;
pub mod errors;
pub mod handlers;
pub mod server;

// Re-export the UiServer struct for easier imports
pub use server::UiServer;
pub use types::{ApiSuccessResponse, QueryRequest, NetworkInitRequest, ConnectToNodeRequest};
pub use errors::{UiError, UiErrorResponse};
