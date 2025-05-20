use super::http_server::AppState;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::schema::types::Operation;

/// Execute an operation (query or mutation).
#[derive(Deserialize)]
pub(crate) struct OperationRequest {
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
    let operation = match serde_json::from_value::<Operation>(query.into_inner()) {
        Ok(op) => match op {
            Operation::Query { .. } => op,
            _ => return HttpResponse::BadRequest().json(json!({"error": "Expected a query operation"})),
        },
        Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("Failed to parse query: {}", e)})),
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => HttpResponse::Ok().json(json!({"data": result})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to execute query: {}", e)})),
    }
}

/// Execute a mutation.
pub async fn execute_mutation(mutation: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    let operation = match serde_json::from_value::<Operation>(mutation.into_inner()) {
        Ok(op) => match op {
            Operation::Mutation { .. } => op,
            _ => return HttpResponse::BadRequest().json(json!({"error": "Expected a mutation operation"})),
        },
        Err(e) => return HttpResponse::BadRequest().json(json!({"error": format!("Failed to parse mutation: {}", e)})),
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(_) => HttpResponse::Ok().json(json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(json!({"error": format!("Failed to execute mutation: {}", e)})),
    }
}

/// List all sample schemas.
pub async fn list_schema_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": state.sample_manager.list_schema_samples()}))
}

/// List all sample queries.
pub async fn list_query_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": state.sample_manager.list_query_samples()}))
}

/// List all sample mutations.
pub async fn list_mutation_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(json!({"data": state.sample_manager.list_mutation_samples()}))
}

/// Get a sample schema by name.
pub async fn get_schema_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();

    match state.sample_manager.get_schema_sample(&name) {
        Some(schema) => HttpResponse::Ok().json(schema),
        None => HttpResponse::NotFound().json(json!({"error": format!("Sample schema '{}' not found", name)})),
    }
}

/// Get a sample query by name.
pub async fn get_query_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();

    match state.sample_manager.get_query_sample(&name) {
        Some(query) => HttpResponse::Ok().json(query),
        None => HttpResponse::NotFound().json(json!({"error": format!("Sample query '{}' not found", name)})),
    }
}

/// Get a sample mutation by name.
pub async fn get_mutation_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();

    match state.sample_manager.get_mutation_sample(&name) {
        Some(mutation) => HttpResponse::Ok().json(mutation),
        None => HttpResponse::NotFound().json(json!({"error": format!("Sample mutation '{}' not found", name)})),
    }
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
            sample_manager: super::super::sample_manager::SampleManager { schemas: Default::default(), queries: Default::default(), mutations: Default::default() }
        });
        let resp = list_schema_samples(state).await.respond_to(&actix_web::HttpRequest::default());
        assert_eq!(resp.status(), 200);
    }
}

