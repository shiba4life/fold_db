use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::AppSdkResult;
use crate::network_utils::NetworkUtils;
use crate::types::{
    NodeConnection, AuthCredentials, SchemaCache, NodeInfo, AppRequest, RemoteNodeInfo
};

/// Manager for network operations
#[derive(Debug, Clone)]
pub struct NetworkManager {
    /// Connection to the local node
    connection: NodeConnection,
    
    /// Authentication credentials
    auth: AuthCredentials,
    
    /// Schema cache
    schema_cache: Arc<Mutex<SchemaCache>>,
    
    /// Known nodes
    known_nodes: Arc<Mutex<HashMap<String, NodeInfo>>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(
        connection: NodeConnection,
        auth: AuthCredentials,
        schema_cache: Arc<Mutex<SchemaCache>>,
    ) -> Self {
        Self {
            connection,
            auth,
            schema_cache,
            known_nodes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Discover available nodes in the network
    pub async fn discover_nodes(&self) -> AppSdkResult<Vec<NodeInfo>> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "discover_nodes",
            serde_json::json!({}),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let nodes: Vec<NodeInfo> = serde_json::from_value(response)?;
        
        // Update the known nodes
        let mut known_nodes = self.known_nodes.lock().await;
        for node in &nodes {
            known_nodes.insert(node.id.clone(), node.clone());
        }
        
        Ok(nodes)
    }

    /// Check if a node is available
    pub async fn is_node_available(&self, node_id: &str) -> AppSdkResult<bool> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "check_node_availability",
            serde_json::json!({
                "node_id": node_id,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let available: bool = serde_json::from_value(response)?;
        
        Ok(available)
    }

    /// Get information about a node
    pub async fn get_node_info(&self, node_id: &str) -> AppSdkResult<NodeInfo> {
        // Check if the node is in the known nodes
        let known_nodes = self.known_nodes.lock().await;
        if let Some(node) = known_nodes.get(node_id) {
            return Ok(node.clone());
        }
        drop(known_nodes);
        
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "get_node_info",
            serde_json::json!({
                "node_id": node_id,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let node: NodeInfo = serde_json::from_value(response)?;
        
        // Update the known nodes
        let mut known_nodes = self.known_nodes.lock().await;
        known_nodes.insert(node.id.clone(), node.clone());
        
        Ok(node)
    }

    /// Get information about all known nodes
    pub async fn get_all_nodes(&self) -> AppSdkResult<Vec<RemoteNodeInfo>> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "get_all_nodes",
            serde_json::json!({}),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let nodes: Vec<RemoteNodeInfo> = serde_json::from_value(response)?;
        
        // Update the known nodes
        let mut known_nodes = self.known_nodes.lock().await;
        for node in &nodes {
            known_nodes.insert(node.id.clone(), NodeInfo {
                id: node.id.clone(),
                trust_distance: node.trust_distance,
            });
            
            // Update the schema cache
            let mut schema_cache = self.schema_cache.lock().await;
            let schemas: HashSet<String> = node.available_schemas.iter().cloned().collect();
            schema_cache.add_node_schemas(&node.id, schemas);
        }
        
        Ok(nodes)
    }

    /// Send a request to the node
    async fn send_request(&self, request: AppRequest) -> AppSdkResult<Value> {
        NetworkUtils::send_request(&self.connection, request).await
    }
}
