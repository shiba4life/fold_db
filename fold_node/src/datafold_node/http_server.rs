use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::Schema;
use super::sample_manager::SampleManager;
use super::{schema_routes, query_routes, network_routes};
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
    /// The sample data manager
    #[allow(dead_code)]
    sample_manager: SampleManager,
}


/// Shared application state for the HTTP server.
pub struct AppState {
    /// The DataFold node
    pub(crate) node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    /// The sample data manager
    pub(crate) sample_manager: SampleManager,
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
    /// * There is an error creating the sample manager
    pub async fn new(mut node: DataFoldNode, bind_address: &str) -> FoldDbResult<Self> {
        // Ensure the web logger is initialized so log routes have data
        crate::web_logger::init().ok();

        // Create sample manager
        let sample_manager = SampleManager::new().await?;

        // Load sample schemas into the node in name order to satisfy dependencies
        let mut sample_schemas: Vec<_> = sample_manager.schemas.values().cloned().collect();
        sample_schemas.sort_by_key(|v| v.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string());
        for schema_value in sample_schemas {
            let schema: Schema = serde_json::from_value(schema_value)
                .map_err(|e| FoldDbError::Config(format!("Failed to deserialize sample schema: {}", e)))?;
            info!("Loading sample schema into node: {}", schema.name);
            node.load_schema(schema)?;
        }

        Ok(Self {
            node: Arc::new(Mutex::new(node)),
            bind_address: bind_address.to_string(),
            sample_manager,
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
            sample_manager: self.sample_manager.clone(),
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
                        .route("/schemas/available", web::get().to(schema_routes::list_available_schemas_route))
                        .route("/schema/{name}", web::get().to(schema_routes::get_schema))
                        .route("/schema", web::post().to(schema_routes::create_schema))
                        .route("/schema/{name}", web::put().to(schema_routes::update_schema))
                        .route("/schema/{name}", web::delete().to(schema_routes::unload_schema_route))
                        .route("/schema/load/{name}", web::post().to(schema_routes::load_available_schema_route))
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




