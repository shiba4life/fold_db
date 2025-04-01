use std::collections::HashMap;
use serde_json::Value;

use crate::error::{AppSdkError, AppSdkResult};
use crate::types::{
    NodeConnection, AuthCredentials, MutationResult, AppRequest
};

/// Builder for constructing and executing mutations
#[derive(Debug, Clone)]
pub struct MutationBuilder {
    /// Schema name
    schema_name: String,
    
    /// Mutation type
    mutation_type: MutationType,
    
    /// Field values
    data: HashMap<String, Value>,
    
    /// Target node (None for local node)
    target_node: Option<String>,
    
    /// Connection to the node
    #[allow(dead_code)]
    connection: NodeConnection,
    
    /// Authentication credentials
    auth: AuthCredentials,
}

impl MutationBuilder {
    /// Create a new mutation builder
    pub fn new(
        schema_name: &str,
        connection: NodeConnection,
        auth: AuthCredentials,
        target_node: Option<String>,
    ) -> Self {
        Self {
            schema_name: schema_name.to_string(),
            mutation_type: MutationType::Create,
            data: HashMap::new(),
            target_node,
            connection,
            auth,
        }
    }

    /// Set the mutation type
    pub fn operation(mut self, operation: MutationType) -> Self {
        self.mutation_type = operation;
        self
    }

    /// Set a field value
    pub fn set(mut self, field: &str, value: Value) -> Self {
        self.data.insert(field.to_string(), value);
        self
    }

    /// Set multiple field values
    pub fn set_many(mut self, values: HashMap<String, Value>) -> Self {
        self.data.extend(values);
        self
    }

    /// Target a specific remote node
    pub fn on_node(mut self, node_id: &str) -> Self {
        self.target_node = Some(node_id.to_string());
        self
    }

    /// Execute the mutation
    pub async fn execute(&self) -> AppSdkResult<MutationResult> {
        // Validate the mutation
        if self.data.is_empty() && self.mutation_type != MutationType::Delete {
            return Err(AppSdkError::Client("No data provided for mutation".to_string()));
        }

        // Create the operation parameters
        let mut params = HashMap::new();
        params.insert("schema".to_string(), Value::String(self.schema_name.clone()));
        params.insert("data".to_string(), serde_json::to_value(&self.data)?);
        params.insert("mutation_type".to_string(), serde_json::to_value(&self.mutation_type)?);

        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            self.target_node.clone(),
            "mutation",
            serde_json::to_value(params)?,
            &self.auth.private_key,
        );

        // Send the request
        self.send_request(request).await
    }

    /// Send a request to the node
    async fn send_request(&self, request: AppRequest) -> AppSdkResult<MutationResult> {
        // In a real implementation, this would send the request to the node
        // For now, we'll just log that we're sending a request and return a dummy response
        println!("Sending mutation request to node: {:?}", request);
        
        // Create a dummy response
        Ok(MutationResult {
            success: true,
            id: Some("123".to_string()),
            error: None,
        })
    }
}

/// Mutation types
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MutationType {
    #[serde(rename = "create")]
    Create,
    
    #[serde(rename = "update")]
    Update,
    
    #[serde(rename = "delete")]
    Delete,
    
    #[serde(rename = "add_to_collection")]
    AddToCollection(String),
    
    #[serde(rename = "update_in_collection")]
    UpdateInCollection(String),
    
    #[serde(rename = "delete_from_collection")]
    DeleteFromCollection(String),
}

/// Helper function to create a mutation for a specific schema
pub fn mutate(schema_name: &str) -> MutationBuilder {
    // Create a default node connection using a Unix socket
    let socket_path = "/var/run/datafold/node.sock".to_string();
    let connection = NodeConnection::UnixSocket(socket_path);
    
    // Create dummy authentication credentials
    let auth = AuthCredentials {
        app_id: "dummy-app".to_string(),
        private_key: "dummy-private-key".to_string(),
        public_key: "dummy-public-key".to_string(),
    };
    
    MutationBuilder::new(schema_name, connection, auth, None)
}
