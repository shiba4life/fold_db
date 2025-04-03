use std::collections::HashMap;
use serde_json::Value;

use crate::error::{AppSdkError, AppSdkResult};
use crate::network_utils::NetworkUtils;
use crate::types::{
    NodeConnection, AuthCredentials, QueryFilter, QueryResult, AppRequest
};

/// Builder for constructing and executing queries
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// Schema name
    schema_name: String,
    
    /// Fields to retrieve
    fields: Vec<String>,
    
    /// Filter condition
    filter: Option<QueryFilter>,
    
    /// Target node (None for local node)
    target_node: Option<String>,
    
    /// Connection to the node
    connection: NodeConnection,
    
    /// Authentication credentials
    auth: AuthCredentials,
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new(
        schema_name: &str,
        connection: NodeConnection,
        auth: AuthCredentials,
        target_node: Option<String>,
    ) -> Self {
        Self {
            schema_name: schema_name.to_string(),
            fields: Vec::new(),
            filter: None,
            target_node,
            connection,
            auth,
        }
    }

    /// Select fields to retrieve
    pub fn select(mut self, fields: &[&str]) -> Self {
        self.fields = fields.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Add a filter condition
    pub fn filter(mut self, filter: QueryFilter) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Target a specific remote node
    pub fn on_node(mut self, node_id: &str) -> Self {
        self.target_node = Some(node_id.to_string());
        self
    }

    /// Execute the query
    pub async fn execute(&self) -> AppSdkResult<QueryResult> {
        // Validate the query
        if self.fields.is_empty() {
            return Err(AppSdkError::Client("No fields selected for query".to_string()));
        }

        // Create the operation parameters
        let mut params = HashMap::new();
        params.insert("schema".to_string(), Value::String(self.schema_name.clone()));
        params.insert("fields".to_string(), serde_json::to_value(&self.fields)?);
        
        if let Some(filter) = &self.filter {
            params.insert("filter".to_string(), serde_json::to_value(filter)?);
        }

        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            self.target_node.clone(),
            "query",
            serde_json::to_value(params)?,
            &self.auth.private_key,
        );

        // Send the request
        self.send_request(request).await
    }

    /// Send a request to the node
    async fn send_request(&self, request: AppRequest) -> AppSdkResult<QueryResult> {
        let response = NetworkUtils::send_request(&self.connection, request).await?;
        
        // Check if the response is an array directly (common case)
        if let Some(array) = response.as_array() {
            return Ok(QueryResult {
                results: array.clone(),
                errors: Vec::new(),
            });
        }
        
        // Try to deserialize the response as a standard QueryResult
        match serde_json::from_value::<QueryResult>(response.clone()) {
            Ok(result) => Ok(result),
            Err(e) => {
                // Try to extract results from the response directly
                if let Some(results) = response.get("results") {
                    if let Some(results_array) = results.as_array() {
                        return Ok(QueryResult {
                            results: results_array.clone(),
                            errors: Vec::new(),
                        });
                    } else if let Some(results_obj) = results.as_object() {
                        let results_vec: Vec<Value> = results_obj
                            .iter()
                            .map(|(_, v)| v.clone())
                            .collect();
                        
                        return Ok(QueryResult {
                            results: results_vec,
                            errors: Vec::new(),
                        });
                    }
                }
                
                // If all else fails, return the original error
                Err(AppSdkError::Serialization(format!("Failed to deserialize QueryResult: {}", e)))
            }
        }
    }
}

/// Helper function to create a query for a specific schema
pub fn query(schema_name: &str) -> QueryBuilder {
    // Create a default node connection using a Unix socket
    let socket_path = "/var/run/datafold/node.sock".to_string();
    let connection = NodeConnection::UnixSocket(socket_path);
    
    // Create dummy authentication credentials
    let auth = AuthCredentials {
        app_id: "dummy-app".to_string(),
        private_key: "dummy-private-key".to_string(),
        public_key: "dummy-public-key".to_string(),
    };
    
    QueryBuilder::new(schema_name, connection, auth, None)
}
