use std::convert::Infallible;
use std::sync::Arc;
use warp::Filter;
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub operation: String,
}

#[derive(Debug, Deserialize)]
pub struct NetworkInitRequest {
    pub listen_address: String,
    pub discovery_port: u16,
    pub max_connections: usize,
    pub connection_timeout_secs: u64,
    pub announcement_interval_secs: u64,
    pub enable_discovery: bool,
}

#[derive(Debug, Deserialize)]
pub struct ConnectToNodeRequest {
    pub node_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccessResponse<T: Serialize> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

impl warp::reject::Reject for ApiErrorResponse {}

impl<T: Serialize> ApiSuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl ApiErrorResponse {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}

/// Utility function to share the DataFoldNode with route handlers
pub fn with_node(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<DataFoldNode>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&node))
}
