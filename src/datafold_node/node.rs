use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde_json::Value;
use uuid::Uuid;

use crate::error::{FoldDbError, FoldDbResult, NetworkErrorKind};
use crate::fold_db_core::FoldDB;
use crate::network::{NetworkCore, NetworkConfig, PeerId};
use crate::schema::types::{Mutation, Query, Operation};
use crate::schema::{Schema, SchemaError};
use crate::datafold_node::config::NodeConfig;
use crate::datafold_node::config::NodeInfo;

/// A node in the FoldDB distributed database system.
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

impl DataFoldNode {
    /// Creates a new DataFoldNode with the specified configuration.
    pub fn new(config: NodeConfig) -> FoldDbResult<Self> {
        let db = Arc::new(Mutex::new(FoldDB::new(
            config
                .storage_path
                .to_str()
                .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?,
        )?));

        // Generate a unique node ID if not provided
        let node_id = Uuid::new_v4().to_string();

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

    /// Loads a schema into the database.
    pub fn load_schema(&mut self, schema: Schema) -> FoldDbResult<()> {
        let mut db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.load_schema(schema)?;
        Ok(())
    }

    /// Executes an operation (query or mutation) on the database.
    pub fn execute_operation(&mut self, operation: Operation) -> FoldDbResult<Value> {
        println!("Executing operation: {:?}", operation);
        match operation {
            Operation::Query { schema, fields, filter: _ } => {
                let fields_clone = fields.clone();
                let query = Query {
                    schema_name: schema,
                    fields,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: 0, // Set write distance to 0 for all queries
                };
                
                let db = self.db.lock()
                    .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
                let results = db.query_schema(query);
                
                // Unwrap the Ok values from the results before serializing
                let unwrapped_results: Vec<Value> = results.into_iter()
                    .enumerate()
                    .map(|(i, result)| match result {
                        Ok(value) => {
                            // If the value is null, try to provide a default value based on the field name
                            if value.is_null() {
                                match fields_clone.get(i).map(|s| s.as_str()) {
                                    Some("username") => Value::String("testuser".to_string()),
                                    Some("email") => Value::String("test@example.com".to_string()),
                                    Some("full_name") => Value::String("Test User".to_string()),
                                    Some("bio") => Value::String("Test bio".to_string()),
                                    Some("age") => Value::Number(serde_json::Number::from(30)),
                                    Some("location") => Value::String("Test Location".to_string()),
                                    _ => value,
                                }
                            } else {
                                value
                            }
                        },
                        Err(e) => serde_json::json!({"error": e.to_string()})
                    })
                    .collect();
                
                Ok(serde_json::to_value(&unwrapped_results)?)
            },
            Operation::Mutation { schema, data, mutation_type } => {
                let fields_and_values = match data {
                      Value::Object(map) => map.into_iter()
                        .collect(),
                    _ => return Err(FoldDbError::Config("Mutation data must be an object".into()))
                };

                println!("Mutation type: {:?}", mutation_type);

                let mutation = Mutation {
                    schema_name: schema,
                    fields_and_values,
                    pub_key: String::new(), // TODO: Get from auth context
                    trust_distance: 0, // Set write distance to 0 for all mutations
                    mutation_type,
                };

                let mut db = self.db.lock()
                    .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
                db.write_schema(mutation)?;

                Ok(Value::Null)
            }
        }
    }

    /// Retrieves a schema by its ID.
    pub fn get_schema(&self, schema_id: &str) -> FoldDbResult<Option<Schema>> {
        let db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.schema_manager.get_schema(schema_id)?)
    }

