use super::http_server::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::json;
use crate::network::NetworkConfig;

#[derive(Deserialize)]
pub(crate) struct NetworkConfigPayload {
    listen_address: String,
    discovery_port: Option<u16>,
    max_connections: Option<usize>,
    connection_timeout_secs: Option<u64>,
    announcement_interval_secs: Option<u64>,
    enable_discovery: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct ConnectRequest {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, config::NodeConfig};
    use actix_web::web;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_nodes_empty() {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let state = web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager: super::super::sample_manager::SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() }
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_nodes(state).await.respond_to(&req);
        assert_eq!(resp.status(), 500);
    }

    fn create_state() -> web::Data<super::super::http_server::AppState> {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager: super::super::sample_manager::SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() },
        })
    }

    #[tokio::test]
    async fn init_network_applies_config() {
        use actix_web::test;
        let state = create_state();
        let payload = NetworkConfigPayload {
            listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
            discovery_port: Some(1234),
            max_connections: Some(5),
            connection_timeout_secs: Some(1),
            announcement_interval_secs: Some(2),
            enable_discovery: Some(false),
        };
        let req = test::TestRequest::default().to_http_request();
        let resp = init_network(web::Json(payload), state.clone())
            .await
            .respond_to(&req);
        assert_eq!(resp.status(), 200);
        let node = state.node.lock().await;
        assert!(node.network.is_some());
    }

    #[tokio::test]
    async fn start_and_stop_network_responses() {
        use actix_web::test;
        let state = create_state();
        let req = test::TestRequest::default().to_http_request();
        let start_resp = start_network(state.clone()).await.respond_to(&req);
        assert_eq!(start_resp.status(), 500);

        let payload = NetworkConfigPayload { listen_address: "/ip4/127.0.0.1/tcp/0".to_string(), discovery_port: None, max_connections: None, connection_timeout_secs: None, announcement_interval_secs: None, enable_discovery: None };
        let _ = init_network(web::Json(payload), state.clone()).await;

        let start_resp = start_network(state.clone()).await.respond_to(&req);
        assert_eq!(start_resp.status(), 200);
        let stop_resp = stop_network(state.clone()).await.respond_to(&req);
        assert_eq!(stop_resp.status(), 200);
    }

    #[tokio::test]
    async fn get_network_status_uninitialized_vs_initialized() {
        use actix_web::test;
        let state = create_state();
        let req = test::TestRequest::default().to_http_request();
        let resp = get_network_status(state.clone()).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
        let body_bytes = actix_web::body::to_bytes(resp.into_body())
            .await
            .unwrap_or_else(|_| panic!("body error"));
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body["data"]["initialized"], false);

        let payload = NetworkConfigPayload { listen_address: "/ip4/127.0.0.1/tcp/0".to_string(), discovery_port: None, max_connections: None, connection_timeout_secs: None, announcement_interval_secs: None, enable_discovery: None };
        let _ = init_network(web::Json(payload), state.clone()).await;
        let resp2 = get_network_status(state.clone()).await.respond_to(&req);
        assert_eq!(resp2.status(), 200);
        let body_bytes = actix_web::body::to_bytes(resp2.into_body())
            .await
            .unwrap_or_else(|_| panic!("body error"));
        let body2: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(body2["data"]["initialized"], true);
    }

    #[tokio::test]
    async fn connect_and_discover_nodes() {
        use actix_web::test;
        let state = create_state();
        let req = test::TestRequest::default().to_http_request();
        let resp = discover_nodes(state.clone()).await.respond_to(&req);
        assert_eq!(resp.status(), 500);

        let payload = NetworkConfigPayload { listen_address: "/ip4/127.0.0.1/tcp/0".to_string(), discovery_port: None, max_connections: None, connection_timeout_secs: None, announcement_interval_secs: None, enable_discovery: None };
        let _ = init_network(web::Json(payload), state.clone()).await;
        let resp = connect_to_node(web::Json(ConnectRequest { node_id: "peer1".into() }), state.clone()).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
        let node = state.node.lock().await;
        assert!(node.trusted_nodes.contains_key("peer1"));
        drop(node);
        let resp = discover_nodes(state.clone()).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}

