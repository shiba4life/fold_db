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
            while !*shutdown.lock().unwrap() {
                // Set non-blocking mode for the listener
                listener.set_nonblocking(true).expect("Failed to set non-blocking mode");
                
                match listener.accept() {
                    Ok((stream, _)) => {
                        let node = Arc::clone(&node);
                        thread::spawn(move || {
                            if let Err(e) = handle_connection(stream, node, buffer_size) {
                                eprintln!("Error handling connection: {}", e);
                            }
                        });
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No connection available, sleep briefly before checking again
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        });

        Ok(handle)
    }

    /// Shutdown the server gracefully
    pub fn shutdown(&self) -> std::io::Result<()> {
        // Set shutdown flag
        let mut shutdown = self.shutdown.lock().unwrap();
        *shutdown = true;
        
        // Clean up socket file
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
    let mut reader = BufReader::with_capacity(buffer_size, stream.try_clone()?);
    let mut writer = stream;
    let mut request_data = String::new();

    // Read request
    reader.read_line(&mut request_data)?;

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
        super::types::OperationType::Query => {
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
        super::types::OperationType::Mutation => {
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
        super::types::OperationType::GetSchema => {
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
    let response_json = serde_json::to_vec(&response)?;
    writer.write_all(&response_json)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

/// Verify authentication context
fn verify_auth(auth: &super::types::AuthContext) -> bool {
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

    let query = Query::new(
        schema.to_string(),
        fields,
        payload["pub_key"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        0, // Use node's default trust distance
    );

    let node = node.lock()
        .map_err(|_| "Failed to acquire lock for query".to_string())?;
    
    let results = node.query(query)
        .map_err(|e| e.to_string())?;
    
    // Convert results to JSON
    let result_values: Vec<serde_json::Value> = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect();

    Ok(serde_json::json!({
        "results": result_values
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

    let mutation = Mutation::new(
        schema.to_string(),
        fields_and_values,
        payload["pub_key"]
            .as_str()
            .unwrap_or("default")
            .to_string(),
        0, // Use node's default trust distance
    );

    // Get lock on node for mutation
    let mut node = node.lock()
        .map_err(|_| "Failed to acquire lock for mutation".to_string())?;
    
    node.mutate(mutation)
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
    use std::path::PathBuf;

    #[test]
    fn test_server_creation() {
        let config = SocketConfig {
            socket_path: PathBuf::from("/tmp/test.sock"),
            permissions: 0o660,
            buffer_size: 8192,
        };
        // TODO: Add proper server creation test with mocked DataFoldNode
    }
}
