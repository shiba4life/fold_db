use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use serde_json::Value;
use crate::datafold_node::DataFoldNode;
use crate::error::FoldDbResult;
use crate::schema::Schema;
use crate::schema::types::operations::MutationType;
use crate::schema::types::fields::{SchemaField, FieldType};
use crate::permissions::types::policy::PermissionsPolicy;
use crate::fees::FieldPaymentConfig;
use std::collections::HashMap;

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
                    return Err(e.into());
                }
            };
            
            // Read the request
            let mut request_bytes = vec![0u8; request_len];
            socket.read_exact(&mut request_bytes).await?;
            
            // Deserialize the request
            let request: Value = serde_json::from_slice(&request_bytes)?;
            
            // Process the request
            let response = Self::process_request(&request, node.clone()).await?;
            
            // Serialize the response
            let response_bytes = serde_json::to_vec(&response)?;
            
            // Send the response length
            socket.write_u32(response_bytes.len() as u32).await?;
            
            // Send the response
            socket.write_all(&response_bytes).await?;
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
                
                // Extract schema name and fields
                let schema_name = schema_json.get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing schema name".to_string()))?
                    .to_string();
                
                let fields_array = schema_json.get("fields")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| crate::error::FoldDbError::Config("Missing fields array".to_string()))?;
                
                // Create a new schema
                let mut schema = Schema::new(schema_name);
                
                // Convert fields array to HashMap
                for field_json in fields_array {
                    let field_name = field_json.get("name")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| crate::error::FoldDbError::Config("Missing field name".to_string()))?
                        .to_string();
                    
                    // Create a field with default permissions and payment config
                    let field = SchemaField::new(
                        PermissionsPolicy::default(),
                        FieldPaymentConfig::default(),
                        HashMap::new(),
                        Some(FieldType::Single),
                    );
                    
                    // Add the field to the schema
                    schema.add_field(field_name, field);
                }
                
                let mut node_guard = node.lock().await;
                node_guard.load_schema(schema)?;
                
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
                    "id": "123" // TODO: Return the actual ID
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
}
