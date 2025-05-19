use serde_json::Value;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use log::info;

use crate::datafold_node::config::NodeConfig;
use crate::datafold_node::config::NodeInfo;
use crate::error::{FoldDbError, FoldDbResult, NetworkErrorKind};
use crate::fold_db_core::FoldDB;
use crate::network::{NetworkConfig, NetworkCore, PeerId};
use crate::schema::types::{Mutation, Operation, Query, Transform};
use crate::schema::{Schema, SchemaError};

/// A node in the DataFold distributed database system.
///
/// DataFoldNode combines database storage, schema management, and networking
/// capabilities into a complete node implementation. It can operate independently
/// or as part of a network of nodes, with trust relationships defining data access.
///
/// # Features
///
/// * Schema loading and management
/// * Query and mutation execution
/// * Network communication with other nodes
/// * Permission management for schemas
/// * Request forwarding to trusted nodes
///
/// # Examples
///
/// ```rust,no_run
/// use fold_node::datafold_node::{DataFoldNode, NodeConfig};
/// use fold_node::schema::{Schema, types::Operation};
/// use fold_node::error::FoldDbResult;
/// use std::path::PathBuf;
/// use std::collections::HashMap;
///
/// fn main() -> FoldDbResult<()> {
///     // Create a new node with default configuration
///     let config = NodeConfig::new(PathBuf::from("data"));
///     let mut node = DataFoldNode::new(config)?;
///
///     // Create and load a schema
///     let schema = Schema::new("user_profile".to_string());
///
///     // Load the schema
///     node.load_schema(schema)?;
///
///     // Execute a query
///     let operation = Operation::Query {
///         schema: "user_profile".to_string(),
///         fields: vec!["username".to_string(), "email".to_string()],
///         filter: None,
///     };
///     let result = node.execute_operation(operation)?;
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct DataFoldNode {
    /// The underlying database instance for data storage and operations
    db: Arc<Mutex<FoldDB>>,
    /// Configuration settings for this node
    config: NodeConfig,
    /// Map of trusted nodes and their trust distances
    trusted_nodes: HashMap<String, NodeInfo>,
    /// Unique identifier for this node
    node_id: String,
    /// Network layer for P2P communication
    network: Option<Arc<tokio::sync::Mutex<NetworkCore>>>,
}

/// Basic status information about the network layer
#[derive(Debug, Clone, Serialize)]
pub struct NetworkStatus {
    pub node_id: String,
    pub initialized: bool,
    pub connected_nodes_count: usize,
}

impl DataFoldNode {
    /// Creates a new DataFoldNode with the specified configuration.
    pub fn new(config: NodeConfig) -> FoldDbResult<Self> {
        let db = Arc::new(Mutex::new(FoldDB::new(
            config
                .storage_path
                .to_str()
                .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?,
        )?));

        // Retrieve or generate the persistent node_id from fold_db
        let node_id = {
            let guard = db
                .lock()
                .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
            guard
                .get_node_id()
                .map_err(|e| FoldDbError::Config(format!("Failed to get node_id: {}", e)))?
        };

        Ok(Self {
            db,
            config,
            trusted_nodes: HashMap::new(),
            node_id,
            network: None,
        })
    }

    /// Loads an existing database node from the specified configuration.
    pub fn load(config: NodeConfig) -> FoldDbResult<Self> {
        Self::new(config)
    }

