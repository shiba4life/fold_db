use std::sync::Arc;
use std::net::SocketAddr;
use std::time::Duration;
use warp::{Rejection, Reply};
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::network::NetworkConfig;
use crate::datafold_node::web_server::types::{ApiSuccessResponse, ApiErrorResponse, NetworkInitRequest, ConnectToNodeRequest};

pub async fn handle_init_network(
    config: NetworkInitRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let listen_address: SocketAddr = match config.listen_address.parse() {
        Ok(addr) => addr,
        Err(e) => {
            return Ok(warp::reply::json(&ApiErrorResponse::new(
                format!("Invalid listen address: {}", e)
            )));
        }
    };

    let network_config = NetworkConfig {
        listen_address,
        discovery_port: config.discovery_port,
        max_connections: config.max_connections,
        connection_timeout: Duration::from_secs(config.connection_timeout_secs),
        announcement_interval: Duration::from_secs(config.announcement_interval_secs),
        enable_discovery: config.enable_discovery,
    };

    let mut node = node.lock().await;
    match node.init_network(network_config) {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new("Network initialized successfully"))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_start_network(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node = node.lock().await;
    match node.start_network() {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new("Network started successfully"))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_stop_network(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node = node.lock().await;
    match node.stop_network() {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new("Network stopped successfully"))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_network_status(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    // Check if network is initialized
    let network_initialized = node.get_node_id() != "";
    
    // Get connected nodes if network is initialized
    let connected_nodes = if network_initialized {
        match node.get_connected_nodes() {
            Ok(nodes) => nodes.len(),
            Err(_) => 0,
        }
    } else {
        0
    };
    
    let status = json!({
        "initialized": network_initialized,
        "node_id": node.get_node_id(),
        "connected_nodes_count": connected_nodes,
    });
    
    Ok(warp::reply::json(&ApiSuccessResponse::new(status)))
}

pub async fn handle_discover_nodes(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node = node.lock().await;
    match node.discover_nodes() {
        Ok(nodes) => {
            let node_info = nodes.into_iter()
                .map(|node| json!({
                    "node_id": node.node_id,
                    "address": node.address.to_string(),
                    "trust_distance": node.trust_distance,
                }))
                .collect::<Vec<_>>();
            
            Ok(warp::reply::json(&ApiSuccessResponse::new(node_info)))
        },
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_connect_to_node(
    request: ConnectToNodeRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    match node.connect_to_node(&request.node_id) {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new(
            format!("Connected to node {}", request.node_id)
        ))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_list_nodes(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    
    // Get connected nodes
    let connected_nodes = match node.get_connected_nodes() {
        Ok(nodes) => nodes,
        Err(e) => return Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    };
    
    // Get known nodes
    let known_nodes = match node.get_known_nodes() {
        Ok(nodes) => nodes,
        Err(e) => return Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    };
    
    // Format response
    let response = json!({
        "node_id": node.get_node_id(),
        "connected_nodes": connected_nodes,
        "known_nodes": known_nodes.into_iter()
            .map(|(id, info)| json!({
                "node_id": id,
                "address": info.address.to_string(),
                "trust_distance": info.trust_distance,
                "connected": connected_nodes.contains(&id),
            }))
            .collect::<Vec<_>>(),
    });
    
    Ok(warp::reply::json(&ApiSuccessResponse::new(response)))
}
