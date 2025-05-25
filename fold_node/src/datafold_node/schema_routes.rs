#[cfg(test)]
use super::sample_manager::SampleManager;
use super::http_server::AppState;
use crate::schema::Schema;
use actix_web::{web, HttpResponse, Responder};
use serde_json::json;
use log::{info, error};

/// List all schemas.
pub async fn list_schemas(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to list schemas");
    let node_guard = state.node.lock().await;

    match node_guard.list_schemas_with_state() {
        Ok(schemas) => HttpResponse::Ok().json(json!({"data": schemas})),
        Err(e) => {
            error!("Failed to list schemas: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to list schemas: {}", e)}))
        }
    }
}

/// Get a schema by name.
pub async fn get_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let node_guard = state.node.lock().await;

    match node_guard.get_schema(&name) {
        Ok(Some(schema)) => HttpResponse::Ok().json(schema),
        Ok(None) => HttpResponse::NotFound().json(json!({"error": format!("Schema '{}' not found", name)})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to get schema: {}", e)})),
    }
}

/// Create a new schema.
pub async fn create_schema(schema: web::Json<Schema>, state: web::Data<AppState>) -> impl Responder {
    let mut node_guard = state.node.lock().await;

    match node_guard.load_schema(schema.into_inner()) {
        Ok(_) => HttpResponse::Created().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to create schema: {}", e)})),
    }
}

/// Update an existing schema.
pub async fn update_schema(path: web::Path<String>, schema: web::Json<Schema>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let schema_data = schema.into_inner();

    if schema_data.name != name {
        return HttpResponse::BadRequest().json(json!({"error": format!("Schema name '{}' does not match path '{}'", schema_data.name, name)}));
    }

    let mut node_guard = state.node.lock().await;

    let _ = node_guard.unload_schema(&name);

    match node_guard.load_schema(schema_data) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to update schema: {}", e)})),
    }
}

/// Unload a schema so it is no longer active.
pub async fn unload_schema_route(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let mut node_guard = state.node.lock().await;

    match node_guard.unload_schema(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to unload schema: {}", e)})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, config::NodeConfig};
    use actix_web::web;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_schemas_empty() {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let state = web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
            sample_manager: SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() }
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_schemas(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}