    /// Loads a schema into the database and grants this node permission.
    ///
    /// This function loads a schema into the database, making it available for
    /// queries and mutations. It also grants the local node permission to access
    /// the schema.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema to load
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` indicating success or failure.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * There is an error loading the schema into the database
    /// * There is an error granting permission to the local node
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use fold_node::datafold_node::{DataFoldNode, NodeConfig};
    /// use fold_node::schema::Schema;
    /// use fold_node::error::FoldDbResult;
    /// use std::path::PathBuf;
    /// use std::collections::HashMap;
    ///
    /// fn main() -> FoldDbResult<()> {
    ///     let config = NodeConfig::new(PathBuf::from("data"));
    ///     let mut node = DataFoldNode::new(config)?;
    ///     
    ///     // Create a schema
    ///     let schema = Schema::new("user_profile".to_string());
    ///
    ///     // Load the schema
    ///     node.load_schema(schema)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn load_schema(&mut self, schema: Schema) -> FoldDbResult<()> {
        let schema_name = schema.name.clone();
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.load_schema(schema)?;
        drop(db);
        self.grant_schema_permission(&schema_name)?;
        Ok(())
    }

    /// Executes an operation (query or mutation) on the database.
    ///
    /// This function processes a schema operation, which can be either a query
    /// to retrieve data or a mutation to modify data. It handles permission
    /// checking, operation routing, and result formatting.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation to execute, either a Query or Mutation
    ///
    /// # Returns
    ///
    /// A `FoldDbResult` containing the JSON result of the operation.
    /// For queries, this will be an array of values for the requested fields.
    /// For mutations, this will typically be null or a success indicator.
    ///
    /// # Errors
    ///
    /// Returns a `FoldDbError` if:
    /// * The schema does not exist
    /// * The operation is not permitted
    /// * There is an error executing the operation
    /// * The result cannot be serialized
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use fold_node::datafold_node::{DataFoldNode, NodeConfig};
    /// use fold_node::schema::types::{Operation, MutationType};
    /// use fold_node::error::FoldDbResult;
    /// use std::path::PathBuf;
    /// use serde_json::json;
    ///
    /// fn main() -> FoldDbResult<()> {
    ///     let config = NodeConfig::new(PathBuf::from("data"));
    ///     let mut node = DataFoldNode::new(config)?;
    ///
    ///     // Execute a query
    ///     let query_op = Operation::Query {
    ///         schema: "user_profile".to_string(),
    ///         fields: vec!["username".to_string(), "email".to_string()],
    ///         filter: None,
    ///     };
    ///     let query_result = node.execute_operation(query_op)?;
    ///
    ///     // Execute a mutation
    ///     let mutation_op = Operation::Mutation {
    ///         schema: "user_profile".to_string(),
    ///         data: json!({
    ///             "username": "new_user",
    ///             "email": "user@example.com"
    ///         }),
    ///         mutation_type: MutationType::Create,
    ///     };
    ///     let mutation_result = node.execute_operation(mutation_op)?;
    ///     Ok(())
    /// }
    /// ```
    pub fn execute_operation(&mut self, operation: Operation) -> FoldDbResult<Value> {
        info!("Executing operation: {:?}", operation);
        match operation {
            Operation::Query {
                schema,
                fields,
                filter: _,
            } => {
                let query = Query {
                    schema_name: schema,
                    fields,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: 0,      // Set write distance to 0 for all queries
                };

                let results = self.query(query)?;

                let unwrapped_results: Vec<Value> = results
                    .into_iter()
                    .map(|result| match result {
                        Ok(value) => value,
                        Err(e) => serde_json::json!({"error": e.to_string()}),
                    })
                    .collect();

                Ok(serde_json::to_value(&unwrapped_results)?)
            }
            Operation::Mutation {
                schema,
                data,
                mutation_type,
            } => {
                let fields_and_values = match data {
                    Value::Object(map) => map.into_iter().collect(),
                    _ => {
                        return Err(FoldDbError::Config(
                            "Mutation data must be an object".into(),
                        ))
                    }
                };

                info!("Mutation type: {:?}", mutation_type);

                let mutation = Mutation {
                    schema_name: schema,
                    fields_and_values,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: 0,      // Set write distance to 0 for all mutations
                    mutation_type,
                };

                self.mutate(mutation)?;

                Ok(Value::Null)
            }
        }
    }

