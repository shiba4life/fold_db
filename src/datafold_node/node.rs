use log::info;
use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::datafold_node::config::NodeConfig;
use crate::datafold_node::config::NodeInfo;
use crate::error::{FoldDbError, FoldDbResult, NetworkErrorKind};
use crate::fold_db_core::FoldDB;
use crate::network::{NetworkConfig, NetworkCore, PeerId};

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
    pub(super) db: Arc<Mutex<FoldDB>>,
    /// Configuration settings for this node
    pub(super) config: NodeConfig,
    /// Map of trusted nodes and their trust distances
    pub(super) trusted_nodes: HashMap<String, NodeInfo>,
    /// Unique identifier for this node
    pub(super) node_id: String,
    /// Network layer for P2P communication
    pub(super) network: Option<Arc<tokio::sync::Mutex<NetworkCore>>>,
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
    pub async fn load(config: NodeConfig) -> FoldDbResult<Self> {
        info!("Loading DataFoldNode from config");
        let node = Self::new(config)?;

        // Delegate to SchemaCore for unified schema discovery and loading
        {
            let db = node
                .db
                .lock()
                .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
            // Initialize schema system via SchemaCore
            db.schema_manager.discover_and_load_all_schemas().map_err(|e| {
                FoldDbError::Config(format!("Failed to initialize schema system: {}", e))
            })?;
        }

        info!("DataFoldNode loaded successfully with schema system initialized");
        Ok(node)
    }

    /// Get comprehensive schema status for UI
    pub fn get_schema_status(&self) -> FoldDbResult<crate::schema::core::SchemaLoadingReport> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.schema_manager
            .get_schema_status()
            .map_err(|e| FoldDbError::Config(format!("Failed to get schema status: {}", e)))
    }

    /// Refresh schemas from all sources
    pub fn refresh_schemas(&self) -> FoldDbResult<crate::schema::core::SchemaLoadingReport> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.schema_manager
            .discover_and_load_all_schemas()
            .map_err(|e| FoldDbError::Config(format!("Failed to refresh schemas: {}", e)))
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

    /// Gets the unique identifier for this node.
    pub fn get_node_id(&self) -> &str {
        &self.node_id
    }

    /// Restart the node by reinitializing all components
    pub async fn restart(&mut self) -> FoldDbResult<()> {
        info!("Restarting DataFoldNode...");

        // Stop network if it's running
        if self.network.is_some() {
            info!("Stopping network service for restart");
            if let Err(e) = self.stop_network().await {
                log::warn!("Failed to stop network during restart: {}", e);
            }
        }

        // Get the storage path before dropping the old database
        let storage_path = self
            .config
            .storage_path
            .to_str()
            .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?
            .to_string();

        // Properly close the existing database by dropping all references
        info!("Closing existing database");
        let old_db = std::mem::replace(
            &mut self.db,
            Arc::new(Mutex::new(
                // Create a dummy database with a different path to avoid conflicts
                FoldDB::new(&format!("{}_temp", storage_path))?,
            )),
        );

        // Ensure the old database is fully dropped
        drop(old_db);

        // Wait for file system to release locks
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Create new database instance
        info!("Reinitializing database");
        let new_db = Arc::new(Mutex::new(FoldDB::new(&storage_path)?));

        // Update the database reference
        self.db = new_db;

        // Clear network state
        self.network = None;

        // Clear trusted nodes (they can be re-added if needed)
        self.trusted_nodes.clear();

        info!("DataFoldNode restart completed successfully");
        Ok(())
    }

    /// Perform a soft restart that preserves network connections
    pub async fn soft_restart(&mut self) -> FoldDbResult<()> {
        info!("Performing soft restart of DataFoldNode...");

        // Get the storage path before dropping the old database
        let storage_path = self
            .config
            .storage_path
            .to_str()
            .ok_or_else(|| FoldDbError::Config("Invalid storage path".to_string()))?
            .to_string();

        // Properly close the existing database by dropping all references
        info!("Closing existing database");
        let old_db = std::mem::replace(
            &mut self.db,
            Arc::new(Mutex::new(
                // Create a dummy database with a different path to avoid conflicts
                FoldDB::new(&format!("{}_temp", storage_path))?,
            )),
        );

        // Ensure the old database is fully dropped
        drop(old_db);

        // Wait for file system to release locks
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Create new database instance
        info!("Reinitializing database");
        let new_db = Arc::new(Mutex::new(FoldDB::new(&storage_path)?));

        // Update the database reference
        self.db = new_db;

        info!("DataFoldNode soft restart completed successfully");
        Ok(())
    }

    /// Schema Management Methods - Delegate to FoldDB/SchemaCore
    ///
    /// Add a schema to the available schemas list
    pub fn add_schema_available(
        &mut self,
        schema: crate::schema::Schema,
    ) -> crate::error::FoldDbResult<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.add_schema_available(schema)
            .map_err(|e| crate::error::FoldDbError::Config(format!("Failed to add schema: {}", e)))
    }

    /// List all schemas with their states
    pub fn list_schemas_with_state(
        &self,
    ) -> crate::error::FoldDbResult<
        std::collections::HashMap<String, crate::schema::core::SchemaState>,
    > {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        let states = db.schema_manager.load_states();
        Ok(states)
    }

    /// Get a schema by name
    pub fn get_schema(
        &self,
        schema_name: &str,
    ) -> crate::error::FoldDbResult<Option<crate::schema::Schema>> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.get_schema(schema_name)
            .map_err(|e| crate::error::FoldDbError::Config(format!("Failed to get schema: {}", e)))
    }

    /// Load a schema (approve it for use)
    pub fn load_schema(&mut self, schema: crate::schema::Schema) -> crate::error::FoldDbResult<()> {
        // First add as available
        self.add_schema_available(schema.clone())?;
        // Then approve it
        self.approve_schema(&schema.name)
    }

    /// Approve a schema for queries and mutations
    pub fn approve_schema(&mut self, schema_name: &str) -> crate::error::FoldDbResult<()> {
        // First approve the schema
        {
            let db = self
                .db
                .lock()
                .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
            db.schema_manager.approve_schema(schema_name).map_err(|e| {
                crate::error::FoldDbError::Config(format!("Failed to approve schema: {}", e))
            })?;
        }
        
        // Then grant permission for this schema to this node
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        
        let mut current_perms = db.get_schema_permissions(&self.node_id);
        if !current_perms.contains(&schema_name.to_string()) {
            current_perms.push(schema_name.to_string());
            db.set_schema_permissions(&self.node_id, &current_perms)
                .map_err(|e| crate::error::FoldDbError::Config(format!("Failed to set permissions: {}", e)))?;
        }
        
        Ok(())
    }

    /// Unload a schema (set to available state)
    pub fn unload_schema(&self, schema_name: &str) -> crate::error::FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.unload_schema(schema_name).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to unload schema: {}", e))
        })
    }

    /// List all loaded (approved) schemas
    pub fn list_schemas(&self) -> crate::error::FoldDbResult<Vec<String>> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.schema_manager.list_loaded_schemas().map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to list schemas: {}", e))
        })
    }

    /// List all available schemas (any state)
    pub fn list_available_schemas(&self) -> crate::error::FoldDbResult<Vec<String>> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        
        // Use schema_manager (which is Arc<SchemaCore>) to get available schemas
        db.schema_manager.list_available_schemas().map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to list available schemas: {}", e))
        })
    }

    /// Load schema from file
    pub fn load_schema_from_file(&mut self, path: &str) -> crate::error::FoldDbResult<()> {
        let mut db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.load_schema_from_file(path).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to load schema from file: {}", e))
        })
    }

    /// Check if a schema is loaded (approved)
    pub fn is_schema_loaded(&self, schema_name: &str) -> crate::error::FoldDbResult<bool> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        let state = db.schema_manager.get_schema_state(schema_name);
        Ok(matches!(
            state,
            Some(crate::schema::core::SchemaState::Approved)
        ))
    }

    /// List schemas by specific state
    pub fn list_schemas_by_state(
        &self,
        state: crate::schema::core::SchemaState,
    ) -> crate::error::FoldDbResult<Vec<String>> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.schema_manager.list_schemas_by_state(state).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to list schemas by state: {}", e))
        })
    }

    /// Block a schema from queries and mutations
    pub fn block_schema(&mut self, schema_name: &str) -> crate::error::FoldDbResult<()> {
        // Check permissions first
        if !self.check_schema_permission(schema_name)? {
            return Err(crate::error::FoldDbError::Config(format!(
                "Permission denied for schema {}",
                schema_name
            )));
        }

        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.schema_manager.block_schema(schema_name).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to block schema: {}", e))
        })
    }

    /// Get the current state of a schema
    pub fn get_schema_state(
        &self,
        schema_name: &str,
    ) -> crate::error::FoldDbResult<crate::schema::core::SchemaState> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;

        // Check if schema exists
        let exists = db.schema_manager.schema_exists(schema_name).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to check schema existence: {}", e))
        })?;

        if !exists {
            return Err(crate::error::FoldDbError::Config(format!(
                "Schema '{}' not found",
                schema_name
            )));
        }

        // Get state from schema manager
        let states = db.schema_manager.load_states();

        Ok(states
            .get(schema_name)
            .copied()
            .unwrap_or(crate::schema::core::SchemaState::Available))
    }

    /// Set schema permissions for a node (for testing)
    pub fn set_schema_permissions(
        &self,
        node_id: &str,
        schemas: &[String],
    ) -> crate::error::FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.set_schema_permissions(node_id, schemas).map_err(|e| {
            crate::error::FoldDbError::Config(format!("Failed to set schema permissions: {}", e))
        })
    }

    /// Add a new schema from JSON content to the available_schemas directory with validation
    pub fn add_schema_to_available_directory(
        &self,
        json_content: &str,
        schema_name: Option<String>,
    ) -> crate::error::FoldDbResult<String> {
        let db = self
            .db
            .lock()
            .map_err(|_| crate::error::FoldDbError::Config("Cannot lock database mutex".into()))?;

        // Use the schema core method for full validation
        db.schema_manager
            .add_schema_to_available_directory(json_content, schema_name)
            .map_err(|e| crate::error::FoldDbError::Config(format!("Failed to add schema: {}", e)))
    }
}
