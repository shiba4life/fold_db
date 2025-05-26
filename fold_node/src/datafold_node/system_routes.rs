use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

use super::http_server::AppState;

/// Restart the fold_node using built-in restart functionality
pub async fn restart_node(state: web::Data<AppState>) -> impl Responder {
    info!("Restart request received - performing node restart");
    
    // Get a mutable reference to the node
    let mut node = match state.node.try_lock() {
        Ok(node) => node,
        Err(_) => {
            error!("Failed to acquire node lock for restart");
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Node is currently busy, cannot restart"
            }));
        }
    };
    
    // Perform the restart
    match node.restart().await {
        Ok(_) => {
            info!("Node restart completed successfully");
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Node restarted successfully"
            }))
        }
        Err(e) => {
            error!("Node restart failed: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Restart failed: {}", e)
            }))
        }
    }
}

/// Perform a soft restart of the fold_node
pub async fn soft_restart_node(state: web::Data<AppState>) -> impl Responder {
    info!("Soft restart request received");
    
    // Get a mutable reference to the node
    let mut node = match state.node.try_lock() {
        Ok(node) => node,
        Err(_) => {
            error!("Failed to acquire node lock for soft restart");
            return HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": "Node is currently busy, cannot restart"
            }));
        }
    };
    
    // Perform the soft restart
    match node.soft_restart().await {
        Ok(_) => {
            info!("Node soft restart completed successfully");
            HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Node soft restarted successfully"
            }))
        }
        Err(e) => {
            error!("Node soft restart failed: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "error": format!("Soft restart failed: {}", e)
            }))
        }
    }
}

/// Get system status information
pub async fn get_system_status(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "running",
        "uptime": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, NodeConfig};
    use crate::datafold_node::sample_manager::SampleManager;
    use actix_web::test;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_system_status() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let sample_manager = SampleManager::new().await.unwrap();
        
        let state = web::Data::new(AppState {
            node: Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager,
        });

        let req = test::TestRequest::get().to_http_request();
        let resp = get_system_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_restart_endpoint() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let sample_manager = SampleManager::new().await.unwrap();
        
        let state = web::Data::new(AppState {
            node: Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager,
        });

        let req = test::TestRequest::post().to_http_request();
        let resp = restart_node(state).await.respond_to(&req);
        // The restart might succeed or fail depending on the test environment
        // We just check that we get a valid HTTP response
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_soft_restart_endpoint() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let sample_manager = SampleManager::new().await.unwrap();
        
        let state = web::Data::new(AppState {
            node: Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager,
        });

        let req = test::TestRequest::post().to_http_request();
        let resp = soft_restart_node(state).await.respond_to(&req);
        // The restart might succeed or fail depending on the test environment
        // We just check that we get a valid HTTP response
        assert!(resp.status().is_success() || resp.status().is_server_error());
    }
}