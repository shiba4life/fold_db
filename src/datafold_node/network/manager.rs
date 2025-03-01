use std::collections::{HashMap, HashSet};

use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::network_core::NetworkCore;
use crate::datafold_node::network::types::{
    NodeId, NodeInfo, NetworkConfig, SchemaInfo, QueryResult
};
use crate::schema::types::Query;

/// Manages the network layer for a DataFold node
/// 
/// This is a simplified wrapper around NetworkCore that provides backward compatibility
/// with the old NetworkManager API.
/// 
/// Note: In a future version, this class could be removed entirely and NetworkCore
/// could be used directly, as it now provides all the functionality needed.
pub struct NetworkManager {
    /// The underlying network core
    core: NetworkCore,
}

impl NetworkManager {
    /// Creates a new network manager
    pub fn new(
        config: NetworkConfig,
        local_node_id: NodeId,
        public_key: Option<String>,
    ) -> NetworkResult<Self> {
        // Create network core
        let core = NetworkCore::new(config, local_node_id, public_key)?;
        
        Ok(Self { core })
    }

    /// Sets the callback for handling schema list requests
    pub fn set_schema_list_callback<F>(&self, callback: F)
    where
        F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static,
    {
        self.core.set_schema_list_callback(callback);
    }

    /// Sets the callback for handling query requests
    pub fn set_query_callback<F>(&self, callback: F)
    where
        F: Fn(Query) -> QueryResult + Send + Sync + 'static,
    {
        self.core.set_query_callback(callback);
    }

    /// Starts the network manager
    pub fn start(&mut self) -> NetworkResult<()> {
        self.core.start()
    }

    /// Stops the network manager
    pub fn stop(&mut self) -> NetworkResult<()> {
        self.core.stop()
    }

    /// Discovers nodes on the network
    pub fn discover_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>> {
        self.core.discover_nodes()
    }

    /// Connects to a node by ID
    pub fn connect_to_node(&self, node_id: &NodeId) -> NetworkResult<()> {
        self.core.connect_to_node(node_id)
    }

    /// Queries a node for data
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> NetworkResult<QueryResult> {
        self.core.query_node(node_id, query)
    }

    /// Lists available schemas on a node
    pub fn list_available_schemas(&self, node_id: &NodeId) -> NetworkResult<Vec<SchemaInfo>> {
        self.core.list_available_schemas(node_id)
    }

    /// Gets the list of connected nodes
    pub fn connected_nodes(&self) -> HashSet<NodeId> {
        self.core.connected_nodes()
    }

    /// Gets the list of known nodes
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        self.core.known_nodes()
    }
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            eprintln!("Error stopping network manager: {}", e);
        }
    }
}
