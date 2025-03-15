use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

use crate::error::FoldDbResult;
use crate::datafold_node::network::libp2p_network::LibP2pNetwork;
use crate::datafold_node::network::types::{
    NodeId, NodeInfo, NetworkConfig, SchemaInfo, QueryResult
};
use crate::schema::types::Query;

/// Manages the network layer for a DataFold node using libp2p
/// 
/// This is a wrapper around LibP2pNetwork that provides compatibility
/// with the existing NetworkManager API.
pub struct LibP2pManager {
    /// The underlying libp2p network
    network: Arc<Mutex<LibP2pNetwork>>,
    /// Runtime for executing async tasks
    runtime: tokio::runtime::Handle,
}

impl LibP2pManager {
    /// Creates a new libp2p network manager
    pub fn new(
        config: NetworkConfig,
        local_node_id: NodeId,
        public_key: Option<String>,
    ) -> FoldDbResult<Self> {
        // Create the libp2p network
        let network = LibP2pNetwork::new(config, Some(local_node_id.clone()), public_key)?;
        
        // Get the current runtime handle
        let runtime = tokio::runtime::Handle::current();
        
        Ok(Self {
            network: Arc::new(Mutex::new(network)),
            runtime,
        })
    }

    /// Sets the callback for handling schema list requests
    pub fn set_schema_list_callback<F>(&self, callback: F)
    where
        F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static,
    {
        let mut network = self.network.lock().unwrap();
        network.set_schema_list_callback(callback);
    }

    /// Sets the callback for handling query requests
    pub fn set_query_callback<F>(&self, callback: F)
    where
        F: Fn(Query) -> QueryResult + Send + Sync + 'static,
    {
        let mut network = self.network.lock().unwrap();
        network.set_query_callback(callback);
    }

    /// Starts the network manager
    pub fn start(&mut self) -> FoldDbResult<()> {
        let network = Arc::clone(&self.network);
        
        // Run the async start method in the runtime
        self.runtime.block_on(async {
            let mut network = network.lock().unwrap();
            network.start().await
        })
    }

    /// Stops the network manager
    pub fn stop(&mut self) -> FoldDbResult<()> {
        let network = Arc::clone(&self.network);
        
        // Run the async stop method in the runtime
        self.runtime.block_on(async {
            let mut network = network.lock().unwrap();
            network.stop().await
        })
    }

    /// Discovers nodes on the network
    pub fn discover_nodes(&mut self) -> FoldDbResult<Vec<NodeInfo>> {
        let network = Arc::clone(&self.network);
        
        // Run the async discover_nodes method in the runtime
        self.runtime.block_on(async {
            let mut network = network.lock().unwrap();
            network.discover_nodes().await
        })
    }

    /// Connects to a node by ID
    pub fn connect_to_node(&self, node_id: &NodeId) -> FoldDbResult<()> {
        let network = Arc::clone(&self.network);
        let node_id = node_id.clone();
        
        // Run the async connect_to_node method in the runtime
        self.runtime.block_on(async {
            let mut network = network.lock().unwrap();
            network.connect_to_node(&node_id).await
        })
    }

    /// Queries a node for data
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> FoldDbResult<QueryResult> {
        let network = Arc::clone(&self.network);
        let node_id = node_id.clone();
        
        // Run the async query_node method in the runtime
        self.runtime.block_on(async {
            let network = network.lock().unwrap();
            network.query_node(&node_id, query).await
        })
    }

    /// Lists available schemas on a node
    pub fn list_available_schemas(&self, node_id: &NodeId) -> FoldDbResult<Vec<SchemaInfo>> {
        let network = Arc::clone(&self.network);
        let node_id = node_id.clone();
        
        // Run the async list_available_schemas method in the runtime
        self.runtime.block_on(async {
            let network = network.lock().unwrap();
            network.list_available_schemas(&node_id).await
        })
    }

    /// Gets the list of connected nodes
    pub fn connected_nodes(&self) -> HashSet<NodeId> {
        let network = self.network.lock().unwrap();
        network.connected_nodes()
    }

    /// Gets the list of known nodes
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        let network = self.network.lock().unwrap();
        network.known_nodes()
    }
}
