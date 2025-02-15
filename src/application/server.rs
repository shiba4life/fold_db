use super::types::{ApiRequest, ApiResponse, ErrorDetails, ResponseStatus, SocketConfig};
use std::os::unix::net::UnixListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::io::{BufRead, BufReader, Write};
use std::fs;
use std::thread;
use std::sync::{Arc, Mutex};
use crate::datafold_node::DataFoldNode;
use serde_json::Value;

/// Server that handles Unix Domain Socket connections
pub struct SocketServer {
    config: SocketConfig,
    node: Arc<Mutex<DataFoldNode>>,
    shutdown: Arc<Mutex<bool>>,
}

impl SocketServer {
    /// Create a new socket server
    pub fn new(config: SocketConfig, node: DataFoldNode) -> std::io::Result<Self> {
        // Wrap node and shutdown flag in Arc and Mutex for shared ownership
        let node = Arc::new(Mutex::new(node));
        let shutdown = Arc::new(Mutex::new(false));
        // Ensure the socket directory exists
        if let Some(parent) = Path::new(&config.socket_path).parent() {
            fs::create_dir_all(parent)?;
        }

        // Remove existing socket file if it exists
        if Path::new(&config.socket_path).exists() {
            fs::remove_file(&config.socket_path)?;
        }

        Ok(Self { config, node, shutdown })
    }

    /// Start the server and listen for connections
    pub fn start(&self) -> std::io::Result<thread::JoinHandle<()>> {
        let listener = UnixListener::bind(&self.config.socket_path)?;
        let shutdown = Arc::clone(&self.shutdown);

        // Set socket permissions
        #[cfg(unix)]
        fs::set_permissions(&self.config.socket_path, fs::Permissions::from_mode(self.config.permissions))?;

        // Clone references for the handler thread
        let node = Arc::clone(&self.node);
        let buffer_size = self.config.buffer_size;

        // Handle connections in a separate thread
        let handle = thread::spawn(move || {
            loop {
                // Check shutdown flag first
                if *shutdown.lock().unwrap() {
                    break;
                }

                // Set non-blocking mode for accept
                if let Err(e) = listener.set_nonblocking(true) {
                    eprintln!("Failed to set non-blocking mode: {}", e);
                    break;
                }

                // Set blocking mode for accept during tests
                #[cfg(test)]
                if let Err(e) = listener.set_nonblocking(false) {
                    eprintln!("Failed to set blocking mode: {}", e);
                    break;
                }

                match listener.accept() {
                    Ok((stream, _)) => {
                        let node = Arc::clone(&node);
                        // Handle connection in current thread to ensure proper request processing
                        if let Err(e) = handle_connection(stream, node, buffer_size) {
                            eprintln!("Error handling connection: {}", e);
                        }
                    }
                    Err(e) => {
                        // Only break on fatal errors, not temporary ones
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock | 
                            std::io::ErrorKind::TimedOut |
                            std::io::ErrorKind::Interrupted |
                            std::io::ErrorKind::ResourceBusy => {
                                // For temporary errors, sleep briefly and continue
                                std::thread::sleep(std::time::Duration::from_millis(100));
                                continue;
                            }
                            _ => {
                                eprintln!("Fatal error accepting connection: {}", e);
                                break;
                            }
                        }
                    }
                }

                // Reset to non-blocking mode after accept during tests
                #[cfg(test)]
                if let Err(e) = listener.set_nonblocking(true) {
                    eprintln!("Failed to reset non-blocking mode: {}", e);
                    break;
                }
            }
        });

        Ok(handle)
    }

    /// Shutdown the server gracefully
    pub fn shutdown(&self) -> std::io::Result<()> {
        // Set shutdown flag first
        {
            let mut shutdown = self.shutdown.lock().unwrap();
            *shutdown = true;
        }

        // Clean up socket file to prevent new connections
        if Path::new(&self.config.socket_path).exists() {
            fs::remove_file(&self.config.socket_path)?;
        }

        Ok(())
    }
}

