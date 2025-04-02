use std::collections::HashSet;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::AppSdkResult;
use crate::network_utils::NetworkUtils;
use crate::types::{
    NodeConnection, AuthCredentials, SchemaCache, SchemaInfo, AppRequest
};

/// Discovery for schemas
#[derive(Debug, Clone)]
pub struct SchemaDiscovery {
    /// Connection to the local node
    connection: NodeConnection,
    
    /// Authentication credentials
    auth: AuthCredentials,
    
    /// Schema cache
    schema_cache: Arc<Mutex<SchemaCache>>,
}

impl SchemaDiscovery {
    /// Create a new schema discovery
    pub fn new(
        connection: NodeConnection,
        auth: AuthCredentials,
        schema_cache: Arc<Mutex<SchemaCache>>,
    ) -> Self {
        Self {
            connection,
            auth,
            schema_cache,
        }
    }

    /// Get all available schemas on the local node
    pub async fn get_local_schemas(&self) -> AppSdkResult<Vec<String>> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "list_schemas",
            serde_json::json!({}),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let schemas: Vec<String> = serde_json::from_value(response)?;
        
        Ok(schemas)
    }

    /// Get schemas available on a remote node
    pub async fn get_remote_schemas(&self, node_id: &str) -> AppSdkResult<Vec<String>> {
        // Check if the schemas are in the cache
        let schema_cache = self.schema_cache.lock().await;
        if let Some(schemas) = schema_cache.get_node_schemas(node_id) {
            return Ok(schemas.iter().cloned().collect());
        }
        drop(schema_cache);
        
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            Some(node_id.to_string()),
            "list_schemas",
            serde_json::json!({}),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response
        let schemas: Vec<String> = serde_json::from_value(response)?;
        
        // Update the cache
        let mut schema_cache = self.schema_cache.lock().await;
        let schemas_set: HashSet<String> = schemas.iter().cloned().collect();
        schema_cache.add_node_schemas(node_id, schemas_set);
        
        Ok(schemas)
    }

    /// Get detailed information about a schema
    pub async fn get_schema_details(
        &self,
        schema_name: &str,
        node_id: Option<&str>,
    ) -> AppSdkResult<Value> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            node_id.map(|s| s.to_string()),
            "get_schema",
            serde_json::json!({
                "schema_name": schema_name,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        
        // Parse the response and update the cache
        let schema_info: SchemaInfo = serde_json::from_value(response.clone())?;
        let mut schema_cache = self.schema_cache.lock().await;
        schema_cache.add_schema(schema_info);
        
        Ok(response)
    }

    /// Send a request to the node
    async fn send_request(&self, request: AppRequest) -> AppSdkResult<Value> {
        NetworkUtils::send_request(&self.connection, request).await
    }
}
