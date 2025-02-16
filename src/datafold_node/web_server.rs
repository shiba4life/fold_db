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

#[derive(Debug, Serialize)]
struct ApiResponse<T> {
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            data: Some(data),
            error: None,
        }
    }

    fn error(msg: impl Into<String>) -> Self {
        Self {
            data: None,
            error: Some(msg.into()),
        }
    }
}

pub struct WebServer {
    node: Arc<DataFoldNode>,
}

impl WebServer {
    pub fn new(node: Arc<DataFoldNode>) -> Self {
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

fn with_node(
    node: Arc<DataFoldNode>,
) -> impl Filter<Extract = (Arc<DataFoldNode>,), Error = Infallible> + Clone {
    warp::any().map(move || Arc::clone(&node))
}

async fn handle_schema(
    schema: Schema,
    node: Arc<DataFoldNode>,
) -> Result<impl Reply, Rejection> {
    let mut node = (*node).clone();
    let result = tokio::task::spawn_blocking(move || {
        node.load_schema(schema)
    }).await;

    match result {
        Ok(Ok(_)) => Ok(warp::reply::json(&ApiResponse::<()>::success(()))),
        Ok(Err(e)) => Ok(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<()>::error(format!("Task error: {}", e)))),
    }
}

async fn handle_execute(
    query: QueryRequest,
    node: Arc<DataFoldNode>,
) -> Result<impl Reply, Rejection> {
    // Parse the operation string into an Operation
    let operation: Operation = match serde_json::from_str(&query.operation) {
        Ok(op) => op,
        Err(e) => return Ok(warp::reply::json(&ApiResponse::<serde_json::Value>::error(
            format!("Invalid operation format: {}", e)
        ))),
    };

    let mut node = (*node).clone();
    let result = tokio::task::spawn_blocking(move || {
        node.execute_operation(operation)
    }).await;

    match result {
        Ok(Ok(result)) => Ok(warp::reply::json(&ApiResponse::success(result))),
        Ok(Err(e)) => Ok(warp::reply::json(&ApiResponse::<serde_json::Value>::error(e.to_string()))),
        Err(e) => Ok(warp::reply::json(&ApiResponse::<serde_json::Value>::error(format!("Task error: {}", e)))),
    }
}
