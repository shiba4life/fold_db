use super::http_server::AppState;
use crate::schema::types::{
    operations::{Mutation, Query},
    Operation,
};
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use serde_json::{json, Value};

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
            return HttpResponse::BadRequest()
                .json(json!({"error": format!("Failed to parse operation: {}", e)}));
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => HttpResponse::Ok().json(json!({"data": result})),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({"error": format!("Failed to execute operation: {}", e)})),
    }
}

/// Execute a query.
pub async fn execute_query(query: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    let query_value = query.into_inner();
    log::info!(
        "Received query request: {}",
        serde_json::to_string(&query_value).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    // Parse the simple web UI operation
    let web_operation = match serde_json::from_value::<Operation>(query_value) {
        Ok(op) => match op {
            Operation::Query { .. } => op,
            _ => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": "Expected a query operation"}))
            }
        },
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(json!({"error": format!("Failed to parse query: {}", e)}))
        }
    };

    // Convert to full internal query with default trust_distance=0 and pub_key="web-ui"
    let internal_query = match web_operation {
        Operation::Query {
            schema,
            fields,
            filter,
        } => Query {
            schema_name: schema,
            fields,
            pub_key: "web-ui".to_string(),
            trust_distance: 0,
            filter,
        },
        _ => {
            return HttpResponse::BadRequest().json(json!({"error": "Expected a query operation"}))
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.query(internal_query) {
        Ok(results) => {
            log::info!("Query executed successfully");
            // Convert Vec<Result<Value, SchemaError>> to Vec<Value> with errors as JSON
            let unwrapped: Vec<Value> = results
                .into_iter()
                .map(|r| r.unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})))
                .collect();
            HttpResponse::Ok().json(json!({"data": unwrapped}))
        }
        Err(e) => {
            log::error!("Query execution failed: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": format!("Failed to execute query: {}", e)}))
        }
    }
}

/// Execute a mutation.
pub async fn execute_mutation(
    mutation: web::Json<Value>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mutation_value = mutation.into_inner();
    log::info!(
        "Received mutation request: {}",
        serde_json::to_string(&mutation_value).unwrap_or_else(|_| "Invalid JSON".to_string())
    );

    // Parse the simple web UI operation
    let web_operation = match serde_json::from_value::<Operation>(mutation_value) {
        Ok(op) => match op {
            Operation::Mutation { .. } => op,
            _ => {
                return HttpResponse::BadRequest()
                    .json(json!({"error": "Expected a mutation operation"}))
            }
        },
        Err(e) => {
            return HttpResponse::BadRequest()
                .json(json!({"error": format!("Failed to parse mutation: {}", e)}))
        }
    };

    // Convert to full internal mutation with default trust_distance=0 and pub_key="web-ui"
    let internal_mutation = match web_operation {
        Operation::Mutation {
            schema,
            data,
            mutation_type,
        } => {
            // Convert data Value to fields_and_values HashMap
            let fields_and_values = match data {
                Value::Object(map) => map.into_iter().collect(),
                _ => {
                    return HttpResponse::BadRequest()
                        .json(json!({"error": "Mutation data must be an object"}))
                }
            };

            Mutation {
                schema_name: schema,
                fields_and_values,
                pub_key: "web-ui".to_string(),
                trust_distance: 0,
                mutation_type,
            }
        }
        _ => {
            return HttpResponse::BadRequest()
                .json(json!({"error": "Expected a mutation operation"}))
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.mutate(internal_mutation) {
        Ok(_) => {
            log::info!("Mutation executed successfully");
            HttpResponse::Ok().json(json!({"success": true}))
        }
        Err(e) => {
            log::error!("Mutation execution failed: {}", e);
            HttpResponse::InternalServerError()
                .json(json!({"error": format!("Failed to execute mutation: {}", e)}))
        }
    }
}

pub async fn list_transforms(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.list_transforms() {
        Ok(map) => HttpResponse::Ok().json(json!({ "data": map })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to list transforms: {}", e) })),
    }
}

pub async fn run_transform(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let id = path.into_inner();
    let mut node = state.node.lock().await;
    match node.run_transform(&id) {
        Ok(val) => HttpResponse::Ok().json(json!({ "data": val })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to run transform: {}", e) })),
    }
}

pub async fn add_to_transform_queue(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let transform_id = path.into_inner();
    let node = state.node.lock().await;

    match node.list_transforms() {
        Ok(transforms) => {
            if !transforms.contains_key(&transform_id) {
                return HttpResponse::NotFound().json(json!({"error": format!("Transform '{}' not found. Available transforms: {:?}", transform_id, transforms.keys().collect::<Vec<_>>())}));
            }
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .json(json!({"error": format!("Failed to verify transform: {}", e)}))
        }
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
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to get transform queue info: {}", e) })),
    }
}

#[cfg(test)]
mod tests {}
