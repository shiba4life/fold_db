use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};

use crate::datafold_node::network::error::NetworkResult;
use crate::datafold_node::network::connection_manager::ConnectionManager;
use crate::datafold_node::network::message_router::{MessageRouter, MessageHandler};
use crate::datafold_node::network::query_service::QueryService;
use crate::datafold_node::network::schema_service::SchemaService;
use crate::datafold_node::network::discovery::{NodeDiscovery, DiscoveryConfig};
use crate::datafold_node::network::types::{
    NodeId, NodeInfo, NetworkConfig, SchemaInfo, QueryResult
};
use crate::schema::types::Query;

/// Core network component that coordinates all network operations
pub struct NetworkCore {
    /// Connection manager for handling connections
    connection_manager: ConnectionManager,
    /// Message router for routing messages
    #[allow(dead_code)]
    message_router: Arc<MessageRouter>,
    /// Query service for handling query operations
    query_service: Arc<QueryService>,
    /// Schema service for handling schema operations
    schema_service: Arc<SchemaService>,
    /// Node discovery service
    discovery: Arc<Mutex<NodeDiscovery>>,
    /// Network configuration
    #[allow(dead_code)]
    config: NetworkConfig,
    /// Local node ID
    #[allow(dead_code)]
    local_node_id: NodeId,
}

impl NetworkCore {
    /// Creates a new network core
    pub fn new(
        config: NetworkConfig,
        local_node_id: NodeId,
        public_key: Option<String>,
    ) -> NetworkResult<Self> {
        // Create local node info for discovery
        let local_node_info = NodeDiscovery::create_local_node_info(
            local_node_id.clone(),
            config.listen_address,
            0, // Trust distance to self is 0
            public_key,
        );
        
        // Create discovery config
        let discovery_config = DiscoveryConfig {
            discovery_port: config.discovery_port,
            announcement_interval: config.announcement_interval,
            enable_discovery: config.enable_discovery,
            local_node_info,
        };
        
        // Create discovery service
        let discovery = NodeDiscovery::new(discovery_config)?;
        
        // Create connection manager
        let connection_manager = ConnectionManager::new(
            config.clone(),
            local_node_id.clone(),
        );
        
        // Create services
        let query_service = Arc::new(QueryService::new());
        let schema_service = Arc::new(SchemaService::new());
        
        // Register handlers with the message router
        let mut router = MessageRouter::new();
        router.register_handler(Arc::clone(&query_service) as Arc<dyn MessageHandler>);
        router.register_handler(Arc::clone(&schema_service) as Arc<dyn MessageHandler>);
        
        let message_router = Arc::new(router);
        
        Ok(Self {
            connection_manager,
            message_router,
            query_service,
            schema_service,
            discovery: Arc::new(Mutex::new(discovery)),
            config,
            local_node_id,
        })
    }

    /// Starts the network core
    pub fn start(&mut self) -> NetworkResult<()> {
        // Start discovery service
        {
            let mut discovery = self.discovery.lock().unwrap();
            discovery.start()?;
        }
        
        // Start connection manager
        self.connection_manager.start()?;
        
        Ok(())
    }

    /// Stops the network core
    pub fn stop(&mut self) -> NetworkResult<()> {
        // Stop discovery service
        {
            let mut discovery = self.discovery.lock().unwrap();
            discovery.stop();
        }
        
        // Stop connection manager
        self.connection_manager.stop()?;
        
        Ok(())
    }

    /// Sets the callback for handling query requests
    pub fn set_query_callback<F>(&self, callback: F)
    where
        F: Fn(Query) -> QueryResult + Send + Sync + 'static,
    {
        // Create a new service with the callback
        let mut service = QueryService::new();
        service.set_query_callback(callback);
        let service_arc = Arc::new(service);
        
        // Update the message router with the new service
        let mut router = MessageRouter::new();
        router.register_handler(Arc::clone(&service_arc) as Arc<dyn MessageHandler>);
        
        // Note: In a real implementation, we would need to update self.query_service
        // and self.message_router, but since they are immutable here, we're just
        // demonstrating the pattern. In a production system, we would need to use
        // interior mutability or a different design to allow updating these components.
    }

    /// Sets the callback for handling schema list requests
    pub fn set_schema_list_callback<F>(&self, callback: F)
    where
        F: Fn() -> Vec<SchemaInfo> + Send + Sync + 'static,
    {
        // Create a new service with the callback
        let mut service = SchemaService::new();
        service.set_schema_list_callback(callback);
        let service_arc = Arc::new(service);
        
        // Update the message router with the new service
        let mut router = MessageRouter::new();
        router.register_handler(Arc::clone(&service_arc) as Arc<dyn MessageHandler>);
        
        // Note: In a real implementation, we would need to update self.schema_service
        // and self.message_router, but since they are immutable here, we're just
        // demonstrating the pattern. In a production system, we would need to use
        // interior mutability or a different design to allow updating these components.
    }

    /// Discovers nodes on the network
    pub fn discover_nodes(&mut self) -> NetworkResult<Vec<NodeInfo>> {
        let mut discovery = self.discovery.lock().unwrap();
        let found_nodes = discovery.find_nodes()?;
        
        // Add nodes to connection manager
        for node in &found_nodes {
            self.connection_manager.add_node(node.clone());
        }
        
        Ok(found_nodes)
    }

    /// Connects to a node by ID
    pub fn connect_to_node(&self, node_id: &NodeId) -> NetworkResult<()> {
        self.connection_manager.connect_to_node(node_id)
    }

    /// Queries a node for data
    pub fn query_node(&self, node_id: &NodeId, query: Query) -> NetworkResult<QueryResult> {
        // Get connection
        let connection = self.connection_manager.get_connection(node_id)?;
        
        // Create trust proof
        let trust_proof = crate::datafold_node::network::message::TrustProof {
            public_key: "".to_string(), // TODO: Use actual public key
            signature: "".to_string(),  // TODO: Sign the query
            trust_distance: 0,          // TODO: Use actual trust distance
        };
        
        // Send query
        self.query_service.query_node(connection, query, trust_proof)
    }

    /// Lists available schemas on a node
    pub fn list_available_schemas(&self, node_id: &NodeId) -> NetworkResult<Vec<SchemaInfo>> {
        // Get connection
        let connection = self.connection_manager.get_connection(node_id)?;
        
        // List schemas
        self.schema_service.list_remote_schemas(connection)
    }

    /// Gets the list of connected nodes
    pub fn connected_nodes(&self) -> HashSet<NodeId> {
        self.connection_manager.connected_nodes()
    }

    /// Gets the list of known nodes
    pub fn known_nodes(&self) -> HashMap<NodeId, NodeInfo> {
        self.connection_manager.known_nodes()
    }
}

impl Drop for NetworkCore {
    fn drop(&mut self) {
        if let Err(e) = self.stop() {
            eprintln!("Error stopping network core: {}", e);
        }
    }
}
