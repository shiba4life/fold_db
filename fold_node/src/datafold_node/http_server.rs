use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use super::{schema_routes, query_routes, network_routes, system_routes};
use super::log_routes;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpServer as ActixHttpServer};
use log::info;
use std::sync::Arc;
use tokio::sync::Mutex;

/// HTTP server for the DataFold node.
///
/// DataFoldHttpServer provides a web-based interface for external clients to interact
/// with a DataFold node. It handles HTTP requests and can serve the built React
/// UI,
/// and provides REST API endpoints for schemas, queries, and mutations.
///
/// # Features
///
/// * Static file serving for the UI
/// * REST API endpoints for schemas, queries, and mutations
/// * Sample data management
/// * One-click loading of sample data
pub struct DataFoldHttpServer {
    /// The DataFold node
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    /// The HTTP server bind address
    bind_address: String,
}


/// Shared application state for the HTTP server.
pub struct AppState {
    /// The DataFold node
    pub(crate) node: Arc<tokio::sync::Mutex<DataFoldNode>>,
}

impl DataFoldHttpServer {
    /// Create a new HTTP server.
    ///
    /// This method creates a new HTTP server that listens on the specified address.
    /// It uses the provided DataFoldNode to process client requests.
    ///
    /// # Arguments
    ///
    /// * `node` - The DataFoldNode instance to use for processing requests
    /// * `bind_address` - The address to bind to (e.g., "127.0.0.1:9001")
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` containing the new DataFoldHttpServer instance.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * There is an error starting the HTTP server
    pub async fn new(node: DataFoldNode, bind_address: &str) -> FoldDbResult<Self> {
        // Ensure the web logger is initialized so log routes have data
        crate::web_logger::init().ok();

        Ok(Self {
            node: Arc::new(Mutex::new(node)),
            bind_address: bind_address.to_string(),
        })
    }

    /// Run the HTTP server.
    ///
    /// This method starts the HTTP server and begins accepting client connections.
    /// It can serve the compiled React UI and provides REST API endpoints for
    /// schemas, queries, and mutations.
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * There is an error binding to the specified address
    /// * There is an error starting the server
    pub async fn run(&self) -> FoldDbResult<()> {
        info!("HTTP server running on {}", self.bind_address);

        // Create shared application state
        let app_state = web::Data::new(AppState {
            node: self.node.clone(),
        });

        // Start the HTTP server
        let server = ActixHttpServer::new(move || {
            // Create CORS middleware
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            App::new()
                .wrap(cors)
                .app_data(app_state.clone())
                .service(
                    web::scope("/api")
                        // Schema endpoints
                        .route("/schemas", web::get().to(schema_routes::list_schemas))
                        .route("/schemas/status", web::get().to(schema_routes::get_schema_status))
                        .route("/schemas/refresh", web::post().to(schema_routes::refresh_schemas))
                        .route("/schemas/available", web::get().to(schema_routes::list_available_schemas))
                        .route("/schemas/by-state/{state}", web::get().to(schema_routes::list_schemas_by_state))
                        .route("/schema/{name}", web::get().to(schema_routes::get_schema))
                        .route("/schema", web::post().to(schema_routes::create_schema))
                        .route("/schema/{name}", web::put().to(schema_routes::update_schema))
                        .route("/schema/{name}", web::delete().to(schema_routes::unload_schema_route))
                        .route("/schema/{name}/load", web::post().to(schema_routes::load_schema_route))
                        .route("/schema/{name}/approve", web::post().to(schema_routes::approve_schema))
                        .route("/schema/{name}/block", web::post().to(schema_routes::block_schema))
                        .route("/schema/{name}/state", web::get().to(schema_routes::get_schema_state))
                        // Operation endpoints
                        .route("/execute", web::post().to(query_routes::execute_operation))
                        .route("/query", web::post().to(query_routes::execute_query))
                        .route("/mutation", web::post().to(query_routes::execute_mutation))
                        // Sample endpoints
                        .route("/samples/schemas", web::get().to(query_routes::list_schema_samples))
                        .route("/samples/queries", web::get().to(query_routes::list_query_samples))
                        .route("/samples/mutations", web::get().to(query_routes::list_mutation_samples))
                        .route("/samples/schema/{name}", web::get().to(query_routes::get_schema_sample))
                        .route("/samples/query/{name}", web::get().to(query_routes::get_query_sample))
                        .route("/samples/mutation/{name}", web::get().to(query_routes::get_mutation_sample))
                        // Transform endpoints
                        .route("/transforms", web::get().to(query_routes::list_transforms))
                        .route("/transform/{id}/run", web::post().to(query_routes::run_transform))
                        .route("/transforms/queue", web::get().to(query_routes::get_transform_queue))
                        .route("/transforms/queue/{id}", web::post().to(query_routes::add_to_transform_queue))
                        // Log endpoints
                        .route("/logs", web::get().to(log_routes::list_logs))
                        .route("/logs/stream", web::get().to(log_routes::stream_logs))
                        // System endpoints
                        .route("/system/restart", web::post().to(system_routes::restart_node))
                        .route("/system/soft-restart", web::post().to(system_routes::soft_restart_node))
                        .route("/system/status", web::get().to(system_routes::get_system_status))
                        // Network endpoints
                        .service(
                            web::scope("/network")
                                .route("/init", web::post().to(network_routes::init_network))
                                .route("/start", web::post().to(network_routes::start_network))
                                .route("/stop", web::post().to(network_routes::stop_network))
                                .route("/status", web::get().to(network_routes::get_network_status))
                                .route("/connect", web::post().to(network_routes::connect_to_node))
                                .route("/discover", web::post().to(network_routes::discover_nodes))
                                .route("/nodes", web::get().to(network_routes::list_nodes))
                        ),
                )
                // Serve the built React UI if it exists
                .service(Files::new("/", "src/datafold_node/static-react/dist").index_file("index.html"))
       })
        .bind(&self.bind_address)
        .map_err(|e| FoldDbError::Config(format!("Failed to bind HTTP server: {}", e)))?
        .run();

        // Run the server
        server
            .await
            .map_err(|e| FoldDbError::Config(format!("HTTP server error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::DataFoldHttpServer;
    use crate::datafold_node::{DataFoldNode, NodeConfig};
    use serde_json::json;
    use std::net::TcpListener;
    use tempfile::tempdir;

    /// Test the new unified schema status endpoint
    #[tokio::test]
    async fn test_unified_schema_status() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::load(config).await.unwrap();

        // pick an available port
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let bind_addr = format!("127.0.0.1:{}", addr.port());

        let server = DataFoldHttpServer::new(node, &bind_addr)
            .await
            .expect("server init");

        let handle = tokio::spawn(async move { server.run().await.unwrap() });

        // Wait for server to start
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        // Test new unified schema status endpoint
        let client = reqwest::Client::new();
        let url = format!("http://{}/api/schemas/status", bind_addr);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .expect("Failed to connect to server");
        
        assert!(response.status().is_success());
        
        let json_value: serde_json::Value = response.json()
            .await
            .expect("Failed to parse JSON response");
        
        // Verify response structure
        assert!(json_value.get("data").is_some());

        handle.abort();
        let _ = handle.await;
    }

    /// Verify that logs endpoint returns data
    #[tokio::test]
    async fn logs_endpoint_returns_lines() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::new(config).unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let bind_addr = format!("127.0.0.1:{}", addr.port());

        let server = DataFoldHttpServer::new(node, &bind_addr)
            .await
            .expect("server init");

        let handle = tokio::spawn(async move { server.run().await.unwrap() });

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let client = reqwest::Client::new();
        let url = format!("http://{}/api/logs", bind_addr);

        let logs: serde_json::Value = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .expect("request failed")
            .json()
            .await
            .expect("invalid json");

        assert!(logs.as_array().map(|v| !v.is_empty()).unwrap_or(false));

        handle.abort();
        let _ = handle.await;
    }

    /// Ensure sample schemas start unloaded
    #[tokio::test]
    async fn samples_start_unloaded() {
        let temp_dir = tempdir().unwrap();
        let config = NodeConfig::new(temp_dir.path().to_path_buf());
        let node = DataFoldNode::load(config).await.unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let bind_addr = format!("127.0.0.1:{}", addr.port());

        let server = DataFoldHttpServer::new(node, &bind_addr)
            .await
            .expect("server init");

        let node_guard = server.node.lock().await;
        let loaded = node_guard.list_schemas().unwrap();
        assert!(!loaded.iter().any(|s| s == "BlogPost"));
        let available = node_guard.list_available_schemas().unwrap();
        assert!(available.iter().any(|n| n == "BlogPost"));
    }
}


