use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::types::Operation;
use crate::schema::Schema;
use crate::network::NetworkConfig;

use actix_cors::Cors;
use actix_files::Files;
use actix_web::{web, App, HttpResponse, HttpServer as ActixHttpServer, Responder};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs;

/// HTTP server for the DataFold node.
///
/// DataFoldHttpServer provides a web-based interface for external clients to interact
/// with a DataFold node. It handles HTTP requests, serves static files for the UI,
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

/// Sample data manager for the HTTP server.
///
/// SampleManager provides access to sample schemas, queries, and mutations
/// for one-click loading in the UI.
pub struct SampleManager {
    /// Sample schemas
    schemas: HashMap<String, Value>,
    /// Sample queries
    queries: HashMap<String, Value>,
    /// Sample mutations
    mutations: HashMap<String, Value>,
}

impl SampleManager {
    /// Create a new sample manager.
    pub async fn new() -> Self {
        let mut manager = Self {
            schemas: HashMap::new(),
            queries: HashMap::new(),
            mutations: HashMap::new(),
        };

        // Load sample data
        manager.load_samples().await;

        manager
    }

    /// Load sample data from files.
    async fn load_samples(&mut self) {
        let samples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/datafold_node/samples/data");

        let mut entries = match fs::read_dir(&samples_dir).await {
            Ok(e) => e,
            Err(_) => return,
        };

        while let Ok(Some(entry)) = entries.next_entry().await {
            if let Ok(ft) = entry.file_type().await {
                if !ft.is_file() {
                    continue;
                }
            }

            if let Ok(content) = fs::read_to_string(entry.path()).await {
                if let Ok(value) = serde_json::from_str::<Value>(&content) {
                    let name = entry
                        .file_name()
                        .to_string_lossy()
                        .trim_end_matches(".json")
                        .to_string();

                    match value.get("type").and_then(|v| v.as_str()) {
                        Some("query") => {
                            self.queries.insert(name, value);
                        }
                        Some("mutation") => {
                            self.mutations.insert(name, value);
                        }
                        _ => {
                            self.schemas.insert(name, value);
                        }
                    }
                }
            }
        }
    }

    /// Get a sample schema by name.
    pub fn get_schema_sample(&self, name: &str) -> Option<&Value> {
        self.schemas.get(name)
    }

    /// Get a sample query by name.
    pub fn get_query_sample(&self, name: &str) -> Option<&Value> {
        self.queries.get(name)
    }

    /// Get a sample mutation by name.
    pub fn get_mutation_sample(&self, name: &str) -> Option<&Value> {
        self.mutations.get(name)
    }

    /// List all sample schemas.
    pub fn list_schema_samples(&self) -> Vec<String> {
        self.schemas.keys().cloned().collect()
    }

    /// List all sample queries.
    pub fn list_query_samples(&self) -> Vec<String> {
        self.queries.keys().cloned().collect()
    }

    /// List all sample mutations.
    pub fn list_mutation_samples(&self) -> Vec<String> {
        self.mutations.keys().cloned().collect()
    }
}

/// Shared application state for the HTTP server.
struct AppState {
    /// The DataFold node
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    /// The sample data manager
    sample_manager: SampleManager,
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
        // Create sample manager
        let sample_manager = SampleManager::new().await;

