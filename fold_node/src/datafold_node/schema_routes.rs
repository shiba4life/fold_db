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
        Ok(schemas) => {
            info!("Successfully loaded {} schemas with states: {:?}", schemas.len(), schemas);
            HttpResponse::Ok().json(json!({"data": schemas}))
        },
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
    let node_guard = state.node.lock().await;

    match node_guard.unload_schema(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to unload schema: {}", e)})),
    }
}

/// Load a schema that exists but is not currently loaded.
pub async fn load_schema_route(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let mut node_guard = state.node.lock().await;

    // First check if schema exists but is unloaded
    match node_guard.get_schema(&name) {
        Ok(Some(schema)) => {
            // Schema exists, try to load it
            match node_guard.load_schema(schema) {
                Ok(_) => HttpResponse::Ok().json(json!({"success": true, "message": format!("Schema '{}' loaded successfully", name)})),
                Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to load schema '{}': {}", name, e)})),
            }
        },
        Ok(None) => HttpResponse::NotFound().json(json!({"error": format!("Schema '{}' not found", name)})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to get schema '{}': {}", name, e)})),
    }
}

/// List all available schemas (any state)
pub async fn list_available_schemas(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to list available schemas");
    let node_guard = state.node.lock().await;

    match node_guard.list_available_schemas() {
        Ok(schemas) => HttpResponse::Ok().json(json!({"data": schemas})),
        Err(e) => {
            error!("Failed to list available schemas: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to list available schemas: {}", e)}))
        }
    }
}

/// List schemas by specific state
pub async fn list_schemas_by_state(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let state_str = path.into_inner();
    info!("Received request to list schemas by state: {}", state_str);
    
    let schema_state = match state_str.as_str() {
        "available" => crate::schema::core::SchemaState::Available,
        "approved" => crate::schema::core::SchemaState::Approved,
        "blocked" => crate::schema::core::SchemaState::Blocked,
        _ => {
            return HttpResponse::BadRequest().json(json!({
                "error": "Invalid state. Use: available, approved, or blocked"
            }));
        }
    };

    let node_guard = state.node.lock().await;
    match node_guard.list_schemas_by_state(schema_state) {
        Ok(schemas) => HttpResponse::Ok().json(json!({
            "data": schemas,
            "state": state_str
        })),
        Err(e) => {
            error!("Failed to list schemas by state '{}': {}", state_str, e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to list schemas by state: {}", e)
            }))
        }
    }
}

/// Approve a schema for queries and mutations
pub async fn approve_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let schema_name = path.into_inner();
    info!("Received request to approve schema: {}", schema_name);
    
    let mut node_guard = state.node.lock().await;
    match node_guard.approve_schema(&schema_name) {
        Ok(()) => {
            info!("Schema '{}' approved successfully", schema_name);
            HttpResponse::Ok().json(json!({
                "message": format!("Schema '{}' approved successfully", schema_name),
                "schema": schema_name,
                "state": "approved"
            }))
        },
        Err(e) => {
            error!("Failed to approve schema '{}': {}", schema_name, e);
            HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to approve schema: {}", e)
            }))
        }
    }
}

/// Block a schema from queries and mutations
pub async fn block_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let schema_name = path.into_inner();
    info!("Received request to block schema: {}", schema_name);
    
    let mut node_guard = state.node.lock().await;
    match node_guard.block_schema(&schema_name) {
        Ok(()) => {
            info!("Schema '{}' blocked successfully", schema_name);
            HttpResponse::Ok().json(json!({
                "message": format!("Schema '{}' blocked successfully", schema_name),
                "schema": schema_name,
                "state": "blocked"
            }))
        },
        Err(e) => {
            error!("Failed to block schema '{}': {}", schema_name, e);
            HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to block schema: {}", e)
            }))
        }
    }
}

/// Get the current state of a schema
pub async fn get_schema_state(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let schema_name = path.into_inner();
    info!("Received request to get state for schema: {}", schema_name);
    
    let node_guard = state.node.lock().await;
    match node_guard.get_schema_state(&schema_name) {
        Ok(schema_state) => {
            let state_str = match schema_state {
                crate::schema::core::SchemaState::Available => "available",
                crate::schema::core::SchemaState::Approved => "approved",
                crate::schema::core::SchemaState::Blocked => "blocked",
            };
            HttpResponse::Ok().json(json!({
                "schema": schema_name,
                "state": state_str
            }))
        },
        Err(e) => {
            error!("Failed to get state for schema '{}': {}", schema_name, e);
            HttpResponse::NotFound().json(json!({
                "error": format!("Failed to get schema state: {}", e)
            }))
        }
    }
}

/// Get comprehensive schema status (NEW UNIFIED ENDPOINT)
pub async fn get_schema_status(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to get comprehensive schema status");
    let node_guard = state.node.lock().await;

    match node_guard.get_schema_status() {
        Ok(report) => HttpResponse::Ok().json(json!({"data": report})),
        Err(e) => {
            error!("Failed to get schema status: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to get schema status: {}", e)}))
        }
    }
}

/// Refresh schemas from all sources (NEW UNIFIED ENDPOINT)
pub async fn refresh_schemas(state: web::Data<AppState>) -> impl Responder {
    info!("Received request to refresh schemas from all sources");
    let node_guard = state.node.lock().await;

    match node_guard.refresh_schemas() {
        Ok(report) => {
            info!("Schema refresh completed: {} discovered, {} loaded, {} failed",
                  report.discovered_schemas.len(), report.loaded_schemas.len(), report.failed_schemas.len());
            HttpResponse::Ok().json(json!({"data": report}))
        },
        Err(e) => {
            error!("Failed to refresh schemas: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to refresh schemas: {}", e)}))
        }
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
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_schemas(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}

