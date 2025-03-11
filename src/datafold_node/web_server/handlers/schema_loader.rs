use std::sync::Arc;
use warp::{Rejection, Reply};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::web_server::types::{ApiSuccessResponse, ApiErrorResponse};
use crate::schema::Schema;
use std::fs;
use std::path::Path;

/// Request to load a schema from a file
#[derive(Debug, Deserialize)]
pub struct LoadSchemaFromFileRequest {
    /// Path to the schema file
    pub file_path: String,
}

/// Request to load a schema from JSON content
#[derive(Debug, Deserialize)]
pub struct LoadSchemaFromJsonRequest {
    /// JSON content of the schema
    pub schema_json: Value,
}

/// Response for schema loading
#[derive(Debug, Serialize)]
pub struct LoadSchemaResponse {
    /// Name of the loaded schema
    pub schema_name: String,
    /// Status message
    pub message: String,
}

/// Handle loading a schema from a file
pub async fn handle_load_schema_from_file(
    request: LoadSchemaFromFileRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Validate file path
    let path = Path::new(&request.file_path);
    if !path.exists() {
        return Ok(warp::reply::json(&ApiErrorResponse::new(
            format!("Schema file not found: {}", request.file_path)
        )));
    }

    // Read schema file
    let schema_str = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(e) => {
            return Ok(warp::reply::json(&ApiErrorResponse::new(
                format!("Failed to read schema file: {}", e)
            )));
        }
    };

    // Parse schema
    let schema: Schema = match serde_json::from_str(&schema_str) {
        Ok(schema) => schema,
        Err(e) => {
            return Ok(warp::reply::json(&ApiErrorResponse::new(
                format!("Failed to parse schema: {}", e)
            )));
        }
    };

    // Get schema name for response
    let schema_name = schema.name.clone();

    // Load schema
    let mut node = node.lock().await;
    match node.load_schema(schema) {
        Ok(_) => {
            let response = LoadSchemaResponse {
                schema_name,
                message: "Schema loaded successfully".to_string(),
            };
            Ok(warp::reply::json(&ApiSuccessResponse::new(response)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiErrorResponse::new(format!(
                "Failed to load schema: {}",
                e
            ))))
        }
    }
}

/// Handle loading a schema from JSON content
pub async fn handle_load_schema_from_json(
    request: LoadSchemaFromJsonRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Parse schema
    let schema: Schema = match serde_json::from_value(request.schema_json) {
        Ok(schema) => schema,
        Err(e) => {
            return Ok(warp::reply::json(&ApiErrorResponse::new(
                format!("Failed to parse schema: {}", e)
            )));
        }
    };

    // Get schema name for response
    let schema_name = schema.name.clone();

    // Load schema
    let mut node = node.lock().await;
    match node.load_schema(schema) {
        Ok(_) => {
            let response = LoadSchemaResponse {
                schema_name,
                message: "Schema loaded successfully".to_string(),
            };
            Ok(warp::reply::json(&ApiSuccessResponse::new(response)))
        }
        Err(e) => {
            Ok(warp::reply::json(&ApiErrorResponse::new(format!(
                "Failed to load schema: {}",
                e
            ))))
        }
    }
}

/// Handle loading a schema with authentication
pub async fn handle_load_schema_from_file_with_auth(
    trust_level: u32,
    request: LoadSchemaFromFileRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for schema loading operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for schema loading operations",
        )));
    }

    // Call the original handler
    handle_load_schema_from_file(request, node).await
}

/// Handle loading a schema from JSON with authentication
pub async fn handle_load_schema_from_json_with_auth(
    trust_level: u32,
    request: LoadSchemaFromJsonRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for schema loading operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for schema loading operations",
        )));
    }

    // Call the original handler
    handle_load_schema_from_json(request, node).await
}
