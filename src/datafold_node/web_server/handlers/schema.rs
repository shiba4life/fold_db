use std::sync::Arc;
use warp::{Rejection, Reply};
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::schema::types::schema::Schema;
use crate::schema::types::Operation;
use crate::datafold_node::web_server::types::{ApiSuccessResponse, ApiErrorResponse, QueryRequest};

pub async fn handle_list_schemas(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let node = node.lock().await;
    let schemas = node.list_schemas();
    match schemas {
        Ok(schemas) => Ok(warp::reply::json(&ApiSuccessResponse::new(schemas))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_schema(
    schema: Schema,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Validate schema before loading
    if schema.name.is_empty() {
        return Ok(warp::reply::json(&ApiErrorResponse::new("Schema name cannot be empty")));
    }

    // Check if schema already exists
    let mut node = node.lock().await;
    let exists = node.get_schema(&schema.name).map(|s| s.is_some()).unwrap_or(false);
    if exists {
        return Ok(warp::reply::json(&ApiErrorResponse::new("Schema error: Schema already exists")));
    }

    // Load schema if it doesn't exist
    let schema_clone = schema.clone();
    let result = node.load_schema(schema_clone);

    match result {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new(schema))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_delete_schema(
    name: String,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    let mut node = node.lock().await;
    
    // Check if schema exists before trying to remove it
    if !node.get_schema(&name).map(|s| s.is_some()).unwrap_or(false) {
        return Ok(warp::reply::json(&ApiErrorResponse::new("Schema not found")));
    }

    match node.remove_schema(&name) {
        Ok(_) => Ok(warp::reply::json(&ApiSuccessResponse::new("Schema removed successfully"))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}

pub async fn handle_execute(
    query: QueryRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Parse the operation into an Operation
    println!("Operation Entry: {:?}", query);
    println!("Operation String: {}", query.operation);
    
    // Try to parse the operation directly
    let operation: Operation = match serde_json::from_str(&query.operation) {
        Ok(op) => {
            println!("Successfully parsed operation directly");
            op
        },
        Err(e1) => {
            println!("Failed to parse operation directly: {}", e1);
            
            // The operation might be a JSON string inside a JSON string, try to parse it
            if query.operation.starts_with("\"") && query.operation.ends_with("\"") {
                match serde_json::from_str::<String>(&query.operation) {
                    Ok(inner_str) => {
                        println!("Unescaped inner operation string: {}", inner_str);
                        match serde_json::from_str(&inner_str) {
                            Ok(op) => {
                                println!("Successfully parsed inner operation");
                                op
                            },
                            Err(e2) => {
                                println!("Error parsing inner operation: {:?}", e2);
                                println!("Inner operation string that failed to parse: {}", inner_str);
                                return Err(warp::reject::custom(ApiErrorResponse::new(
                                    format!("Invalid operation format: {}", e2)
                                )));
                            }
                        }
                    },
                    Err(e3) => {
                        println!("Error unescaping operation string: {:?}", e3);
                        return Err(warp::reject::custom(ApiErrorResponse::new(
                            format!("Invalid operation format: {}", e3)
                        )));
                    }
                }
            } else {
                println!("Operation string that failed to parse: {}", query.operation);
                return Err(warp::reject::custom(ApiErrorResponse::new(
                    format!("Invalid operation format: {}", e1)
                )));
            }
        }
    };

    println!("Operation: {:?}", operation);

    let mut node = node.lock().await;
    let result = node.execute_operation(operation);

    // Print the result for debugging
    println!("Operation result: {:?}", result);

    match result {
        Ok(result) => {
            // Debug: Print the raw result before processing
            println!("Raw result before processing: {}", serde_json::to_string(&result).unwrap_or_else(|e| format!("Error serializing: {}", e)));
            
            // Check if the result is actually an error wrapped in Ok
            if let serde_json::Value::Array(arr) = &result {
                if arr.len() == 1 {
                    if let Some(serde_json::Value::Object(obj)) = arr.first() {
                        if let Some(err_obj) = obj.get("Err") {
                            println!("Found Err object: {:?}", err_obj);
                            if let Some(not_found) = err_obj.as_object().and_then(|o| o.get("NotFound")) {
                                let error_msg = not_found.as_str().unwrap_or("Schema not found").to_string();
                                println!("Error message from NotFound: {}", error_msg);
                                if error_msg.contains("Schema") {
                                    return Ok(warp::reply::json(&ApiErrorResponse::new("Schema not found")));
                                }
                                return Ok(warp::reply::json(&ApiErrorResponse::new(error_msg)));
                            }
                        }
                    }
                }
            }

            // For successful query results, ensure we return an object
            match result {
                serde_json::Value::Object(obj) => {
                    // If it's already an object, return it directly
                    println!("Returning object directly: {:?}", obj);
                    let response = ApiSuccessResponse::new(obj);
                    let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                    println!("Final JSON response: {}", json_response);
                    Ok(warp::reply::json(&response))
                },
                serde_json::Value::Array(arr) => {
                    if arr.len() == 1 {
                        match &arr[0] {
                            serde_json::Value::Object(obj) => {
                                println!("Processing single object in array: {:?}", obj);
                                // Check if this is an error object
                                if obj.contains_key("error") {
                                    let error_msg = obj.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error").to_string();
                                    println!("Found error object: {}", error_msg);
                                    return Ok(warp::reply::json(&ApiErrorResponse::new(error_msg)));
                                }
                                
                                // Check if this is a wrapped Ok value
                                if let Some(ok_value) = obj.get("Ok") {
                                    println!("Found Ok value: {:?}", ok_value);
                                    // Try to parse the operation to get field names
                                    if let Ok(Operation::Query { fields, .. }) = serde_json::from_str::<Operation>(&query.operation) {
                                        if !fields.is_empty() {
                                            let mut result_obj = serde_json::Map::new();
                                            
                                            // Get the first field name
                                            let field_name = fields[0].clone();
                                            println!("Using field name from query: {}", field_name);
                                            
                                            // Handle string values
                                            if let Some(field_value) = ok_value.as_str() {
                                                println!("Ok value is a string: {}", field_value);
                                                // Try to parse as JSON first
                                                if let Ok(parsed_value) = serde_json::from_str::<serde_json::Value>(field_value) {
                                                    println!("Parsed string as JSON: {:?}", parsed_value);
                                                    result_obj.insert(field_name, parsed_value);
                                                } else {
                                                    println!("Using as plain string");
                                                    // Use as string if not valid JSON
                                                    result_obj.insert(field_name, serde_json::Value::String(field_value.to_string()));
                                                }
                                            } else {
                                                println!("Ok value is not a string, using directly");
                                                // Handle non-string values directly
                                                result_obj.insert(field_name, ok_value.clone());
                                            }
                                            
                                            let response = ApiSuccessResponse::new(result_obj);
                                            let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                            println!("Final JSON response with field name: {}", json_response);
                                            return Ok(warp::reply::json(&response));
                                        }
                                    }
                                    
                                    // Fallback if we can't get field names
                                    println!("Using generic result key for Ok value");
                                    let response = ApiSuccessResponse::new(json!({
                                        "result": ok_value
                                    }));
                                    let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                    println!("Final JSON response with generic result: {}", json_response);
                                    return Ok(warp::reply::json(&response));
                                }
                                
                                // Return the object directly if it's not a special case
                                println!("Returning object from array directly: {:?}", obj);
                                let response = ApiSuccessResponse::new(obj);
                                let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                println!("Final JSON response for direct object: {}", json_response);
                                Ok(warp::reply::json(&response))
                            },
                            serde_json::Value::String(s) => {
                                println!("Processing string value in array: {}", s);
                                // For string values, try to get the field name from the query
                                if let Ok(Operation::Query { fields, .. }) = serde_json::from_str::<Operation>(&query.operation) {
                                    if !fields.is_empty() {
                                        let field_name = fields[0].clone();
                                        println!("Using field name from query for string: {}", field_name);
                                        let response = ApiSuccessResponse::new(json!({
                                            field_name: s
                                        }));
                                        let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                        println!("Final JSON response for string with field name: {}", json_response);
                                        return Ok(warp::reply::json(&response));
                                    }
                                }
                                
                                // Fallback to generic result
                                println!("Using generic result key for string");
                                let response = ApiSuccessResponse::new(json!({ "result": s }));
                                let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                println!("Final JSON response for string with generic result: {}", json_response);
                                Ok(warp::reply::json(&response))
                            },
                            _ => {
                                println!("Processing other value type in array");
                                // For other types, use a generic result key
                                let response = ApiSuccessResponse::new(json!({ "result": arr[0] }));
                                let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                                println!("Final JSON response for other type: {}", json_response);
                                Ok(warp::reply::json(&response))
                            }
                        }
                    } else {
                        // For multiple results, wrap them in a results array
                        println!("Processing multiple results in array: {:?}", arr);
                        let response = ApiSuccessResponse::new(json!({ "results": arr }));
                        let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                        println!("Final JSON response for multiple results: {}", json_response);
                        Ok(warp::reply::json(&response))
                    }
                },
                _ => {
                    // For any other type, use a generic result key
                    println!("Processing other value type: {:?}", result);
                    let response = ApiSuccessResponse::new(json!({ "result": result }));
                    let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing response: {}\"}}", e));
                    println!("Final JSON response for other type: {}", json_response);
                    Ok(warp::reply::json(&response))
                }
            }
        },
        Err(e) => {
            println!("Operation error: {:?}", e);
            let error_msg = e.to_string();
            println!("Error message: {}", error_msg);
            let response = ApiErrorResponse::new(error_msg);
            let json_response = serde_json::to_string(&response).unwrap_or_else(|e| format!("{{\"error\":\"Error serializing error response: {}\"}}", e));
            println!("Final JSON error response: {}", json_response);
            Ok(warp::reply::json(&response))
        },
    }
}
