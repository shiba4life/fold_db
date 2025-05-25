use super::http_server::AppState;
use super::http_helpers::with_node;
use crate::schema::types::Fold;
use actix_web::{http::StatusCode, web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

/// List all folds.
pub async fn list_folds(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to list folds");
    with_node(state, |node| {
        node.list_folds()
            .map(|folds| (StatusCode::OK, json!({"data": folds})))
            .map_err(|e| {
                error!("Failed to list folds: {}", e);
                e
            })
    })
    .await
}

/// Get a fold by name.
pub async fn get_fold(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    with_node(state, move |node| {
        match node.get_fold(&name)? {
            Some(fold) => Ok((StatusCode::OK, json!(fold))),
            None => Ok((StatusCode::NOT_FOUND, json!({"error": format!("Fold '{}' not found", name)}))),
        }
    })
    .await
}

/// Create a new fold.
pub async fn load_fold(fold: web::Json<Fold>, state: web::Data<AppState>) -> impl Responder {
    with_node(state, |node| {
        node.load_fold(fold.into_inner())
            .map(|_| (StatusCode::CREATED, json!({"success": true})))
    })
    .await
}

/// Update an existing fold.
pub async fn update_fold(path: web::Path<String>, fold: web::Json<Fold>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let fold_data = fold.into_inner();

    if fold_data.name != name {
        return HttpResponse::BadRequest().json(json!({"error": format!("Fold name '{}' does not match path '{}'", fold_data.name, name)}));
    }

    with_node(state, move |node| {
        let _ = node.unload_fold(&name);
        node.load_fold(fold_data)
            .map(|_| (StatusCode::OK, json!({"success": true})))
    })
    .await
}

/// Unload a fold so it is no longer active.
pub async fn unload_fold_route(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    with_node(state, move |node| {
        node.unload_fold(&name)
            .map(|_| (StatusCode::OK, json!({"success": true})))
    })
    .await
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
