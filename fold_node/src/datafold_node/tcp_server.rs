use crate::datafold_node::DataFoldNode;
use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::types::operations::MutationType;
use crate::schema::Schema;
use libp2p::PeerId;
use serde_json::{Value, json};
use super::unified_response::UnifiedResponse;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use log::{info, error};

/// TCP server for the DataFold node.
///
/// TcpServer provides a TCP-based interface for external clients to interact
/// with a DataFold node. It handles connection management, request parsing,
/// and response formatting.
///
/// # Features
///
/// * Connection handling for multiple clients
/// * JSON-based request/response protocol
/// * Request forwarding to other nodes
/// * Error handling and recovery
///
/// # Examples
///
/// ```rust,no_run
/// use fold_node::datafold_node::{DataFoldNode, NodeConfig, TcpServer};
/// use fold_node::error::FoldDbResult;
/// use std::path::PathBuf;
///
/// #[tokio::main]
/// async fn main() -> FoldDbResult<()> {
///     // Create a node first
///     let config = NodeConfig {
///         storage_path: PathBuf::from("data"),
///         default_trust_distance: 1,
///         network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
///     };
///     let node = DataFoldNode::new(config)?;
///     
///     // Create a new TCP server on port 9000
///     let tcp_server = TcpServer::new(node, 9000).await?;
///
///     // Run the server (this will block until the server is stopped)
///     tcp_server.run().await?;
///     
///     Ok(())
/// }
/// ```
pub struct TcpServer {
    /// The DataFold node
    node: Arc<Mutex<DataFoldNode>>,
    /// The TCP listener
    listener: TcpListener,
}

