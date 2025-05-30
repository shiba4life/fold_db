use super::http_server::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::schema::types::Operation;

/// Execute an operation (query or mutation).
#[derive(Deserialize)]
pub struct OperationRequest {
    operation: String,
}

pub async fn execute_operation(
    request: web::Json<OperationRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let operation_str = &request.operation;

    let operation: Operation = match serde_json::from_str(operation_str) {
        Ok(op) => op,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({"error": format!("Failed to parse operation: {}", e)}));
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => HttpResponse::Ok().json(json!({"data": result})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to execute operation: {}", e)})),
    }
}

/// Execute a query.
pub async fn execute_query(query: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    let query_value = query.into_inner();
    log::info!("Received query request: {}", serde_json::to_string(&query_value).unwrap_or_else(|_| "Invalid JSON".to_string()));
    
    let operation = match serde_json::from_value::<Operation>(query_value) {
        Ok(op) => match op {
            Operation::Query { .. } => op,
            _ => return HttpResponse::BadRequest().json(json!({"error": "Expected a query operation"})),
        },
        Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("Failed to parse query: {}", e)})),
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => {
            log::info!("Query executed successfully");
            HttpResponse::Ok().json(json!({"data": result}))
        },
        Err(e) => {
            log::error!("Query execution failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to execute query: {}", e)}))
        },
    }
}

/// Execute a mutation.
pub async fn execute_mutation(mutation: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    let mutation_value = mutation.into_inner();
    log::info!("Received mutation request: {}", serde_json::to_string(&mutation_value).unwrap_or_else(|_| "Invalid JSON".to_string()));
    
    let operation = match serde_json::from_value::<Operation>(mutation_value) {
        Ok(op) => match op {
            Operation::Mutation { .. } => op,
            _ => return HttpResponse::BadRequest().json(json!({"error": "Expected a mutation operation"})),
        },
        Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("Failed to parse mutation: {}", e)})),
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(_) => {
            log::info!("Mutation executed successfully");
            HttpResponse::Ok().json(json!({"success": true}))
        },
        Err(e) => {
            log::error!("Mutation execution failed: {}", e);
            HttpResponse::InternalServerError().json(json!({"error": format!("Failed to execute mutation: {}", e)}))
        },
    }
}

/// List all sample schemas (DEPRECATED - samples removed).
pub async fn list_schema_samples(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": []}))
}

/// List all sample queries (DEPRECATED - samples removed).
pub async fn list_query_samples(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": []}))
}

/// List all sample mutations (DEPRECATED - samples removed).
pub async fn list_mutation_samples(_state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": []}))
}

/// Get a sample schema by name (DEPRECATED - samples removed).
pub async fn get_schema_sample(path: web::Path<String>, _state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    HttpResponse::NotFound().json(json!({"error": format!("Sample schema '{}' not found - samples have been removed", name)}))
}

/// Get a sample query by name (DEPRECATED - samples removed).
pub async fn get_query_sample(path: web::Path<String>, _state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    HttpResponse::NotFound().json(json!({"error": format!("Sample query '{}' not found - samples have been removed", name)}))
}

/// Get a sample mutation by name (DEPRECATED - samples removed).
pub async fn get_mutation_sample(path: web::Path<String>, _state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    HttpResponse::NotFound().json(json!({"error": format!("Sample mutation '{}' not found - samples have been removed", name)}))
}

pub async fn list_transforms(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.list_transforms() {
        Ok(map) => HttpResponse::Ok().json(json!({ "data": map })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to list transforms: {}", e) })),
    }
}

pub async fn run_transform(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut node = state.node.lock().await;
    match node.run_transform(&id) {
        Ok(val) => HttpResponse::Ok().json(json!({ "data": val })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to run transform: {}", e) })),
    }
}

pub async fn add_to_transform_queue(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let transform_id = path.into_inner();
    let node = state.node.lock().await;

    match node.list_transforms() {
        Ok(transforms) => {
            if !transforms.contains_key(&transform_id) {
                return HttpResponse::NotFound().json(json!({"error": format!("Transform '{}' not found. Available transforms: {:?}", transform_id, transforms.keys().collect::<Vec<_>>())}));
            }
        }
        Err(e) => return HttpResponse::InternalServerError().json(json!({"error": format!("Failed to verify transform: {}", e)})),
    }

    match node.add_transform_to_queue(&transform_id) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true, "message": format!("Transform '{}' added to queue", transform_id)})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to add transform to queue: {}", e)})),
    }
}

pub async fn get_transform_queue(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.get_transform_queue_info() {
        Ok(info) => HttpResponse::Ok().json(info),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": format!("Failed to get transform queue info: {}", e) })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::{DataFoldNode, config::NodeConfig};
    use actix_web::web;
    use tempfile::tempdir;

    #[tokio::test]
    async fn list_sample_schemas_empty() {
        let dir = tempdir().unwrap();
        let config = NodeConfig::new(dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();
        let state = web::Data::new(super::super::http_server::AppState {
            node: std::sync::Arc::new(tokio::sync::Mutex::new(node)),
        });
        use actix_web::test;
        let req = test::TestRequest::default().to_http_request();
        let resp = list_schema_samples(state).await.respond_to(&req);
        assert_eq!(resp.status(), 200);
    }
}

