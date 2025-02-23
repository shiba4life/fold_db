use std::convert::Infallible;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use serde_json::json;
use crate::datafold_node::node::DataFoldNode;
use crate::schema::types::schema::Schema;
use crate::schema_interpreter::types::Operation;

#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub operation: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiSuccessResponse<T: Serialize> {
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: String,
}

impl<T: Serialize> ApiSuccessResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

impl ApiErrorResponse {
    pub fn new(msg: impl Into<String>) -> Self {
        Self { error: msg.into() }
    }
}

pub struct WebServer {
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
}

impl WebServer {
    pub fn new(node: Arc<tokio::sync::Mutex<DataFoldNode>>) -> Self {
        Self { node }
    }

    pub async fn run(&self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let node = Arc::clone(&self.node);

        // API routes
        let api = {
            let node = Arc::clone(&node);
            let list_schemas = warp::path!("api" / "schemas")
                .and(warp::get())
                .and(with_node(node.clone()))
                .and_then(handle_list_schemas);

            let node = Arc::clone(&node);
            let schema = warp::path!("api" / "schema")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_node(node.clone()))
                .and_then(handle_schema);

            let node = Arc::clone(&node);
            let execute = warp::path!("api" / "execute")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_node(node.clone()))
                .and_then(handle_execute);

            let node = Arc::clone(&node);
            let delete_schema = warp::path!("api" / "schema" / String)
                .and(warp::delete())
                .and(with_node(node))
                .and_then(handle_delete_schema);

            list_schemas.or(schema).or(execute).or(delete_schema)
        };

        // Static files
        let index = warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("src/datafold_node/static/index.html"));

        let static_files = warp::path("static")
            .and(warp::fs::dir("src/datafold_node/static"));

        let routes = api.or(index).or(static_files);

        println!("Starting web server on port {}", port);
        let addr = ([127, 0, 0, 1], port);
        println!("Binding to address: {:?}", addr);
        
        println!("Static files configured");
        
        println!("Starting warp server...");
        warp::serve(routes)
            .run(addr)
            .await;
        println!("Warp server stopped");
        Ok(())
    }
}

pub fn with_node(
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> impl Filter<Extract = (Arc<tokio::sync::Mutex<DataFoldNode>>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&node))
}

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
    // Parse the operation string into an Operation
    let operation: Operation = match serde_json::from_str(&query.operation) {
        Ok(op) => op,
        Err(e) => return Ok(warp::reply::json(&ApiErrorResponse::new(
            format!("Invalid operation format: {}", e)
        ))),
    };

    let mut node = node.lock().await;
    let result = node.execute_operation(operation);

    // Print the result for debugging
    println!("Operation result: {:?}", result);

    match result {
        Ok(result) => {
            // Check if the result is actually an error wrapped in Ok
            if let serde_json::Value::Array(arr) = &result {
                if arr.len() == 1 {
                    if let Some(serde_json::Value::Object(obj)) = arr.get(0) {
                        if let Some(err_obj) = obj.get("Err") {
                            if let Some(not_found) = err_obj.as_object().and_then(|o| o.get("NotFound")) {
                                let error_msg = not_found.as_str().unwrap_or("Schema not found").to_string();
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
                serde_json::Value::Object(obj) => Ok(warp::reply::json(&ApiSuccessResponse::new(obj))),
                serde_json::Value::Array(arr) => {
                    if arr.len() == 1 {
                        match &arr[0] {
                            serde_json::Value::Object(obj) => {
                                if let Some(ok_value) = obj.get("Ok") {
                                    if let Some(field_value) = ok_value.as_str() {
                                        // Parse the operation to get the requested field name
                                        if let Ok(Operation::Query { fields, .. }) = serde_json::from_str::<Operation>(&query.operation) {
                                            if let Some(field) = fields.first() {
                                                // Parse the field value as JSON to handle proper serialization
                                                if let Ok(value) = serde_json::from_str::<serde_json::Value>(field_value) {
                                                    return Ok(warp::reply::json(&ApiSuccessResponse::new(json!({
                                                        field: value
                                                    }))))
                                                }
                                                // Fallback to string if not valid JSON
                                                return Ok(warp::reply::json(&ApiSuccessResponse::new(json!({
                                                    field: field_value
                                                }))))
                                            }
                                        }
                                        // Fallback to using the value directly if we can't get the field name
                                        return Ok(warp::reply::json(&ApiSuccessResponse::new(json!({
                                            "value": field_value
                                        }))))
                                    }
                                }
                                Ok(warp::reply::json(&ApiSuccessResponse::new(obj)))
                            },
                            _ => Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "result": arr[0] }))))
                        }
                    } else {
                        Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "results": arr }))))
                    }
                },
                _ => Ok(warp::reply::json(&ApiSuccessResponse::new(json!({ "result": result }))))
            }
        },
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}
