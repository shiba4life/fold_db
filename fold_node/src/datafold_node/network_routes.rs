use super::http_server::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use crate::network::NetworkConfig;

#[derive(Deserialize)]
pub struct NetworkConfigPayload {
    listen_address: String,
    discovery_port: Option<u16>,
    max_connections: Option<usize>,
    connection_timeout_secs: Option<u64>,
    announcement_interval_secs: Option<u64>,
    enable_discovery: Option<bool>,
}

#[derive(Deserialize)]
pub struct ConnectRequest {
    node_id: String,
}

pub async fn init_network(
    config: web::Json<NetworkConfigPayload>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut node = state.node.lock().await;

    let mut network_config = NetworkConfig::new(&config.listen_address);
    if let Some(port) = config.discovery_port {
        network_config = network_config.with_discovery_port(port);
    }
    if let Some(max) = config.max_connections {
        network_config = network_config.with_max_connections(max);
    }
    if let Some(timeout) = config.connection_timeout_secs {
        network_config = network_config.with_connection_timeout(std::time::Duration::from_secs(timeout));
    }
    if let Some(interval) = config.announcement_interval_secs {
        network_config = network_config.with_announcement_interval(std::time::Duration::from_secs(interval));
    }
    if let Some(enable) = config.enable_discovery {
        network_config = network_config.with_mdns(enable);
    }

    match node.init_network(network_config).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to init network: {}", e) })),
    }
}

pub async fn start_network(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.start_network().await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to start network: {}", e) })),
    }
}

pub async fn stop_network(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.stop_network().await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to stop network: {}", e) })),
    }
}

pub async fn get_network_status(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.get_network_status().await {
        Ok(status) => HttpResponse::Ok().json(json!({ "data": status })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to get network status: {}", e) })),
    }
}

pub async fn connect_to_node(
    req: web::Json<ConnectRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut node = state.node.lock().await;
    match node.connect_to_node(&req.node_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to connect to node: {}", e) })),
    }
}

pub async fn discover_nodes(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.discover_nodes().await {
        Ok(peers) => {
            let peers: Vec<String> = peers.into_iter().map(|p| p.to_string()).collect();
            HttpResponse::Ok().json(json!({ "data": peers }))
        }
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to discover nodes: {}", e) })),
    }
}

pub async fn list_nodes(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.get_known_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(json!({ "data": nodes })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to list nodes: {}", e) })),
    }
}