        // Load sample schemas into the node
        for (_, schema_value) in sample_manager.schemas.iter() {
            let schema: Schema = serde_json::from_value(schema_value.clone())
                .map_err(|e| FoldDbError::Config(format!("Failed to deserialize sample schema: {}", e)))?;
            println!("Loading sample schema into node: {}", schema.name);
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
    /// It serves static files for the UI and provides REST API endpoints for
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
        println!("HTTP server running on {}", self.bind_address);

        // Create shared application state
        let app_state = web::Data::new(AppState {
            node: self.node.clone(),
            sample_manager: SampleManager::new().await,
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
                .route("/", web::get().to(|| async {
                    HttpResponse::Ok().body(include_str!("static/index.html"))
                }))
                .service(
                    web::scope("/api")
                        // Schema endpoints
                        .route("/schemas", web::get().to(list_schemas))
                        .route("/schema/{name}", web::get().to(get_schema))
                        .route("/schema", web::post().to(create_schema))
                        .route("/schema/{name}", web::put().to(update_schema))
                        .route("/schema/{name}", web::delete().to(delete_schema))
                        // Operation endpoints
                        .route("/execute", web::post().to(execute_operation))
                        .route("/query", web::post().to(execute_query))
                        .route("/mutation", web::post().to(execute_mutation))
                        // Sample endpoints
                        .route("/samples/schemas", web::get().to(list_schema_samples))
                        .route("/samples/queries", web::get().to(list_query_samples))
                        .route("/samples/mutations", web::get().to(list_mutation_samples))
                        .route("/samples/schema/{name}", web::get().to(get_schema_sample))
                        .route("/samples/query/{name}", web::get().to(get_query_sample))
                        .route("/samples/mutation/{name}", web::get().to(get_mutation_sample))
                        // Network endpoints
                        .service(
                            web::scope("/network")
                                .route("/init", web::post().to(init_network))
                                .route("/start", web::post().to(start_network))
                                .route("/stop", web::post().to(stop_network))
                                .route("/status", web::get().to(get_network_status))
                                .route("/connect", web::post().to(connect_to_node))
                                .route("/discover", web::post().to(discover_nodes))
                                .route("/nodes", web::get().to(list_nodes))
                        ),
                )
                // Static files
                .service(Files::new("/static", "src/datafold_node/static").index_file("index.html"))
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

/// List all schemas.
async fn list_schemas(state: web::Data<AppState>) -> impl Responder {
    println!("Received request to list schemas");
    let node_guard = state.node.lock().await;

    match node_guard.list_schemas() {
        Ok(schemas) => {
            println!("Successfully listed schemas: {:?}", schemas);
            // Wrap the schemas in a data field to match frontend expectations
            HttpResponse::Ok().json(json!({
                "data": schemas
            }))
        },
        Err(e) => {
            println!("Failed to list schemas: {}", e);
            HttpResponse::InternalServerError().json(json!({
                "error": format!("Failed to list schemas: {}", e)
            }))
        },
    }
}

/// Get a schema by name.
async fn get_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let node_guard = state.node.lock().await;

    match node_guard.get_schema(&name) {
        Ok(Some(schema)) => HttpResponse::Ok().json(schema),
        Ok(None) => HttpResponse::NotFound().json(json!({
            "error": format!("Schema '{}' not found", name)
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to get schema: {}", e)
        })),
    }
}

/// Create a new schema.
async fn create_schema(schema: web::Json<Schema>, state: web::Data<AppState>) -> impl Responder {
    let mut node_guard = state.node.lock().await;

    match node_guard.load_schema(schema.into_inner()) {
        Ok(_) => HttpResponse::Created().json(json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to create schema: {}", e)
        })),
    }
}

/// Update an existing schema.
async fn update_schema(
    path: web::Path<String>,
    schema: web::Json<Schema>,
    state: web::Data<AppState>,
) -> impl Responder {
    let name = path.into_inner();
    let schema_data = schema.into_inner();

    // Check if the schema name matches the path
    if schema_data.name != name {
        return HttpResponse::BadRequest().json(json!({
            "error": format!("Schema name '{}' does not match path '{}'", schema_data.name, name)
        }));
    }

    let mut node_guard = state.node.lock().await;

    // First remove the existing schema
    let _ = node_guard.remove_schema(&name);

    // Then load the updated schema
    match node_guard.load_schema(schema_data) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to update schema: {}", e)
        })),
    }
}

/// Delete a schema.
async fn delete_schema(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    let mut node_guard = state.node.lock().await;

    match node_guard.remove_schema(&name) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to delete schema: {}", e)
        })),
    }
}

/// Execute an operation (query or mutation).
#[derive(Deserialize)]
struct OperationRequest {
    operation: String,
}

async fn execute_operation(
    request: web::Json<OperationRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let operation_str = &request.operation;
    
    // Parse the operation
    let operation: Operation = match serde_json::from_str(operation_str) {
        Ok(op) => op,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to parse operation: {}", e)
            }));
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => HttpResponse::Ok().json(json!({
            "data": result
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to execute operation: {}", e)
        })),
    }
}

/// Execute a query.
async fn execute_query(query: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    // Parse the query as an Operation
    let operation = match serde_json::from_value::<Operation>(query.into_inner()) {
        Ok(op) => {
            match op {
                Operation::Query { .. } => op,
                _ => {
                    return HttpResponse::BadRequest().json(json!({
                        "error": "Expected a query operation"
                    }));
                }
            }
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to parse query: {}", e)
            }));
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(result) => HttpResponse::Ok().json(json!({
            "data": result
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to execute query: {}", e)
        })),
    }
}

/// Execute a mutation.
async fn execute_mutation(mutation: web::Json<Value>, state: web::Data<AppState>) -> impl Responder {
    // Parse the mutation as an Operation
    let operation = match serde_json::from_value::<Operation>(mutation.into_inner()) {
        Ok(op) => {
            match op {
                Operation::Mutation { .. } => op,
                _ => {
                    return HttpResponse::BadRequest().json(json!({
                        "error": "Expected a mutation operation"
                    }));
                }
            }
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Failed to parse mutation: {}", e)
            }));
        }
    };

    let mut node_guard = state.node.lock().await;

    match node_guard.execute_operation(operation) {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true
        })),
        Err(e) => HttpResponse::InternalServerError().json(json!({
            "error": format!("Failed to execute mutation: {}", e)
        })),
    }
}

