use actix_web::{web, HttpResponse, Responder};
use serde_json::json;

use super::http_server::AppState;

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
    use actix_web::test;
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_system_status() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();

        let state = web::Data::new(AppState {
            node: Arc::new(tokio::sync::Mutex::new(node)),
        });

        let req = test::TestRequest::get().to_http_request();
        let resp = get_system_status(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}
