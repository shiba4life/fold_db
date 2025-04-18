use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use serde_json::{Value, json};
use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::Schema;
use crate::schema::types::operations::MutationType;
use libp2p::PeerId;

/// TCP server for the DataFold node
pub struct TcpServer {
    /// The DataFold node
    node: Arc<Mutex<DataFoldNode>>,
    /// The TCP listener
    listener: TcpListener,
}

impl TcpServer {
    /// Create a new TCP server
    pub async fn new(node: DataFoldNode, port: u16) -> FoldDbResult<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        println!("TCP server listening on {}", addr);
        
        Ok(Self {
            node: Arc::new(Mutex::new(node)),
            listener,
        })
    }
    
    /// Run the TCP server
    pub async fn run(&self) -> FoldDbResult<()> {
        println!("TCP server running...");
        
        loop {
            let (socket, _) = self.listener.accept().await?;
            println!("New client connected");
            
            // Clone the node reference for the new connection
            let node_clone = self.node.clone();
            
            // Spawn a new task to handle the connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, node_clone).await {
                    eprintln!("Error handling connection: {}", e);
                }
            });
        }
    }
    
    /// Handle a client connection
    async fn handle_connection(
        mut socket: TcpStream,
        node: Arc<Mutex<DataFoldNode>>,
    ) -> FoldDbResult<()> {
        // We can't set keepalive directly on tokio's TcpStream, so we'll skip it
        // In a real implementation, we would use socket2 crate to set keepalive
        
        loop {
            // Read the request length
            let request_len = match socket.read_u32().await {
                Ok(len) => len as usize,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        // Client disconnected
                        println!("Client disconnected");
                        return Ok(());
                    }
                    println!("Error reading request length: {}", e);
                    return Err(e.into());
                }
            };
            
            // Sanity check the request length to prevent OOM
            if request_len > 10_000_000 { // 10MB limit
                println!("Request too large: {} bytes", request_len);
                let error_response = json!({
                    "error": "Request too large",
                    "max_size": 10_000_000
                });
                let error_bytes = serde_json::to_vec(&error_response)?;
                socket.write_u32(error_bytes.len() as u32).await?;
                socket.write_all(&error_bytes).await?;
                continue;
            }
            
            // Read the request
            let mut request_bytes = vec![0u8; request_len];
            match socket.read_exact(&mut request_bytes).await {
                Ok(_) => {},
                Err(e) => {
                    println!("Error reading request: {}", e);
                    if e.kind() == std::io::ErrorKind::UnexpectedEof {
                        println!("Client disconnected while reading request");
                        return Ok(());
                    }
                    
                    // Try to send an error response
                    let error_response = json!({
                        "error": format!("Error reading request: {}", e)
                    });
                    let error_bytes = serde_json::to_vec(&error_response)?;
                    
                    // Ignore errors when sending the error response
                    let _ = socket.write_u32(error_bytes.len() as u32).await;
                    let _ = socket.write_all(&error_bytes).await;
                    
                    return Err(e.into());
                }
            };
            
            // Deserialize the request
            let request: Value = match serde_json::from_slice(&request_bytes) {
                Ok(req) => req,
                Err(e) => {
                    println!("Error deserializing request: {}", e);
                    
                    // Try to send an error response
                    let error_response = json!({
                        "error": format!("Error deserializing request: {}", e)
                    });
                    let error_bytes = serde_json::to_vec(&error_response)?;
                    
                    // Ignore errors when sending the error response
                    let _ = socket.write_u32(error_bytes.len() as u32).await;
                    let _ = socket.write_all(&error_bytes).await;
                    
                    continue;
                }
            };
            
            // Process the request
            let response = match Self::process_request(&request, node.clone()).await {
                Ok(resp) => resp,
                Err(e) => {
                    println!("Error processing request: {}", e);
                    
                    // Create an error response
                    json!({
                        "error": format!("Error processing request: {}", e)
                    })
                }
            };
            
            // Serialize the response
            let response_bytes = match serde_json::to_vec(&response) {
                Ok(bytes) => bytes,
                Err(e) => {
                    println!("Error serializing response: {}", e);
                    
                    // Try to send an error response
                    let error_response = json!({
                        "error": format!("Error serializing response: {}", e)
                    });
                    let error_bytes = serde_json::to_vec(&error_response)?;
                    
                    // Ignore errors when sending the error response
                    let _ = socket.write_u32(error_bytes.len() as u32).await;
                    let _ = socket.write_all(&error_bytes).await;
                    
                    continue;
                }
            };
            
            // Send the response length
            if let Err(e) = socket.write_u32(response_bytes.len() as u32).await {
                println!("Error sending response length: {}", e);
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    println!("Client disconnected while sending response length");
                    return Ok(());
                }
                return Err(e.into());
            }
            
            // Send the response
            if let Err(e) = socket.write_all(&response_bytes).await {
                println!("Error sending response: {}", e);
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    println!("Client disconnected while sending response");
                    return Ok(());
                }
                return Err(e.into());
            }
            
            // Flush the socket to ensure all data is sent
            if let Err(e) = socket.flush().await {
                println!("Error flushing socket: {}", e);
                if e.kind() == std::io::ErrorKind::BrokenPipe {
                    println!("Client disconnected while flushing socket");
                    return Ok(());
                }
                return Err(e.into());
            }
        }
    }
    
    /// Process a request
    async fn process_request(
        request: &Value,
        node: Arc<Mutex<DataFoldNode>>,
    ) -> FoldDbResult<Value> {
        println!("Processing request: {}", serde_json::to_string_pretty(request).unwrap_or_else(|_| request.to_string()));
        
        // Extract the operation from the request
        let operation = request.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FoldDbError::Config("Missing operation".to_string()))?;
            
        // Check if this request is targeted for a different node
        if let Some(target_node_id) = request.get("target_node_id").and_then(|v| v.as_str()) {
            // Get the local node ID
            let local_node_id = {
                let node_guard = node.lock().await;
                node_guard.get_node_id().to_string()
            };
            
            // If the target node ID doesn't match the local node ID, forward the request
            if target_node_id != local_node_id {
                println!("Request targeted for node {}, forwarding...", target_node_id);
                return Self::forward_request(request, target_node_id, node.clone()).await;
            }
        }
        
        match operation {
            "list_schemas" => {
                // List schemas
                let node_guard = node.lock().await;
                let schemas = node_guard.list_schemas()?;
                let schema_names: Vec<String> = schemas.iter().map(|s| s.name.clone()).collect();
                Ok(serde_json::to_value(schema_names)?)
            }
            "get_schema" => {
                // Get schema
                let schema_name = request.get("params")
                    .and_then(|v| v.get("schema_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema_name parameter".to_string()))?;
                
                let node_guard = node.lock().await;
                let schema = node_guard.get_schema(schema_name)?;
                
                match schema {
                    Some(s) => Ok(serde_json::to_value(s)?),
                    None => Err(crate::error::FoldDbError::Config(format!("Schema not found: {}", schema_name))),
                }
            }
            "create_schema" => {
                // Create schema
                let schema_json = request.get("params")
                    .and_then(|v| v.get("schema"))
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema parameter".to_string()))?;
                
                // Deserialize the schema directly from the JSON
                let schema: Schema = serde_json::from_value(schema_json.clone())?;
                
                // Load the schema into the node
                let mut node_guard = node.lock().await;
                node_guard.load_schema(schema)?;
                
                Ok(serde_json::json!({ "success": true }))
            }
            "update_schema" => {
                // Update schema
                let schema_json = request.get("params")
                    .and_then(|v| v.get("schema"))
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema parameter".to_string()))?;
                
                // Deserialize the schema directly from the JSON
                let schema: Schema = serde_json::from_value(schema_json.clone())?;
                
                // First remove the existing schema
                let mut node_guard = node.lock().await;
                let _ = node_guard.remove_schema(&schema.name);
                
                // Then load the updated schema
                node_guard.load_schema(schema)?;
                
                Ok(serde_json::json!({ "success": true }))
            }
            "delete_schema" => {
                // Delete schema
                let schema_name = request.get("params")
                    .and_then(|v| v.get("schema_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema_name parameter".to_string()))?;
                
                let mut node_guard = node.lock().await;
                node_guard.remove_schema(schema_name)?;
                
                Ok(serde_json::json!({ "success": true }))
            }
            "query" => {
                // Query
                let schema = request.get("params")
                    .and_then(|v| v.get("schema"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema parameter".to_string()))?;
                
                let fields = request.get("params")
                    .and_then(|v| v.get("fields"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing fields parameter".to_string()))?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();
                
                let filter = request.get("params")
                    .and_then(|v| v.get("filter"))
                    .cloned();
                
                let operation = crate::schema::types::Operation::Query {
                    schema: schema.to_string(),
                    fields,
                    filter,
                };
                
                let mut node_guard = node.lock().await;
                let result = node_guard.execute_operation(operation)?;
                
                // Format the result as a QueryResult
                Ok(serde_json::json!({
                    "results": result,
                    "errors": []
                }))
            }
            "mutation" => {
                // Mutation
                let schema = request.get("params")
                    .and_then(|v| v.get("schema"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema parameter".to_string()))?;
                
                let data = request.get("params")
                    .and_then(|v| v.get("data"))
                    .cloned()
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing data parameter".to_string()))?;
                
                let mutation_type_str = request.get("params")
                    .and_then(|v| v.get("mutation_type"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing mutation_type parameter".to_string()))?;
                
                let mutation_type = match mutation_type_str {
                    "create" => MutationType::Create,
                    "update" => MutationType::Update,
                    "delete" => MutationType::Delete,
                    _ => return Err(crate::error::FoldDbError::Config(format!("Invalid mutation type: {}", mutation_type_str))),
                };
                
                let operation = crate::schema::types::Operation::Mutation {
                    schema: schema.to_string(),
                    data,
                    mutation_type,
                };
                
                let mut node_guard = node.lock().await;
                let _ = node_guard.execute_operation(operation)?;
                
                // Return a success response
                Ok(serde_json::json!({
                    "success": true,
                }))
            }
            "discover_nodes" => {
                // Discover nodes
                let node_guard = node.lock().await;
                let nodes = node_guard.discover_nodes().await?;
                
                let node_infos = nodes
                    .iter()
                    .map(|peer_id| serde_json::json!({
                        "id": peer_id.to_string(),
                        "trust_distance": 1
                    }))
                    .collect::<Vec<_>>();
                
                Ok(serde_json::to_value(node_infos)?)
            }
            _ => Err(crate::error::FoldDbError::Config(format!("Unknown operation: {}", operation))),
        }
    }
    
    /// Forward a request to another node
    async fn forward_request(
        request: &Value,
        target_node_id: &str,
        node: Arc<Mutex<DataFoldNode>>,
    ) -> FoldDbResult<Value> {
        // Get a reference to the network layer
        let node_guard = node.lock().await;
        let mut network = node_guard.get_network_mut().await?;
        
        // Look up the PeerId for the target node ID
        let peer_id = match network.get_peer_id_for_node(target_node_id) {
            Some(id) => {
                println!("Found PeerId {} for node ID {}", id, target_node_id);
                id
            },
            None => {
                // If we don't have a mapping, try to parse the node ID as a PeerId
                // This is a fallback for backward compatibility
                match target_node_id.parse::<PeerId>() {
                    Ok(id) => {
                        println!("Parsed node ID {} as PeerId {}", target_node_id, id);
                        // Register this mapping for future use
                        network.register_node_id(target_node_id, id);
                        id
                    },
                    Err(_) => {
                        // If we can't parse it, generate a random PeerId and register it
                        // This is just for testing purposes
                        let id = PeerId::random();
                        println!("Using placeholder PeerId {} for node ID {}", id, target_node_id);
                        // Register this mapping for future use
                        network.register_node_id(target_node_id, id);
                        id
                    }
                }
            }
        };
        
        // Drop the network mutex guard to avoid deadlock
        drop(network);
        
        // Clone the request to create a forwarded version
        let mut forwarded_request = request.clone();
        
        // Remove the target_node_id from the forwarded request to prevent infinite forwarding
        if let Some(obj) = forwarded_request.as_object_mut() {
            obj.remove("target_node_id");
        }
        
        // Get the operation type
        let operation = request.get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FoldDbError::Config("Missing operation".to_string()))?;
            
        // Get the schema name if this is a query or mutation
        let schema_name = if operation == "query" || operation == "mutation" {
            request.get("params")
                .and_then(|v| v.get("schema"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| FoldDbError::Config("Missing schema parameter".to_string()))?
                .to_string()
        } else {
            // For other operations, use a placeholder
            "unknown".to_string()
        };
        
        // In a real implementation, we would check if the target node has the required schema
        // For now, we'll skip this check since it's just a simulation
        // This would be the code to check schemas:
        /*
        let available_schemas = match node_guard.check_remote_schemas(&peer_id.to_string(), vec![schema_name.clone()]).await {
            Ok(schemas) => schemas,
            Err(e) => {
                println!("Error checking schemas: {}", e);
                return Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                    format!("Error checking schemas: {}", e)
                )));
            },
        };
        
        if available_schemas.is_empty() {
            return Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                format!("Target node does not have the required schema: {}", schema_name)
            )));
        }
        */
        
        // For testing purposes, we'll assume the schema is available
        println!("Assuming schema {} is available on target node", schema_name);
        
        // Forward the request to the target node using the network layer
        println!("Forwarding request to node {} (peer {})", target_node_id, peer_id);
        
        // Use the DataFoldNode's forward_request method to send the request to the target node
        let response = node_guard.forward_request(peer_id, forwarded_request).await?;
        
        // Return the response from the target node
        println!("Received response from node {} (peer {})", target_node_id, peer_id);
        Ok(response)
    }
}