    /// Retrieves a schema by its ID.
    pub fn get_schema(&self, schema_id: &str) -> FoldDbResult<Option<Schema>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.schema_manager.get_schema(schema_id)?)
    }

    /// Lists all loaded schemas in the database.
    pub fn list_schemas(&self) -> FoldDbResult<Vec<Schema>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let schema_names = db.schema_manager.list_schemas()?;
        info!("Schema names from schema_manager: {:?}", schema_names);
        let mut schemas = Vec::new();
        for name in schema_names {
            if let Some(schema) = db.schema_manager.get_schema(&name)? {
                schemas.push(schema);
            }
        }
        Ok(schemas)
    }

    /// Executes a query against the database.
    pub fn query(&self, mut query: Query) -> FoldDbResult<Vec<Result<Value, SchemaError>>> {
        // Enforce per-node schema permission
        if !self.check_schema_permission(&query.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                query.schema_name
            )));
        }
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.query_schema(query))
    }

    /// Executes a mutation on the database.
    pub fn mutate(&mut self, mutation: Mutation) -> FoldDbResult<()> {
        // Enforce per-node schema permission
        if !self.check_schema_permission(&mutation.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                mutation.schema_name
            )));
        }
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.write_schema(mutation)?;
        Ok(())
    }

    /// Adds a trusted node to the node's trusted nodes list.
    pub fn add_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.trusted_nodes.insert(
            node_id.to_string(),
            NodeInfo {
                id: node_id.to_string(),
                trust_distance: self.config.default_trust_distance,
            },
        );
        Ok(())
    }

    /// Removes a trusted node from the node's trusted nodes list.
    pub fn remove_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.trusted_nodes.remove(node_id);
        Ok(())
    }

    /// Gets the current list of trusted nodes and their trust distances.
    pub fn get_trusted_nodes(&self) -> &HashMap<String, NodeInfo> {
        &self.trusted_nodes
    }

    /// Retrieves the version history for a specific atom reference.
    pub fn get_history(&self, aref_uuid: &str) -> FoldDbResult<Vec<Value>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let history = db
            .atom_manager
            .get_atom_history(aref_uuid)
            .map_err(|e| FoldDbError::Database(e.to_string()))?;

        Ok(history
            .into_iter()
            .map(|atom| atom.content().clone())
            .collect())
    }

    /// Allows operations on a schema and persists permission for this node.
    pub fn allow_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.allow_schema(schema_name)?;
        // Persist this node's permission
        drop(db); // release lock before acquiring in grant_schema_permission
        self.grant_schema_permission(schema_name)?;
        Ok(())
    }

    /// Grants schema permission for this node.
    pub fn grant_schema_permission(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let mut perms = db.get_schema_permissions(&self.node_id);
        if !perms.contains(&schema_name.to_string()) {
            perms.push(schema_name.to_string());
            db.set_schema_permissions(&self.node_id, &perms)
                .map_err(|e| FoldDbError::Config(format!("Failed to set permissions: {}", e)))?;
        }
        Ok(())
    }

    /// Revokes schema permission for this node.
    pub fn revoke_schema_permission(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let mut perms = db.get_schema_permissions(&self.node_id);
        perms.retain(|s| s != schema_name);
        db.set_schema_permissions(&self.node_id, &perms)
            .map_err(|e| FoldDbError::Config(format!("Failed to set permissions: {}", e)))?;
        Ok(())
    }

    /// Checks if this node has permission to access the given schema.
    pub fn check_schema_permission(&self, schema_name: &str) -> FoldDbResult<bool> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let perms = db.get_schema_permissions(&self.node_id);
        Ok(perms.contains(&schema_name.to_string()))
    }

    /// Removes a schema from the database.
    pub fn remove_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;

        match db.schema_manager.unload_schema(schema_name) {
            Ok(true) => Ok(()),
            Ok(false) => Err(FoldDbError::Config(format!(
                "Schema {} not found",
                schema_name
            ))),
            Err(e) => Err(e.into()),
        }
    }

    /// Initialize the network layer
    pub async fn init_network(&mut self, network_config: NetworkConfig) -> FoldDbResult<()> {
        // Create the network core
        let network_core = NetworkCore::new(network_config)
            .await
            .map_err(|e| FoldDbError::Network(e.into()))?;

        // Set up the schema check callback
        let mut network_core = network_core;
        let db_clone = self.db.clone();

        network_core
            .schema_service_mut()
            .set_schema_check_callback(move |schema_names| {
                let db = match db_clone.lock() {
                    Ok(db) => db,
                    Err(_) => return Vec::new(), // Return empty list if we can't lock the mutex
                };

                schema_names
                    .iter()
                    .filter(|name| {
                        // Check if the schema exists
                        matches!(db.schema_manager.get_schema(name), Ok(Some(_)))
                    })
                    .cloned()
                    .collect()
            });

        // Register the node ID with the network core
        let local_peer_id = network_core.local_peer_id();
        network_core.register_node_id(&self.node_id, local_peer_id);
        info!(
            "Registered node ID {} with peer ID {}",
            self.node_id, local_peer_id
        );

        // Store the network core
        self.network = Some(Arc::new(tokio::sync::Mutex::new(network_core)));

        Ok(())
    }

    /// Start the network service
    pub async fn start_network(&self) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;
            // Use the address from the config
            let address = &self.config.network_listen_address;
            network
                .run(address)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Start the network service with a specific listen address
    pub async fn start_network_with_address(&self, listen_address: &str) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;
            network
                .run(listen_address)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Stop the network service
    pub async fn stop_network(&self) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let mut network_guard = network.lock().await;
            info!("Stopping network service");
            network_guard.stop();
            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Get a mutable reference to the network core
    pub async fn get_network_mut(&self) -> FoldDbResult<tokio::sync::MutexGuard<'_, NetworkCore>> {
        if let Some(network) = &self.network {
            Ok(network.lock().await)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Discover nodes on the local network using mDNS
    pub async fn discover_nodes(&self) -> FoldDbResult<Vec<PeerId>> {
        if let Some(network) = &self.network {
            let network_guard = network.lock().await;

            // Trigger mDNS discovery
            // This will update the known_peers list in the NetworkCore
            info!("Triggering mDNS discovery...");

            // In a real implementation, this would actively scan for peers
            // For now, we'll just return the current known peers
            let known_peers: Vec<PeerId> = network_guard.known_peers().iter().cloned().collect();

            Ok(known_peers)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Get the list of known nodes
    pub async fn get_known_nodes(&self) -> FoldDbResult<HashMap<String, NodeInfo>> {
        if let Some(network) = &self.network {
            let network_guard = network.lock().await;

            // Convert the PeerId set to a HashMap of NodeInfo
            let mut result = HashMap::new();
            for peer_id in network_guard.known_peers() {
                let peer_id_str = peer_id.to_string();

                // If the peer is already in trusted_nodes, use that info
                if let Some(info) = self.trusted_nodes.get(&peer_id_str) {
                    result.insert(peer_id_str, info.clone());
                } else {
                    // Otherwise, create a new NodeInfo with default trust distance
                    result.insert(
                        peer_id_str.clone(),
                        NodeInfo {
                            id: peer_id_str,
                            trust_distance: self.config.default_trust_distance,
                        },
                    );
                }
            }

            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Check which schemas are available on a remote peer
    pub async fn check_remote_schemas(
        &self,
        peer_id_str: &str,
        schema_names: Vec<String>,
    ) -> FoldDbResult<Vec<String>> {
        if let Some(network) = &self.network {
            // Parse the peer ID
            let peer_id = peer_id_str.parse::<PeerId>().map_err(|e| {
                FoldDbError::Network(NetworkErrorKind::Connection(format!(
                    "Invalid peer ID: {}",
                    e
                )))
            })?;

            // Check schemas
            let mut network = network.lock().await;
            let result = network
                .check_schemas(peer_id, schema_names)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Forward a request to another node
    pub async fn forward_request(&self, peer_id: PeerId, request: Value) -> FoldDbResult<Value> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;

            // Get the node ID for this peer if available
            let node_id = network
                .get_node_id_for_peer(&peer_id)
                .unwrap_or_else(|| peer_id.to_string());

            info!("Forwarding request to node {} (peer {})", node_id, peer_id);

            // Use the network layer to forward the request
            let response = network
                .forward_request(peer_id, request)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;

            info!("Received response from node {} (peer {})", node_id, peer_id);

            Ok(response)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol(
                "Network not initialized".to_string(),
            )))
        }
    }

    /// Simple method to connect to another node
    pub async fn connect_to_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.add_trusted_node(node_id)
    }

    /// Retrieve basic network status information
    pub async fn get_network_status(&self) -> FoldDbResult<NetworkStatus> {
        let initialized = self.network.is_some();
        let connected_nodes_count = if let Some(network) = &self.network {
            let guard = network.lock().await;
            guard.known_peers().len()
        } else {
            0
        };
        Ok(NetworkStatus {
            node_id: self.node_id.clone(),
            initialized,
            connected_nodes_count,
        })
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> FoldDbResult<HashMap<String, Transform>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.list_transforms()?)
    }

    /// Execute a transform by id and return the result.
    pub fn run_transform(&mut self, transform_id: &str) -> FoldDbResult<Value> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.run_transform(transform_id)?)
    }

    /// Gets the unique identifier for this node.
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
}
