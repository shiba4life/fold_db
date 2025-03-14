use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::str::FromStr;
use futures::StreamExt;
use tokio::sync::{mpsc, oneshot};
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::error::{FoldDbError, NetworkErrorKind, FoldDbResult};
use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::types::{
    NodeId, NodeInfo, NetworkConfig, SchemaInfo, QueryResult as FoldDbQueryResult
};
use crate::schema::types::{Query, SchemaError};
// Define TrustProof struct here since we removed the message module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustProof {
    pub public_key: String,
    pub signature: String,
    pub trust_distance: u32,
}

/// Message types for libp2p request-response protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
enum LibP2pRequest {
    Query(QueryRequest),
    ListSchemas(ListSchemasRequest),
    Ping(PingRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum LibP2pResponse {
    Query(QueryResponse),
    ListSchemas(ListSchemasResponse),
    Ping(PingResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryRequest {
    query_id: Uuid,
    query: Query,
    trust_proof: TrustProof,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponse {
    query_id: Uuid,
    results: Vec<Result<serde_json::Value, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListSchemasRequest {
    request_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListSchemasResponse {
    request_id: Uuid,
    schemas: Vec<SchemaInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PingRequest {
    ping_id: Uuid,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PingResponse {
    ping_id: Uuid,
    timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ErrorResponse {
    code: u32,
    message: String,
    details: Option<String>,
    related_id: Option<Uuid>,
}

/// LibP2pNetwork implements the network layer using libp2p
pub struct LibP2pNetwork {
    /// Local node ID
    local_node_id: NodeId,
    /// Network configuration
    config: NetworkConfig,
    /// Map of known nodes by ID
    known_nodes: Arc<Mutex<HashMap<NodeId, NodeInfo>>>,
    /// Map of connected nodes by ID
    connected_nodes: Arc<Mutex<HashSet<NodeId>>>,
    /// Callback for handling query requests
    query_callback: Arc<Mutex<Box<dyn Fn(Query) -> FoldDbQueryResult + Send + Sync>>>,
    /// Callback for handling schema list requests
    schema_list_callback: Arc<Mutex<Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync>>>,
    /// Map of pending query responses by ID
    pending_queries: Arc<Mutex<HashMap<Uuid, oneshot::Sender<FoldDbQueryResult>>>>,
    /// Map of pending schema list responses by ID
    pending_schemas: Arc<Mutex<HashMap<Uuid, oneshot::Sender<Vec<SchemaInfo>>>>>,
    /// Whether the network is running
    running: Arc<Mutex<bool>>,
}

impl LibP2pNetwork {
    /// Creates a new LibP2pNetwork instance
    pub fn new(
        config: NetworkConfig,
        local_node_id: Option<NodeId>,
        public_key: Option<String>,
    ) -> NetworkResult<Self> {
        // Generate a random node ID if not provided
        let local_node_id = local_node_id.unwrap_or_else(|| Uuid::new_v4().to_string());

        // Create default callbacks
        let query_callback: Box<dyn Fn(Query) -> FoldDbQueryResult + Send + Sync> = 
            Box::new(|_| Vec::new());
        
        let schema_list_callback: Box<dyn Fn() -> Vec<SchemaInfo> + Send + Sync> = 
            Box::new(|| Vec::new());

        println!("Creating LibP2pNetwork with node ID: {}", local_node_id);
        println!("Public key: {:?}", public_key);

        Ok(Self {
            local_node_id,
            config,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
            connected_nodes: Arc::new(Mutex::new(HashSet::new())),
            query_callback: Arc::new(Mutex::new(query_callback)),
            schema_list_callback: Arc::new(Mutex::new(schema_list_callback)),
            pending_queries: Arc::new(Mutex::new(HashMap::new())),
            pending_schemas: Arc::new(Mutex::new(HashMap::new())),
            running: Arc::new(Mutex::new(false)),
        })
    }

    /// Starts the network
    pub async fn start(&mut self) -> NetworkResult<()> {
        let mut running = self.running.lock().unwrap();
        if *running {
            return Ok(());
        }
        *running = true;
        drop(running);

        println!("Starting LibP2pNetwork with node ID: {}", self.local_node_id);
        println!("Listening on: {}", self.config.listen_address);

        // In a real implementation, this would initialize the libp2p swarm and start listening
        // For now, we'll just simulate it

        Ok(())
    }

    /// Stops the network
    pub async fn stop(&mut self) -> NetworkResult<()> {
        let mut running = self.running.lock().unwrap();
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);

        println!("Stopping LibP2pNetwork with node ID: {}", self.local_node_id);

        // Clear pending queries and schemas
        {
            let mut pending_queries = self.pending_queries.lock().unwrap();
            pending_queries.clear();
        }
        {
            let mut pending_schemas = self.pending_schemas.lock().unwrap();
            pending_schemas.clear();
        }

        Ok(())
    }

    /// Sets the callback for handling query requests
    pub fn set_query_callback<F>(&mut self, callback: F)
    where
        F: Fn(Query) -> FoldDbQueryResult + Send + Sync + 'static,
    {
        let mut cb = self.query_callback.lock().unwrap();
        *cb = Box::new(callback);
    }

    /// Sets the callback for handling schema list requests
    pub fn set_schema_list_callback<F>(&mut self, callback: F)
    where
        F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static,
    {
        let mut cb = self.schema_list_callback.lock().unwrap();
        *cb = Box::new(callback);
    }

    /// Discovers nodes on the network
    pub async fn discover_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>> {
        println!("Discovering nodes on the network");

        // In a real implementation, this would use libp2p's discovery mechanisms
        // For now, we'll just return the known nodes

        let known_nodes = self.known_nodes.lock().unwrap();
        let nodes: Vec<NodeInfo> = known_nodes.values().cloned().collect();

        Ok(nodes)
    }

    /// Connects to a node by ID
    pub async fn connect_to_node(&mut self, node_id: &NodeId) -> NetworkResult<()> {
        println!("Connecting to node: {}", node_id);

        // Get the node info
        let known_nodes = self.known_nodes.lock().unwrap();
        let node_info = known_nodes.get(node_id).cloned().ok_or_else(|| {
            FoldDbError::Network(NetworkErrorKind::Connection(format!("Node {} not found", node_id)))
        })?;

        // In a real implementation, this would use libp2p to establish a connection
        // For now, we'll just add it to the connected nodes

        let mut connected_nodes = self.connected_nodes.lock().unwrap();
        connected_nodes.insert(node_id.clone());

        println!("Connected to node: {}", node_id);

        Ok(())
    }

    /// Queries a node for data
    pub async fn query_node(&self, node_id: &NodeId, query: Query) -> NetworkResult<FoldDbQueryResult> {
        println!("Querying node: {}", node_id);
        println!("Query: {:?}", query);

        // Check if the node is connected
        let connected_nodes = self.connected_nodes.lock().unwrap();
        if !connected_nodes.contains(node_id) {
            return Err(FoldDbError::Network(NetworkErrorKind::Connection(format!("Not connected to node {}", node_id))));
        }

        // In a real implementation, this would use libp2p to send a query to the node
        // For now, we'll just simulate it by executing the query locally

        let callback = self.query_callback.lock().unwrap();
        let result = (*callback)(query.clone());

        Ok(result)
    }

    /// Lists available schemas on a node
    pub async fn list_available_schemas(&self, node_id: &NodeId) -> NetworkResult<Vec<SchemaInfo>> {
        println!("Listing schemas on node: {}", node_id);

        // Check if the node is connected
        let connected_nodes = self.connected_nodes.lock().unwrap();
        if !connected_nodes.contains(node_id) {
            return Err(FoldDbError::Network(NetworkErrorKind::Connection(format!("Not connected to node {}", node_id))));
        }

        // In a real implementation, this would use libp2p to request schemas from the node
        // For now, we'll just simulate it by returning the local schemas

        let callback = self.schema_list_callback.lock().unwrap();
        let schemas = (*callback)();

        Ok(schemas)
    }

    /// Gets the list of connected nodes
    pub fn connected_nodes(&self) -> HashSet<NodeId> {
        self.connected_nodes.lock().unwrap().clone()
    }

    /// Gets the list of known nodes
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        self.known_nodes.lock().unwrap().clone()
    }

    /// Gets the local node ID
    pub fn get_node_id(&self) -> &NodeId {
        &self.local_node_id
    }
}

impl Drop for LibP2pNetwork {
    fn drop(&mut self) {
        let mut running = self.running.lock().unwrap();
        if *running {
            // We can't call async functions in drop, so we just log a warning
            println!("Warning: LibP2pNetwork dropped while running");
            
            // Set running to false to indicate we're stopping
            *running = false;
            
            // Clear pending queries and schemas
            {
                let mut pending_queries = self.pending_queries.lock().unwrap();
                pending_queries.clear();
            }
            {
                let mut pending_schemas = self.pending_schemas.lock().unwrap();
                pending_schemas.clear();
            }
        }
    }
}
