use std::convert::Infallible;
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;
use crate::schema::types::schema::Schema;
use crate::schema_interpreter::types::Operation;

#[derive(Debug, Deserialize)]
struct QueryRequest {
    operation: String,
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
            let schema = warp::path!("api" / "schema")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_node(node.clone()))
                .and_then(handle_schema);

            let node = Arc::clone(&node);
            let execute = warp::path!("api" / "execute")
                .and(warp::post())
                .and(warp::body::json())
                .and(with_node(node))
                .and_then(handle_execute);

            schema.or(execute)
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

async fn handle_execute(
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

    match result {
        Ok(result) => Ok(warp::reply::json(&ApiSuccessResponse::new(result))),
        Err(e) => Ok(warp::reply::json(&ApiErrorResponse::new(e.to_string()))),
    }
}
