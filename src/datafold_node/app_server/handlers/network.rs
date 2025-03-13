use std::sync::Arc;
use warp::{Rejection, Reply};
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::app_server::errors::{AppError, AppErrorResponse};
use crate::datafold_node::app_server::types::ApiSuccessResponse;
use crate::datafold_node::network::{NetworkConfig, NodeId};
use crate::schema::types::Query;

#[derive(Debug, Deserialize)]
pub struct NetworkInitRequest {
    pub enable_discovery: Option<bool>,
    pub discovery_port: Option<u16>,
    pub listen_port: Option<u16>,
    pub max_connections: Option<usize>,
    pub connection_timeout_secs: Option<u64>,
    pub announcement_interval_secs: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct ConnectRequest {
    pub node_id: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryNodeRequest {
    pub node_id: String,
    pub query: Query,
}

#[derive(Debug, Serialize)]
pub struct NodeIdResponse {
    pub node_id: String,
}

/// Initialize the network layer
pub async fn handle_init_network(
    request: NetworkInitRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node_lock = node.lock().await;
    
    // Create network config
    let listen_port = request.listen_port.unwrap_or(8091);
    let listen_address = format!("127.0.0.1:{}", listen_port).parse().unwrap();
    
    let network_config = NetworkConfig {
        listen_address,
        discovery_port: request.discovery_port.unwrap_or(8090),
        max_connections: request.max_connections.unwrap_or(50),
        connection_timeout: std::time::Duration::from_secs(request.connection_timeout_secs.unwrap_or(10)),
        announcement_interval: std::time::Duration::from_secs(request.announcement_interval_secs.unwrap_or(60)),
        enable_discovery: request.enable_discovery.unwrap_or(true),
    };
    
    // Initialize network
    match node_lock.init_network(network_config) {
        Ok(_) => {
            // Start network
            match node_lock.start_network() {
                Ok(_) => {
                    let node_id = node_lock.get_node_id().to_string();
                    let response = NodeIdResponse { node_id };
                    Ok(warp::reply::json(&ApiSuccessResponse::new(response, "network-init")))
                },
                Err(e) => {
                    let error = AppError::operation_error(format!("Failed to start network: {}", e));
                    Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
                }
            }
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to initialize network: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Connect to a node
pub async fn handle_connect_to_node(
    request: ConnectRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // Connect to node
    match node_lock.connect_to_node(&request.node_id) {
        Ok(_) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(
                format!("Successfully connected to node {}", request.node_id),
                "connect-node"
            )))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to connect to node: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Discover nodes
pub async fn handle_discover_nodes(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node_lock = node.lock().await;
    
    // Discover nodes
    match node_lock.discover_nodes() {
        Ok(nodes) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(nodes, "discover-nodes")))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to discover nodes: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Get connected nodes
pub async fn handle_get_connected_nodes(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // Get connected nodes
    match node_lock.get_connected_nodes() {
        Ok(nodes) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(nodes, "connected-nodes")))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to get connected nodes: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Get known nodes
pub async fn handle_get_known_nodes(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // Get known nodes
    match node_lock.get_known_nodes() {
        Ok(nodes) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(nodes, "known-nodes")))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to get known nodes: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Query a node
pub async fn handle_query_node(
    request: QueryNodeRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // Query node
    match node_lock.query_node(&request.node_id, request.query) {
        Ok(result) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(result, "query-node")))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to query node: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// List schemas on a node
pub async fn handle_list_node_schemas(
    request: ConnectRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // List schemas
    match node_lock.list_node_schemas(&request.node_id) {
        Ok(schemas) => {
            Ok(warp::reply::json(&ApiSuccessResponse::new(schemas, "list-schemas")))
        },
        Err(e) => {
            let error = AppError::operation_error(format!("Failed to list schemas: {}", e));
            Ok(warp::reply::json(&AppErrorResponse::from_app_error(&error)))
        }
    }
}

/// Get node ID
pub async fn handle_get_node_id(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node_lock = node.lock().await;
    
    // Get node ID
    let node_id = node_lock.get_node_id().to_string();
    let response = NodeIdResponse { node_id };
    
    Ok(warp::reply::json(&ApiSuccessResponse::new(response, "node-id")))
}
