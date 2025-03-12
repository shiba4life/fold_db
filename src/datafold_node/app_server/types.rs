use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;

/// Represents a signed request from a third-party application
#[derive(Debug, Deserialize, Serialize)]
pub struct SignedRequest {
    /// Unix timestamp when the request was created
    pub timestamp: u64,
    /// The actual payload of the request
    pub payload: RequestPayload,
}

/// The payload of a signed request
#[derive(Debug, Deserialize, Serialize)]
pub struct RequestPayload {
    /// The type of operation to perform
    pub operation: String,
    /// The content of the operation (JSON string)
    pub content: String,
}

/// Response for successful operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccessResponse<T: Serialize> {
    pub data: T,
    pub timestamp: u64,
    pub request_id: String,
}

impl<T: Serialize> ApiSuccessResponse<T> {
    pub fn new(data: T, request_id: &str) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
            
        Self { 
            data, 
            timestamp,
            request_id: request_id.to_string(),
        }
    }
}

/// Utility function to share the DataFoldNode with route handlers
pub fn with_node(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<DataFoldNode>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&node))
}

/// Utility function to extract the public key from headers
pub fn extract_public_key(headers: &warp::http::HeaderMap) -> Option<String> {
    headers.get("x-public-key")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Utility function to extract the signature from headers
pub fn extract_signature(headers: &warp::http::HeaderMap) -> Option<String> {
    headers.get("x-signature")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// Utility function to extract the client IP address
pub fn extract_client_ip(addr: Option<std::net::SocketAddr>) -> String {
    addr.map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string())
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    use rand::Rng;
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    
    let random: u32 = rand::thread_rng().gen();
    format!("{}-{:x}", timestamp, random)
}