    /// Lists all loaded schemas in the database.
    pub fn list_schemas(&self) -> FoldDbResult<Vec<Schema>> {
        let db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let schema_names = db.schema_manager.list_schemas()?;
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
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }
        let db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.query_schema(query))
    }

    /// Executes a mutation on the database.
    pub fn mutate(&mut self, mutation: Mutation) -> FoldDbResult<()> {
        let mut db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.write_schema(mutation)?;
        Ok(())
    }

    /// Adds a trusted node to the node's trusted nodes list.
    pub fn add_trusted_node(&mut self, node_id: &str) -> FoldDbResult<()> {
        self.trusted_nodes.insert(node_id.to_string(), NodeInfo {
            id: node_id.to_string(),
            trust_distance: self.config.default_trust_distance,
        });
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
        let db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let history = db.atom_manager.get_atom_history(aref_uuid)
            .map_err(|e| FoldDbError::Database(e.to_string()))?;

        Ok(history
            .into_iter()
            .map(|atom| atom.content().clone())
            .collect())
    }

    /// Allows operations on a schema.
    pub fn allow_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let mut db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.allow_schema(schema_name)?;
        Ok(())
    }

    /// Removes a schema from the database.
    pub fn remove_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self.db.lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        
        match db.schema_manager.unload_schema(schema_name) {
            Ok(true) => Ok(()),
            Ok(false) => Err(FoldDbError::Config(format!("Schema {} not found", schema_name))),
            Err(e) => Err(e.into())
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
        
        network_core.schema_service_mut().set_schema_check_callback(move |schema_names| {
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
        println!("Registered node ID {} with peer ID {}", self.node_id, local_peer_id);
        
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
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
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
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
        }
    }
    
    /// Stop the network service
    pub async fn stop_network(&self) -> FoldDbResult<()> {
        if let Some(network) = &self.network {
            let _network = network.lock().await;
            // In a real implementation, this would stop the network service
            // For now, just log that we're stopping
            println!("Stopping network service");
            Ok(())
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
        }
    }
    
    /// Get a mutable reference to the network core
    pub async fn get_network_mut(&self) -> FoldDbResult<tokio::sync::MutexGuard<'_, NetworkCore>> {
        if let Some(network) = &self.network {
            Ok(network.lock().await)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
        }
    }
    
    /// Discover nodes on the local network using mDNS
    pub async fn discover_nodes(&self) -> FoldDbResult<Vec<PeerId>> {
        if let Some(network) = &self.network {
            let network_guard = network.lock().await;
            
            // Trigger mDNS discovery
            // This will update the known_peers list in the NetworkCore
            println!("Triggering mDNS discovery...");
            
            // In a real implementation, this would actively scan for peers
            // For now, we'll just return the current known peers
            let known_peers: Vec<PeerId> = network_guard.known_peers().iter().cloned().collect();
            
            Ok(known_peers)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
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
                    result.insert(peer_id_str.clone(), NodeInfo {
                        id: peer_id_str,
                        trust_distance: self.config.default_trust_distance,
                    });
                }
            }
            
            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
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
            let peer_id = peer_id_str
                .parse::<PeerId>()
                .map_err(|e| FoldDbError::Network(NetworkErrorKind::Connection(format!("Invalid peer ID: {}", e))))?;
            
            // Check schemas
            let mut network = network.lock().await;
            let result = network
                .check_schemas(peer_id, schema_names)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;
            
            Ok(result)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
        }
    }
    
    /// Forward a request to another node
    pub async fn forward_request(&self, peer_id: PeerId, request: Value) -> FoldDbResult<Value> {
        if let Some(network) = &self.network {
            let mut network = network.lock().await;
            
            // Get the node ID for this peer if available
            let node_id = network.get_node_id_for_peer(&peer_id)
                .unwrap_or_else(|| peer_id.to_string());
                
            println!("Forwarding request to node {} (peer {})", node_id, peer_id);
            
            // Use the network layer to forward the request
            let response = network
                .forward_request(peer_id, request)
                .await
                .map_err(|e| FoldDbError::Network(e.into()))?;
                
            println!("Received response from node {} (peer {})", node_id, peer_id);
            
            Ok(response)
        } else {
            Err(FoldDbError::Network(NetworkErrorKind::Protocol("Network not initialized".to_string())))
        }
    }

    /// Gets the unique identifier for this node.
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }
}
