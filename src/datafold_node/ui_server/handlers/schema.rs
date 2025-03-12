use std::sync::Arc;
use warp::{Rejection, Reply};
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::schema::types::schema::Schema;
use crate::schema::types::Operation;
use crate::datafold_node::ui_server::types::{ApiSuccessResponse, QueryRequest};
use crate::datafold_node::ui_server::errors::{UiError, UiErrorResponse};

pub async fn handle_list_schemas(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    let schemas = node.list_schemas();
    match schemas {
        Ok(schemas) => Ok(warp::reply::json(&ApiSuccessResponse::new(schemas))),
        Err(e) => {
            let error = UiError::schema_error(e.to_string());
            Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)))
        }
    }
}

pub async fn handle_schema(
    schema: Schema,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Validate schema before loading
    if schema.name.is_empty() {
        let error = UiError::invalid_request("Schema name cannot be empty");
        return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
    }

    // Check if schema already exists
    let mut node = node.lock().await;
    let exists = node.get_schema(&schema.name).map(|s| s.is_some()).unwrap_or(false);
    if exists {
        let error = UiError::schema_error("Schema already exists");
        return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
    }

    // Load schema if it doesn't exist
    let schema_clone = schema.clone();
    let result = node.load_schema(schema_clone);

    match result {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new(schema))),
        Err(e) => {
            let error = UiError::schema_error(e.to_string());
            Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)))
        }
    }
}

pub async fn handle_delete_schema(
    name: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node = node.lock().await;
    
    // Check if schema exists before trying to remove it
    if !node.get_schema(&name).map(|s| s.is_some()).unwrap_or(false) {
        let error = UiError::schema_error("Schema not found");
        return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
    }

    match node.remove_schema(&name) {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new("Schema removed successfully"))),
        Err(e) => {
            let error = UiError::schema_error(e.to_string());
            Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)))
        }
    }
}

pub async fn handle_execute(
    query: QueryRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Parse the operation string into an Operation
    println!("Operation Entry: {:?}", query);
    let operation: Operation = match serde_json::from_str(&query.operation) {
        Ok(op) => op,
        Err(e) => {
            println!("Error parsing operation: {:?}", e);
            let error = UiError::invalid_request(format!("Invalid operation format: {}", e));
            return Err(warp::reject::custom(error));
        }
    };

    println!("Operation: {:?}", operation);

    let mut node = node.lock().await;
    let result = node.execute_operation(operation);

    // Print the result for debugging
    println!("Operation result: {:?}", result);

    match result {
        Ok(result) => {
            // Check if the result is actually an error wrapped in Ok
            if let serde_json::Value::Array(arr) = &result {
                if arr.len() == 1 {
                    if let Some(serde_json::Value::Object(obj)) = arr.first() {
                        if let Some(err_obj) = obj.get("Err") {
                            if let Some(not_found) = err_obj.as_object().and_then(|o| o.get("NotFound")) {
                                let error_msg = not_found.as_str().unwrap_or("Schema not found").to_string();
                                if error_msg.contains("Schema") {
                                    let error = UiError::schema_error("Schema not found");
                                    return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
                                }
                                let error = UiError::database_error(error_msg);
                                return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
                            }
                        }
                    }
                }
            }

            // For successful query results, ensure we return an object
            match result {
                serde_json::Value::Object(obj) => {
                    // If it's already an object, return it directly
                    Ok(warp::reply::json(&ApiSuccessResponse::new(obj)))
                },
                serde_json::Value::Array(arr) => {
                    if arr.len() == 1 {
                        match &arr[0] {
                            serde_json::Value::Object(obj) => {
                                // Check if this is an error object
                                if obj.contains_key("error") {
                                    let error_msg = obj.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string();
                                    let error = UiError::database_error(error_msg);
                                    return Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)));
                                }
                                
                                // Check if this is a wrapped Ok value
                                if let Some(ok_value) = obj.get("Ok") {
                                    // Try to parse the operation to get field names
                                    if let Ok(Operation::Query { fields, .. }) = serde_json::from_str::<Operation>(&query.operation) {
                                        if !fields.is_empty() {
                                            let mut result_obj = serde_json::Map::new();
                                            
                                            // Get the first field name
                                            let field_name = fields[0].clone();
                                            
                                            // Handle string values
                                            if let Some(field_value) = ok_value.as_str() {
                                                // Try to parse as JSON first
                                                if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(field_value) {
                                                    result_obj.insert(field_name, parsed_value);
                                                } else {
                                                    // Use as string if not valid JSON
                                                    result_obj.insert(field_name, serde_json::Value::String(field_value.to_string()));
                                                }
                                            } else {
                                                // Handle non-string values directly
                                                result_obj.insert(field_name, ok_value.clone());
                                            }
                                            
                                            return Ok(warp::reply::json(&ApiSuccessResponse::new(result_obj)));
                                        }
                                    }
                                    
                                    // Fallback if we can't get field names
                                    return Ok(warp::reply::json(&ApiSuccessResponse::new(json!({
                                        "result": ok_value
                                    }))));
                                }
                                
                                // Return the object directly if it's not a special case
                                Ok(warp::reply::json(&ApiSuccessResponse::new(obj)))
                            },
                            serde_json::Value::String(s) => {
                                // For string values, try to get the field name from the query
                                if let Ok(Operation::Query { fields, .. }) = serde_json::from_str::<Operation>(&query.operation) {
                                    if !fields.is_empty() {
                                        let field_name = fields[0].clone();
                                        return Ok(warp::reply::json(&ApiSuccessResponse::new(json!({
                                            field_name: s
                                        }))));
                                    }
                                }
                                
                                // Fallback to generic result
                                Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "result": s }))))
                            },
                            _ => {
                                // For other types, use a generic result key
                                Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "result": arr[0] }))))
                            }
                        }
                    } else {
                        // For multiple results, wrap them in a results array
                        Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "results": arr }))))
                    }
                },
                _ => {
                    // For any other type, use a generic result key
                    Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "result": result }))))
                }
            }
        },
        Err(e) => {
            let error = UiError::database_error(e.to_string());
            Ok(warp::reply::json(&UiErrorResponse::from_ui_error(&error)))
        }
    }
}
