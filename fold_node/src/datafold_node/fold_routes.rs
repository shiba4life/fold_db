use super::http_server::AppState;
use crate::schema::types::Fold;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

/// List all folds.
pub async fn list_folds(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to list folds");
    let node_guard = state.node.lock().await;

    match node_guard.list_folds() {
        Ok(folds) => HttpResponse::Ok().json(json!({"data": folds})),
        Err(e) => {
            error!("Failed to list folds: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to list folds: {}", e)}))
        }
    }
}

/// Get a fold by name.
pub async fn get_fold(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let node_guard = state.node.lock().await;

    match node_guard.get_fold(&name) {
        Ok(Some(fold)) => HttpResponse::Ok().json(fold),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": format!("Fold '{}' not found", name)})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to get fold: {}", e)})),
    }
}

/// Create a new fold.
pub async fn load_fold(fold: web::Json<Fold>, state: web::Data<AppState>) -> impl Responder {
    let mut node_guard = state.node.lock().await;

    match node_guard.load_fold(fold.into_inner()) {
        Ok(_) => HttpResponse::Created().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create fold: {}", e)})),
    }
}

/// Update an existing fold.
pub async fn update_fold(path: web::Path<String>, fold: web::Json<Fold>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let fold_data = fold.into_inner();

    if fold_data.name != name {
        return HttpResponse::BadRequest().json(json!({"error": format!("Fold name '{}' does not match path '{}'", fold_data.name, name)}));
    }

    let mut node_guard = state.node.lock().await;

    let _ = node_guard.unload_fold(&name);

    match node_guard.load_fold(fold_data) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to update fold: {}", e)})),
    }
}

/// Unload a fold so it is no longer active.
pub async fn unload_fold_route(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let mut node_guard = state.node.lock().await;

    match node_guard.unload_fold(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to unload fold: {}", e)})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, config::NodeConfig};
    use actix_web::web;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_folds_empty() {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let state = web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager: super::super::sample_manager::SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() }
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_folds(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}
