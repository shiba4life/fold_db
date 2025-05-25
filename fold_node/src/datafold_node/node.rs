use serde_json::Value;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::future::Future;
use std::pin::Pin;
use log::info;

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
    async fn with_network<F, R>(&self, f: F) -> FoldDbResult<R>
    where
        F: for<'a> FnOnce(
            &'a mut NetworkCore,
        ) -> Pin<Box<dyn Future<Output = FoldDbResult<R>> + Send + 'a>>,
    {
        match &self.network {
            Some(net) => {
                let mut guard = net.lock().await;
                f(&mut guard).await
            }
            None => Err(FoldDbError::Network(
                NetworkErrorKind::Protocol("Network not initialized".into()),
            )),
        }
    }
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
        let address = self.config.network_listen_address.clone();
        self
            .with_network(|network| Box::pin(async move {
                network
                    .run(&address)
                    .await
                    .map_err(|e| FoldDbError::Network(e.into()))
            }) as Pin<Box<dyn Future<Output = FoldDbResult<()>> + Send + '_>>)
            .await
    }

    /// Start the network service with a specific listen address
    pub async fn start_network_with_address(&self, listen_address: &str) -> FoldDbResult<()> {
        let addr = listen_address.to_string();
        self
            .with_network(|network| Box::pin(async move {
                network
                    .run(&addr)
                    .await
                    .map_err(|e| FoldDbError::Network(e.into()))
            }) as Pin<Box<dyn Future<Output = FoldDbResult<()>> + Send + '_>>)
            .await
    }

    /// Stop the network service
    pub async fn stop_network(&self) -> FoldDbResult<()> {
        self
            .with_network(|network| Box::pin(async move {
                info!("Stopping network service");
                network.stop();
                Ok(())
            }) as Pin<Box<dyn Future<Output = FoldDbResult<()>> + Send + '_>>)
            .await
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
        self
            .with_network(|network| Box::pin(async move {
                // Trigger mDNS discovery
                // This will update the known_peers list in the NetworkCore
                info!("Triggering mDNS discovery...");

                // In a real implementation, this would actively scan for peers
                // For now, we'll just return the current known peers
                let known_peers: Vec<PeerId> = network.known_peers().iter().cloned().collect();
                Ok(known_peers)
            }) as Pin<Box<dyn Future<Output = FoldDbResult<Vec<PeerId>>> + Send + '_>>)
            .await
    }

    /// Get the list of known nodes
    pub async fn get_known_nodes(&self) -> FoldDbResult<HashMap<String, NodeInfo>> {
        let trusted_nodes = self.trusted_nodes.clone();
        let default_distance = self.config.default_trust_distance;
        self
            .with_network(|network| Box::pin(async move {
                let mut result = HashMap::new();
                for peer_id in network.known_peers() {
                    let peer_id_str = peer_id.to_string();
                    if let Some(info) = trusted_nodes.get(&peer_id_str) {
                        result.insert(peer_id_str, info.clone());
                    } else {
                        result.insert(
                            peer_id_str.clone(),
                            NodeInfo {
                                id: peer_id_str,
                                trust_distance: default_distance,
                            },
                        );
                    }
                }
                Ok(result)
            }) as Pin<Box<dyn Future<Output = FoldDbResult<HashMap<String, NodeInfo>>> + Send + '_>>)
            .await
    }

    /// Check which schemas are available on a remote peer
    pub async fn check_remote_schemas(
        &self,
        peer_id_str: &str,
        schema_names: Vec<String>,
    ) -> FoldDbResult<Vec<String>> {
        let peer_id = peer_id_str.parse::<PeerId>().map_err(|e| {
            FoldDbError::Network(NetworkErrorKind::Connection(format!(
                "Invalid peer ID: {}",
                e
            )))
        })?;

        self
            .with_network(|network| Box::pin(async move {
                network
                    .check_schemas(peer_id, schema_names)
                    .await
                    .map_err(|e| FoldDbError::Network(e.into()))
            }) as Pin<Box<dyn Future<Output = FoldDbResult<Vec<String>>> + Send + '_>>)
            .await
    }

    /// Forward a request to another node
    pub async fn forward_request(&self, peer_id: PeerId, request: Value) -> FoldDbResult<Value> {
        self
            .with_network(|network| Box::pin(async move {
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
            }) as Pin<Box<dyn Future<Output = FoldDbResult<Value>> + Send + '_>>)
            .await
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

}