/// List all sample schemas.
async fn list_schema_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(state.sample_manager.list_schema_samples())
}

/// List all sample queries.
async fn list_query_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(state.sample_manager.list_query_samples())
}

/// List all sample mutations.
async fn list_mutation_samples(state: web::Data<AppState>) -> impl Responder {
    HttpResponse::Ok().json(state.sample_manager.list_mutation_samples())
}

/// Get a sample schema by name.
async fn get_schema_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    
    match state.sample_manager.get_schema_sample(&name) {
        Some(schema) => HttpResponse::Ok().json(schema),
        None => HttpResponse::NotFound().json(json!({
            "error": format!("Sample schema '{}' not found", name)
        })),
    }
}

/// Get a sample query by name.
async fn get_query_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    
    match state.sample_manager.get_query_sample(&name) {
        Some(query) => HttpResponse::Ok().json(query),
        None => HttpResponse::NotFound().json(json!({
            "error": format!("Sample query '{}' not found", name)
        })),
    }
}

/// Get a sample mutation by name.
async fn get_mutation_sample(path: web::Path<String>, state: web::Data<AppState>) -> impl Responder {
    let name = path.into_inner();
    
    match state.sample_manager.get_mutation_sample(&name) {
        Some(mutation) => HttpResponse::Ok().json(mutation),
        None => HttpResponse::NotFound().json(json!({
            "error": format!("Sample mutation '{}' not found", name)
        })),
    }
}

#[derive(Deserialize)]
struct NetworkConfigPayload {
    listen_address: String,
    discovery_port: Option<u16>,
    max_connections: Option<usize>,
    connection_timeout_secs: Option<u64>,
    announcement_interval_secs: Option<u64>,
    enable_discovery: Option<bool>,
}

#[derive(Deserialize)]
struct ConnectRequest {
    node_id: String,
}

async fn init_network(
    config: web::Json<NetworkConfigPayload>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut node = state.node.lock().await;

    let mut network_config = NetworkConfig::new(&config.listen_address);
    if let Some(port) = config.discovery_port {
        network_config = network_config.with_discovery_port(port);
    }
    if let Some(max) = config.max_connections {
        network_config = network_config.with_max_connections(max);
    }
    if let Some(timeout) = config.connection_timeout_secs {
        network_config =
            network_config.with_connection_timeout(std::time::Duration::from_secs(timeout));
    }
    if let Some(interval) = config.announcement_interval_secs {
        network_config =
            network_config.with_announcement_interval(std::time::Duration::from_secs(interval));
    }
    if let Some(enable) = config.enable_discovery {
        network_config = network_config.with_mdns(enable);
    }

    match node.init_network(network_config).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to init network: {}", e) })),
    }
}

async fn start_network(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.start_network().await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to start network: {}", e) })),
    }
}

async fn stop_network(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.stop_network().await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to stop network: {}", e) })),
    }
}

async fn get_network_status(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.get_network_status().await {
        Ok(status) => HttpResponse::Ok().json(json!({ "data": status })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to get network status: {}", e) })),
    }
}

async fn connect_to_node(
    req: web::Json<ConnectRequest>,
    state: web::Data<AppState>,
) -> impl Responder {
    let mut node = state.node.lock().await;
    match node.connect_to_node(&req.node_id).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "success": true })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to connect to node: {}", e) })),
    }
}

async fn discover_nodes(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.discover_nodes().await {
        Ok(peers) => {
            let peers: Vec<String> = peers.into_iter().map(|p| p.to_string()).collect();
            HttpResponse::Ok().json(json!({ "data": peers }))
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to discover nodes: {}", e) })),
    }
}

async fn list_nodes(state: web::Data<AppState>) -> impl Responder {
    let node = state.node.lock().await;
    match node.get_known_nodes().await {
        Ok(nodes) => HttpResponse::Ok().json(json!({ "data": nodes })),
        Err(e) => HttpResponse::InternalServerError()
            .json(json!({ "error": format!("Failed to list nodes: {}", e) })),
    }
}

#[cfg(test)]
mod tests {
    use super::SampleManager;

    /// Ensure the sample manager loads schema samples from disk.
    #[tokio::test]
    async fn sample_manager_loads_schemas() {
        let manager = SampleManager::new().await;
        let schemas = manager.list_schema_samples();
        assert!(schemas.contains(&"UserProfile".to_string()));
        assert!(schemas.contains(&"ProductCatalog".to_string()));
    }
}