impl Drop for SocketServer {
    fn drop(&mut self) {
        // Attempt to clean up on drop
        let _ = self.shutdown();
    }
}

/// Handle an individual client connection
fn handle_connection(
    stream: std::os::unix::net::UnixStream,
    node: Arc<Mutex<DataFoldNode>>,
    buffer_size: usize,
) -> std::io::Result<()> {
    // Use a scope to ensure streams are properly dropped
    {
        let mut reader = BufReader::with_capacity(buffer_size, stream.try_clone()?);
        let mut writer = stream;
        let mut request_data = String::new();

        // Read request with a newline delimiter
        reader.read_line(&mut request_data)?;
        request_data = request_data.trim().to_string();

        // Parse request
        let request: ApiRequest = match serde_json::from_str(&request_data) {
            Ok(req) => req,
            Err(e) => {
                let response = ApiResponse {
                    request_id: String::new(),
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(ErrorDetails {
                        code: "PARSE_ERROR".into(),
                        message: e.to_string(),
                    }),
                };
                send_response(&mut writer, response)?;
                return Ok(());
            }
        };

        // Process request
        let response = process_request(request, &node);
        send_response(&mut writer, response)?;
    }

    Ok(())
}

/// Process an API request
fn process_request(request: ApiRequest, node: &Arc<Mutex<DataFoldNode>>) -> ApiResponse {
    let request_id = request.request_id.clone();

    // Verify authentication
    if !verify_auth(&request.auth) {
        return ApiResponse {
            request_id,
            status: ResponseStatus::Error,
            data: None,
            error: Some(ErrorDetails {
                code: "AUTH_ERROR".into(),
                message: "Authentication failed".into(),
            }),
        };
    }

    // Handle operation
    match request.operation_type {
        crate::application::types::OperationType::Query => {
            match handle_query(request.payload, node) {
                Ok(data) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Success,
                    data: Some(data),
                    error: None,
                },
                Err(e) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(ErrorDetails {
                        code: "QUERY_ERROR".into(),
                        message: e,
                    }),
                },
            }
        }
        crate::application::types::OperationType::Mutation => {
            match handle_mutation(request.payload, node) {
                Ok(data) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Success,
                    data: Some(data),
                    error: None,
                },
                Err(e) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(ErrorDetails {
                        code: "MUTATION_ERROR".into(),
                        message: e,
                    }),
                },
            }
        }
        crate::application::types::OperationType::GetSchema => {
            match handle_get_schema(request.payload, node) {
                Ok(data) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Success,
                    data: Some(data),
                    error: None,
                },
                Err(e) => ApiResponse {
                    request_id,
                    status: ResponseStatus::Error,
                    data: None,
                    error: Some(ErrorDetails {
                        code: "SCHEMA_ERROR".into(),
                        message: e,
                    }),
                },
            }
        }
    }
}

