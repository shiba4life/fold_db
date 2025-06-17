use super::signature_auth::SignatureVerificationState;
use super::{crypto_routes, key_rotation_routes, log_routes};
use super::{network_routes, query_routes, schema_routes, system_routes};
use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use crate::ingestion::routes as ingestion_routes;

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
    pub node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    /// Signature verification state (mandatory)
    pub signature_auth: Arc<SignatureVerificationState>,
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
        // Initialize the enhanced logging system
        if let Err(e) = crate::logging::LoggingSystem::init_default().await {
            log::warn!(
                "Failed to initialize enhanced logging system, falling back to web logger: {}",
                e
            );
            // Fall back to basic logging for backward compatibility
            crate::logging::init().ok();
        }

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

        // Initialize signature verification (always enabled)
        let signature_auth = {
            let node = self.node.lock().await;
            let sig_config = node.config.signature_auth_config();
            info!(
                "Initializing signature verification middleware with {:?} security profile",
                sig_config.security_profile
            );
            match SignatureVerificationState::new(sig_config.clone()) {
                Ok(state) => {
                    info!("Signature verification middleware initialized successfully");
                    Arc::new(state)
                }
                Err(e) => {
                    log::error!("Failed to initialize signature verification: {}", e);
                    return Err(e);
                }
            }
        };

        // Create shared application state
        let app_state = web::Data::new(AppState {
            node: self.node.clone(),
            signature_auth,
        });

        // Start the HTTP server
        let server = ActixHttpServer::new(move || {
            // Create CORS middleware
            let cors = Cors::default()
                .allow_any_origin()
                .allow_any_method()
                .allow_any_header()
                .max_age(3600);

            let app = App::new().wrap(cors).app_data(app_state.clone());

            // Apply signature verification middleware to all API route scopes (mandatory)
            info!("Activating signature verification middleware for all API route scopes");

            app.service(
                web::scope("/api")
                    // Apply signature verification middleware at main API scope level
                    .wrap(super::signature_auth::SignatureVerificationMiddleware::new(
                        (*app_state.signature_auth).clone(),
                    ))
                    // Schema endpoints - all protected except where exempted by should_skip_verification
                    .route("/schemas", web::get().to(schema_routes::list_schemas))
                    .route(
                        "/schemas/status",
                        web::get().to(schema_routes::get_schema_status),
                    )
                    .route(
                        "/schemas/refresh",
                        web::post().to(schema_routes::refresh_schemas),
                    )
                    .route(
                        "/schemas/available",
                        web::get().to(schema_routes::list_available_schemas),
                    )
                    .route(
                        "/schemas/available/add",
                        web::post().to(schema_routes::add_schema_to_available),
                    )
                    .route(
                        "/schemas/by-state/{state}",
                        web::get().to(schema_routes::list_schemas_by_state),
                    )
                    .route("/schema/{name}", web::get().to(schema_routes::get_schema))
                    .route("/schema", web::post().to(schema_routes::create_schema))
                    .route(
                        "/schema/{name}",
                        web::delete().to(schema_routes::unload_schema_route),
                    )
                    .route(
                        "/schema/{name}/load",
                        web::post().to(schema_routes::load_schema_route),
                    )
                    .route(
                        "/schema/{name}/approve",
                        web::post().to(schema_routes::approve_schema),
                    )
                    .route(
                        "/schema/{name}/block",
                        web::post().to(schema_routes::block_schema),
                    )
                    .route(
                        "/schema/{name}/state",
                        web::get().to(schema_routes::get_schema_state),
                    )
                    // Operation endpoints - all protected
                    .route("/execute", web::post().to(query_routes::execute_operation))
                    .route("/query", web::post().to(query_routes::execute_query))
                    .route("/mutation", web::post().to(query_routes::execute_mutation))
                    // Transform endpoints - all protected
                    .route("/transforms", web::get().to(query_routes::list_transforms))
                    .route(
                        "/transform/{id}/run",
                        web::post().to(query_routes::run_transform),
                    )
                    .route(
                        "/transforms/queue",
                        web::get().to(query_routes::get_transform_queue),
                    )
                    .route(
                        "/transforms/queue/{id}",
                        web::post().to(query_routes::add_to_transform_queue),
                    )
                    // Log endpoints - all protected
                    .route("/logs", web::get().to(log_routes::list_logs))
                    .route("/logs/stream", web::get().to(log_routes::stream_logs))
                    .route("/logs/config", web::get().to(log_routes::get_config))
                    .route(
                        "/logs/config/reload",
                        web::post().to(log_routes::reload_config),
                    )
                    .route("/logs/features", web::get().to(log_routes::get_features))
                    .route(
                        "/logs/level",
                        web::put().to(log_routes::update_feature_level),
                    )
                    // System endpoints - /api/system/status is exempted by should_skip_verification
                    .service(
                        web::scope("/system")
                            .route("/status", web::get().to(system_routes::get_system_status))
                            .route(
                                "/reset-database",
                                web::post().to(system_routes::reset_database),
                            ),
                    )
                    // Ingestion endpoints - all protected
                    .service(
                        web::scope("/ingestion")
                            .route("/process", web::post().to(ingestion_routes::process_json))
                            .route("/status", web::get().to(ingestion_routes::get_status))
                            .route("/health", web::get().to(ingestion_routes::health_check))
                            .route("/config", web::get().to(ingestion_routes::get_config))
                            .route("/validate", web::post().to(ingestion_routes::validate_json))
                            .route(
                                "/openrouter-config",
                                web::get().to(ingestion_routes::get_openrouter_config),
                            )
                            .route(
                                "/openrouter-config",
                                web::post().to(ingestion_routes::save_openrouter_config),
                            ),
                    )
                    // Crypto endpoints - /api/crypto/keys/register is exempted by should_skip_verification
                    .service(
                        web::scope("/crypto")
                            .route(
                                "/init/random",
                                web::post().to(crypto_routes::init_random_key),
                            )
                            .route(
                                "/init/passphrase",
                                web::post().to(crypto_routes::init_passphrase_key),
                            )
                            .route("/status", web::get().to(crypto_routes::get_crypto_status))
                            .route(
                                "/validate",
                                web::post().to(crypto_routes::validate_crypto_config),
                            )
                            .route(
                                "/keys/register",
                                web::post().to(crypto_routes::register_public_key),
                            )
                            .route(
                                "/keys/status/{client_id}",
                                web::get().to(crypto_routes::get_public_key_status),
                            )
                            .route(
                                "/signatures/verify",
                                web::post().to(crypto_routes::verify_signature),
                            )
                            // Key rotation endpoints - all protected with mandatory signature verification
                            .route(
                                "/keys/rotate",
                                web::post().to(key_rotation_routes::rotate_key),
                            )
                            .route(
                                "/keys/rotate/status",
                                web::post().to(key_rotation_routes::get_rotation_status),
                            )
                            .route(
                                "/keys/rotate/history",
                                web::post().to(key_rotation_routes::get_rotation_history),
                            )
                            .route(
                                "/keys/rotate/stats",
                                web::get().to(key_rotation_routes::get_rotation_statistics),
                            )
                            .route(
                                "/keys/rotate/validate",
                                web::post().to(key_rotation_routes::validate_rotation_request),
                            ),
                    )
                    // Network endpoints - all protected
                    .service(
                        web::scope("/network")
                            .route("/init", web::post().to(network_routes::init_network))
                            .route("/start", web::post().to(network_routes::start_network))
                            .route("/stop", web::post().to(network_routes::stop_network))
                            .route("/status", web::get().to(network_routes::get_network_status))
                            .route("/connect", web::post().to(network_routes::connect_to_node))
                            .route("/discover", web::post().to(network_routes::discover_nodes))
                            .route("/nodes", web::get().to(network_routes::list_nodes)),
                    ),
            )
            // Serve the built React UI if it exists
            .service(
                Files::new("/", "src/datafold_node/static-react/dist").index_file("index.html"),
            )
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
    use crate::cli::auth::{CliAuthProfile, CliRequestSigner, CliSigningConfig};
    use crate::crypto::ed25519::generate_master_keypair;
    use crate::datafold_node::{DataFoldNode, NodeConfig};
    use reqwest::{Client, Response};
    use serde_json::json;
    use std::collections::HashMap;
    use std::net::TcpListener;
    use tempfile::tempdir;

    /// Helper function to create an authenticated HTTP client for testing
    async fn create_authenticated_client(server_base_url: &str) -> (Client, CliRequestSigner) {
        // Generate a test keypair
        let keypair = generate_master_keypair().expect("Failed to generate test keypair");

        // Create a test authentication profile
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        let profile = CliAuthProfile {
            client_id: "test-client-123".to_string(),
            key_id: "test-key".to_string(),
            user_id: Some("test-user".to_string()),
            server_url: server_base_url.to_string(),
            metadata,
        };

        // Create signing config
        let signing_config = CliSigningConfig::default();

        // Create the request signer (recreate keypair from secret bytes for ownership)
        let keypair_copy =
            crate::crypto::ed25519::MasterKeyPair::from_secret_bytes(&keypair.secret_key_bytes())
                .expect("Failed to recreate keypair");
        let signer = CliRequestSigner::new(keypair_copy, profile.clone(), signing_config);

        // Create a basic HTTP client
        let client = Client::new();

        // Register the public key with the server (skip signature verification for registration)
        let registration_request = json!({
            "client_id": profile.client_id,
            "public_key": hex::encode(keypair.public_key().to_bytes()),
            "metadata": profile.metadata
        });

        let registration_response = client
            .post(format!("{}/api/crypto/keys/register", server_base_url))
            .json(&registration_request)
            .send()
            .await
            .expect("Failed to register public key");

        assert!(
            registration_response.status().is_success(),
            "Public key registration failed: {}",
            registration_response.status()
        );

        (client, signer)
    }

    /// Helper function to make an authenticated request
    async fn make_authenticated_request(
        client: &Client,
        signer: &CliRequestSigner,
        method: &str,
        url: &str,
    ) -> Result<Response, Box<dyn std::error::Error>> {
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => return Err(format!("Unsupported HTTP method: {}", method).into()),
        };

        // Add required content-type header (even for GET requests since it's required by signature auth)
        request_builder = request_builder.header("content-type", "application/json");

        let mut request = request_builder.build()?;

        // Sign the request
        signer.sign_request(&mut request)?;

        // Execute the signed request
        let response = client.execute(request).await?;
        Ok(response)
    }

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

        // Create authenticated client
        let server_base_url = format!("http://{}", bind_addr);
        let (client, signer) = create_authenticated_client(&server_base_url).await;

        // Test new unified schema status endpoint with authentication
        let url = format!("{}/api/schemas/status", server_base_url);
        let response = make_authenticated_request(&client, &signer, "GET", &url)
            .await
            .expect("Failed to make authenticated request");

        assert!(response.status().is_success());

        let json_value: serde_json::Value = response
            .json()
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

        // Create authenticated client
        let server_base_url = format!("http://{}", bind_addr);
        let (client, signer) = create_authenticated_client(&server_base_url).await;

        // Test logs endpoint with authentication
        let url = format!("{}/api/logs", server_base_url);
        let response = make_authenticated_request(&client, &signer, "GET", &url)
            .await
            .expect("Failed to make authenticated request");

        assert!(response.status().is_success());

        let logs: serde_json::Value = response.json().await.expect("invalid json");

        // After architectural simplification, logs may be empty since web_logger was removed
        assert!(logs.as_array().is_some());

        handle.abort();
        let _ = handle.await;
    }
}