impl TcpServer {
    /// Create a new TCP server.
    ///
    /// This method creates a new TCP server that listens on the specified port.
    /// It binds to 127.0.0.1 (localhost) and starts listening for incoming connections.
    /// The server uses the provided DataFoldNode to process client requests.
    ///
    /// # Arguments
    ///
    /// * `node` - The DataFoldNode instance to use for processing requests
    /// * `port` - The port number to listen on
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` containing the new TcpServer instance.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * There is an error binding to the specified port
    /// * The port is already in use
    /// * There is insufficient permission to bind to the port
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use fold_node::datafold_node::{DataFoldNode, NodeConfig, TcpServer};
    /// use fold_node::error::FoldDbResult;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> FoldDbResult<()> {
    ///     let config = NodeConfig {
    ///         storage_path: PathBuf::from("data"),
    ///         default_trust_distance: 1,
    ///         network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    ///     };
    ///     let node = DataFoldNode::new(config)?;
    ///     let tcp_server = TcpServer::new(node, 9000).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(node: DataFoldNode, port: u16) -> FoldDbResult<Self> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).await?;
        info!("TCP server listening on {}", addr);

        // Register this node's address with the network if available
        if let Ok(mut net) = node.get_network_mut().await {
            net.register_node_address(node.get_node_id(), addr.clone());
        }

        Ok(Self {
            node: Arc::new(Mutex::new(node)),
            listener,
        })
    }

    /// Run the TCP server.
    ///
    /// This method starts the TCP server and begins accepting client connections.
    /// It runs in an infinite loop, spawning a new task for each client connection.
    /// Each connection is handled independently, allowing multiple clients to
    /// connect simultaneously.
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * There is an error accepting a connection
    /// * There is an error creating a new task
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use fold_node::datafold_node::{DataFoldNode, NodeConfig, TcpServer};
    /// use fold_node::error::FoldDbResult;
    /// use std::path::PathBuf;
    ///
    /// #[tokio::main]
    /// async fn main() -> FoldDbResult<()> {
    ///     let config = NodeConfig {
    ///         storage_path: PathBuf::from("data"),
    ///         default_trust_distance: 1,
    ///         network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    ///     };
    ///     let node = DataFoldNode::new(config)?;
    ///     let tcp_server = TcpServer::new(node, 9000).await?;
    ///     tcp_server.run().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn run(&self) -> FoldDbResult<()> {
        info!("TCP server running...");

        loop {
            let (socket, _) = self.listener.accept().await?;
            info!("New client connected");

            // Clone the node reference for the new connection
            let node_clone = self.node.clone();

            // Spawn a new task to handle the connection
            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(socket, node_clone).await {
                    error!("Error handling connection: {}", e);
                }
            });
        }
    }


    /// Process a request from a client.
    ///
    /// This function handles the processing of JSON requests from clients.
    /// It extracts the operation type from the request, checks if the request
    /// should be forwarded to another node, and then executes the appropriate
    /// operation on the local node or forwards it to the target node.
    ///
    /// # Arguments
    ///
    /// * `request` - The JSON request to process
    /// * `node` - The DataFoldNode to use for processing the request
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` containing the JSON response to send back to the client.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * The operation is missing from the request
    /// * The operation is unknown or invalid
    /// * There is an error processing the request
    /// * There is an error forwarding the request to another node
    ///
    /// # Supported Operations
    ///
    /// * `list_schemas` - List all schemas loaded in the node
    /// * `get_schema` - Get a specific schema by name
    /// * `create_schema` - Create a new schema
    /// * `update_schema` - Update an existing schema
    /// * `unload_schema` - Unload a schema
    /// * `query` - Execute a query against a schema
    /// * `mutation` - Execute a mutation against a schema
    /// * `discover_nodes` - Discover other nodes in the network
    pub(crate) async fn process_request(
        request: &Value,
        node: Arc<Mutex<DataFoldNode>>,
    ) -> FoldDbResult<Value> {
        info!(
            "Processing request: {}",
            serde_json::to_string_pretty(request).unwrap_or_else(|_| request.to_string())
        );

        // Extract the operation from the request
        let operation = request
            .get("operation")
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
                info!(
                    "Request targeted for node {}, forwarding...",
                    target_node_id
                );
                return Self::forward_request(request, target_node_id, node.clone()).await;
            }
        }

        match operation {
            "list_schemas" => {
                // List loaded schemas
                let node_guard = node.lock().await;
                let schemas = node_guard.list_schemas()?;
                let names: Vec<String> = schemas.iter().map(|s| s.name.clone()).collect();
                Ok(serde_json::to_value(names)?)
            }
            "list_available_schemas" => {
                let node_guard = node.lock().await;
                let names = node_guard.list_available_schemas()?;
                Ok(serde_json::to_value(names)?)
            }
            "get_schema" => {
                // Get schema
                let schema_name = request
                    .get("params")
                    .and_then(|v| v.get("schema_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config(
                            "Missing schema_name parameter".to_string(),
                        )
                    })?;

                let node_guard = node.lock().await;
                let schema = node_guard.get_schema(schema_name)?;

                match schema {
                    Some(s) => Ok(serde_json::to_value(s)?),
                    None => Err(crate::error::FoldDbError::Config(format!(
                        "Schema not found: {}",
                        schema_name
                    ))),
                }
            }
            "create_schema" => {
                // Create schema
                let schema_json = request
                    .get("params")
                    .and_then(|v| v.get("schema"))
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing schema parameter".to_string())
                    })?;

                // Deserialize the schema directly from the JSON
                let schema: Schema = serde_json::from_value(schema_json.clone())?;

                // Load the schema into the node
                let mut node_guard = node.lock().await;
                node_guard.load_schema(schema)?;

                Ok(serde_json::to_value(UnifiedResponse::success(None))?)
            }
            "update_schema" => {
                // Update schema
                let schema_json = request
                    .get("params")
                    .and_then(|v| v.get("schema"))
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing schema parameter".to_string())
                    })?;

                // Deserialize the schema directly from the JSON
                let schema: Schema = serde_json::from_value(schema_json.clone())?;

                // First unload the existing schema
                let mut node_guard = node.lock().await;
                let _ = node_guard.unload_schema(&schema.name);

                // Then load the updated schema
                node_guard.load_schema(schema)?;

                Ok(serde_json::to_value(UnifiedResponse::success(None))?)
            }
            "unload_schema" => {
                // Unload schema
                let schema_name = request
                    .get("params")
                    .and_then(|v| v.get("schema_name"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config(
                            "Missing schema_name parameter".to_string(),
                        )
                    })?;

                let mut node_guard = node.lock().await;
                node_guard.unload_schema(schema_name)?;

                Ok(serde_json::to_value(UnifiedResponse::success(None))?)
            }
            "query" => {
                // Query
                let schema = request
                    .get("params")
                    .and_then(|v| v.get("schema"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing schema parameter".to_string())
                    })?;

                let fields = request
                    .get("params")
                    .and_then(|v| v.get("fields"))
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing fields parameter".to_string())
                    })?
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();

                let filter = request.get("params").and_then(|v| v.get("filter")).cloned();

                let operation = crate::schema::types::Operation::Query {
                    schema: schema.to_string(),
                    fields,
                    filter,
                };

                let mut node_guard = node.lock().await;
                let result = node_guard.execute_operation(operation)?;

                // Format the result as a QueryResult
                Ok(serde_json::to_value(UnifiedResponse::success(Some(json!({
                    "results": result,
                    "errors": []
                }))))?)
            }
            "mutation" => {
                // Mutation
                let schema = request
                    .get("params")
                    .and_then(|v| v.get("schema"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing schema parameter".to_string())
                    })?;

                let data = request
                    .get("params")
                    .and_then(|v| v.get("data"))
                    .cloned()
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config("Missing data parameter".to_string())
                    })?;

                let mutation_type_str = request
                    .get("params")
                    .and_then(|v| v.get("mutation_type"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        crate::error::FoldDbError::Config(
                            "Missing mutation_type parameter".to_string(),
                        )
                    })?;

                let mutation_type = match mutation_type_str {
                    "create" => MutationType::Create,
                    "update" => MutationType::Update,
                    "delete" => MutationType::Delete,
                    _ => {
                        return Err(crate::error::FoldDbError::Config(format!(
                            "Invalid mutation type: {}",
                            mutation_type_str
                        )))
                    }
                };

                let operation = crate::schema::types::Operation::Mutation {
                    schema: schema.to_string(),
                    data,
                    mutation_type,
                };

                let mut node_guard = node.lock().await;
                let _ = node_guard.execute_operation(operation)?;

                // Return a success response
                Ok(serde_json::to_value(UnifiedResponse::success(None))?)
            }
            "discover_nodes" => {
                // Discover nodes
                let node_guard = node.lock().await;
                let nodes = node_guard.discover_nodes().await?;

                let node_infos = nodes
                    .iter()
                    .map(|peer_id| {
                        serde_json::json!({
                            "id": peer_id.to_string(),
                            "trust_distance": 1
                        })
                    })
                    .collect::<Vec<_>>();

                Ok(serde_json::to_value(UnifiedResponse::success(Some(json!(node_infos))))?)
            }
            _ => Err(crate::error::FoldDbError::Config(format!(
                "Unknown operation: {}",
                operation
            ))),
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
                info!("Found PeerId {} for node ID {}", id, target_node_id);
                id
            }
            None => {
                // If we don't have a mapping, try to parse the node ID as a PeerId
                // This is a fallback for backward compatibility
                match target_node_id.parse::<PeerId>() {
                    Ok(id) => {
                        info!("Parsed node ID {} as PeerId {}", target_node_id, id);
                        // Register this mapping for future use
                        network.register_node_id(target_node_id, id);
                        id
                    }
                    Err(_) => {
                        // If we can't parse it, generate a random PeerId and register it
                        // This is just for testing purposes
                        let id = PeerId::random();
                        info!(
                            "Using placeholder PeerId {} for node ID {}",
                            id, target_node_id
                        );
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
        let operation = request
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FoldDbError::Config("Missing operation".to_string()))?;

        // Get the schema name if this is a query or mutation
        let schema_name = if operation == "query" || operation == "mutation" {
            request
                .get("params")
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
                error!("Error checking schemas: {}", e);
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
        info!(
            "Assuming schema {} is available on target node",
            schema_name
        );

        // Forward the request to the target node using the network layer
        info!(
            "Forwarding request to node {} (peer {})",
            target_node_id, peer_id
        );

        // Use the DataFoldNode's forward_request method to send the request to the target node
        let response = node_guard
            .forward_request(peer_id, forwarded_request)
            .await?;

        // Return the response from the target node
        info!(
            "Received response from node {} (peer {})",
            target_node_id, peer_id
        );
        Ok(response)
    }
}