/// Send an API response
fn send_response(writer: &mut impl Write, response: ApiResponse) -> std::io::Result<()> {
    let response_json = serde_json::to_string(&response)?;
    writer.write_all(response_json.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

/// Verify authentication context
fn verify_auth(auth: &crate::application::types::AuthContext) -> bool {
    // TODO: Implement proper authentication verification
    // For now, accept any non-empty public key
    !auth.public_key.is_empty()
}

/// Handle a query operation
fn handle_query(payload: serde_json::Value, node: &Arc<Mutex<DataFoldNode>>) -> Result<serde_json::Value, String> {
    use crate::schema::types::Query;

    let schema = payload["schema"].as_str().ok_or("Missing schema")?;
    let fields: Vec<String> = payload["fields"]
        .as_array()
        .ok_or("Missing fields array")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let node_guard = node.lock()
        .map_err(|_| "Failed to acquire lock for query".to_string())?;
    
    // Clone fields for later use
    let field_names = fields.clone();
    
    let query = Query::new(
        schema.to_string(),
        fields,
        payload["pub_key"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        0, // Use 0 to let node apply its default trust distance
    );

    let results = node_guard.query(query)
        .map_err(|e| e.to_string())?;
    
    // Combine field results into a single object
    let mut combined_result = serde_json::Map::new();
    
    // Zip the field names with their results
    for (field, result) in field_names.iter().zip(results.into_iter()) {
        match result {
            Ok(value) => {
                combined_result.insert(field.clone(), value);
            },
            Err(e) => {
                return Err(format!("Error getting field {}: {}", field, e));
            }
        }
    }

    Ok(serde_json::json!({
        "results": [combined_result]
    }))
}

/// Handle a mutation operation
fn handle_mutation(payload: serde_json::Value, node: &Arc<Mutex<DataFoldNode>>) -> Result<serde_json::Value, String> {
    use crate::schema::types::Mutation;
    use std::collections::HashMap;

    let schema = payload["schema"].as_str().ok_or("Missing schema")?;
    let data = payload["data"].as_object().ok_or("Missing or invalid data object")?;

    // Convert data to HashMap<String, Value>
    let fields_and_values: HashMap<String, Value> = data.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    // Get lock on node for mutation
    let mut node_guard = node.lock()
        .map_err(|_| "Failed to acquire lock for mutation".to_string())?;
    
    let mutation = Mutation::new(
        schema.to_string(),
        fields_and_values,
        payload["pub_key"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        0, // Use 0 to let node apply its default trust distance
    );
    
    node_guard.mutate(mutation)
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true
    }))
}

/// Handle a get schema operation
fn handle_get_schema(payload: serde_json::Value, node: &Arc<Mutex<DataFoldNode>>) -> Result<serde_json::Value, String> {
    // Try to get schema_id from payload, falling back to direct schema name if not found
    let schema_id = payload["schema_id"]
        .as_str()
        .or_else(|| payload.as_str())
        .ok_or("Missing schema identifier")?;

    let node = node.lock()
        .map_err(|_| "Failed to acquire lock for schema retrieval".to_string())?;
    
    let schema = node.get_schema(schema_id)
        .map_err(|e| e.to_string())?;
    
    Ok(match schema {
        Some(s) => serde_json::json!({ "schema": s }),
        None => serde_json::json!({ "error": "Schema not found" }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::schema::types::Schema;

    // Mock DataFoldNode for testing
    struct MockNode {
        temp_dir: tempfile::TempDir,
    }

    impl MockNode {
        fn new() -> Self {
            Self {
                temp_dir: tempdir().expect("Failed to create temp dir"),
            }
        }

        fn into_node(self) -> DataFoldNode {
            let config = crate::datafold_node::NodeConfig {
                storage_path: self.temp_dir.path().to_path_buf(),
                default_trust_distance: 1,
                docker: crate::datafold_node::DockerConfig::default(),
            };
            DataFoldNode::new(config).expect("Failed to create node")
        }
    }

    #[test]
    fn test_server_creation() {
        // Create temporary directory for socket
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test.sock");

        let config = SocketConfig {
            socket_path,
            permissions: 0o660,
            buffer_size: 8192,
        };

        // Create mock node and convert to DataFoldNode
        let mock = MockNode::new();
        let mut node = mock.into_node();

        // Create a schema directly on the node before starting server
        let mut schema = Schema::new("test_schema".to_string());
        // Create a permissive policy that allows all operations
        let policy = crate::permissions::types::policy::PermissionsPolicy::new(
            crate::permissions::types::policy::TrustDistance::NoRequirement,
            crate::permissions::types::policy::TrustDistance::NoRequirement,
        );
        let field = crate::schema::types::fields::SchemaField::new(
            policy,
            crate::fees::types::config::FieldPaymentConfig::default(),
        ).with_ref_atom_uuid("test_field_ref".to_string());
        schema.add_field("test_field".to_string(), field);
        node.load_schema(schema).expect("Failed to create schema");

        // Create and start server
        let server = SocketServer::new(config.clone(), node).expect("Failed to create server");
        let handle = server.start().expect("Failed to start server");

        // Verify socket file exists with correct permissions
        let metadata = std::fs::metadata(&config.socket_path).expect("Socket file not created");
        #[cfg(unix)]
        assert_eq!(metadata.permissions().mode() & 0o777, config.permissions);

        // Clean up
        server.shutdown().expect("Failed to shutdown server");
        handle.join().expect("Server thread panicked");
        
        // Verify socket file is cleaned up
        assert!(!config.socket_path.exists(), "Socket file not cleaned up");
    }

    #[test]
    fn test_server_request_handling() {
        // Create temporary directory for socket
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let socket_path = temp_dir.path().join("test.sock");

        let config = SocketConfig {
            socket_path: socket_path.clone(),
            permissions: 0o660,
            buffer_size: 8192,
        };

        // Create mock node and convert to DataFoldNode
        let mock = MockNode::new();
        let mut node = mock.into_node();

        // Create a schema directly on the node before starting server
        let mut schema = Schema::new("test_schema".to_string());
        // Create a permissive policy that allows all operations
        let policy = crate::permissions::types::policy::PermissionsPolicy::new(
            crate::permissions::types::policy::TrustDistance::NoRequirement,
            crate::permissions::types::policy::TrustDistance::NoRequirement,
        );
        let field = crate::schema::types::fields::SchemaField::new(
            policy,
            crate::fees::types::config::FieldPaymentConfig::default(),
        ).with_ref_atom_uuid("test_field_ref".to_string());
        schema.add_field("test_field".to_string(), field);
        node.load_schema(schema).expect("Failed to create schema");

        // Create and start server
        let server = SocketServer::new(config, node).expect("Failed to create server");
        let _handle = server.start().expect("Failed to start server");

        // First create a mutation request to add data
        let mutation_request = ApiRequest {
            request_id: "test-mutation".to_string(),
            operation_type: crate::application::types::OperationType::Mutation,
            auth: crate::application::types::AuthContext {
                public_key: "test-key".to_string(),
            },
            payload: serde_json::json!({
                "schema": "test_schema",
                "data": {
                    "test_field": "test_value"
                }
            }),
        };

        // Function to send request and get response
        let send_request = |request: &ApiRequest| -> ApiResponse {
            let mut stream = std::os::unix::net::UnixStream::connect(&socket_path)
                .expect("Failed to connect to socket");
            
            stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))
                .expect("Failed to set read timeout");

            let request_json = serde_json::to_string(request).unwrap() + "\n";
            stream.write_all(request_json.as_bytes()).expect("Failed to write request");
            stream.flush().expect("Failed to flush stream");

            // Small delay to allow server to process
            std::thread::sleep(std::time::Duration::from_millis(100));

            let mut reader = BufReader::new(stream);
            let mut response_data = String::new();
            reader.read_line(&mut response_data).expect("Failed to read response");

            serde_json::from_str(&response_data.trim())
                .expect("Failed to parse response")
        };

        // Send mutation request
        let mutation_response = send_request(&mutation_request);
        assert_eq!(mutation_response.status, ResponseStatus::Success);

        // Send query request
        let query_request = ApiRequest {
            request_id: "test-query".to_string(),
            operation_type: crate::application::types::OperationType::Query,
            auth: crate::application::types::AuthContext {
                public_key: "test-key".to_string(),
            },
            payload: serde_json::json!({
                "schema": "test_schema",
                "fields": ["test_field"]
            }),
        };

        // Send query request and verify response
        let query_response = send_request(&query_request);
        assert_eq!(query_response.request_id, "test-query");
        assert_eq!(query_response.status, ResponseStatus::Success);
        assert!(query_response.error.is_none());
        
        // Verify the returned data
        let data = query_response.data.expect("No data in response");
        let results = data.get("results").expect("No results in data").as_array().expect("Results is not an array");
        assert_eq!(results.len(), 1);
        let result = &results[0];
        assert_eq!(result["test_field"].as_str().unwrap(), "test_value");

        // Clean up
        server.shutdown().expect("Failed to shutdown server");
    }
}
