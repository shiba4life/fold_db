use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{AppSdkError, AppSdkResult};
use crate::types::{
    NodeConnection, AuthCredentials, SchemaCache, NodeInfo, AppRequest, RemoteNodeInfo
};
use crate::query_builder::QueryBuilder;
use crate::mutation_builder::MutationBuilder;
use crate::network_manager::NetworkManager;
use crate::network_utils::NetworkUtils;
use crate::schema_discovery::SchemaDiscovery;
use crate::schema::Schema;
use crate::schema_builder::SchemaBuilder;

/// Main client for interacting with the DataFold network
#[derive(Debug, Clone)]
pub struct DataFoldClient {
    /// Connection to the local node
    connection: NodeConnection,
    
    /// Authentication credentials
    auth: AuthCredentials,
    
    /// Schema cache
    schema_cache: Arc<Mutex<SchemaCache>>,
    
    /// Network manager for remote operations
    network_manager: Arc<NetworkManager>,
    
    /// Schema discovery
    schema_discovery: Arc<SchemaDiscovery>,
}

impl DataFoldClient {
    /// Create a new client
    pub fn new(app_id: &str, private_key: &str, public_key: &str) -> Self {
        // Create a default node connection using a Unix socket
        let socket_path = "/var/run/datafold/node.sock".to_string();
        let connection = NodeConnection::UnixSocket(socket_path);
        
        // Create authentication credentials
        let auth = AuthCredentials {
            app_id: app_id.to_string(),
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
        };
        
        // Create schema cache
        let schema_cache = Arc::new(Mutex::new(SchemaCache::new()));
        
        // Create network manager
        let network_manager = Arc::new(NetworkManager::new(
            connection.clone(),
            auth.clone(),
            schema_cache.clone(),
        ));
        
        // Create schema discovery
        let schema_discovery = Arc::new(SchemaDiscovery::new(
            connection.clone(),
            auth.clone(),
            schema_cache.clone(),
        ));
        
        Self {
            connection,
            auth,
            schema_cache,
            network_manager,
            schema_discovery,
        }
    }

    /// Create a new client with a custom node connection
    pub fn with_connection(
        app_id: &str,
        private_key: &str,
        public_key: &str,
        connection: NodeConnection,
    ) -> Self {
        // Create authentication credentials
        let auth = AuthCredentials {
            app_id: app_id.to_string(),
            private_key: private_key.to_string(),
            public_key: public_key.to_string(),
        };
        
        // Create schema cache
        let schema_cache = Arc::new(Mutex::new(SchemaCache::new()));
        
        // Create network manager
        let network_manager = Arc::new(NetworkManager::new(
            connection.clone(),
            auth.clone(),
            schema_cache.clone(),
        ));
        
        // Create schema discovery
        let schema_discovery = Arc::new(SchemaDiscovery::new(
            connection.clone(),
            auth.clone(),
            schema_cache.clone(),
        ));
        
        Self {
            connection,
            auth,
            schema_cache,
            network_manager,
            schema_discovery,
        }
    }

    /// Get a query builder for a specific schema
    pub fn query(&self, schema_name: &str) -> QueryBuilder {
        QueryBuilder::new(
            schema_name,
            self.connection.clone(),
            self.auth.clone(),
            None,
        )
    }

    /// Get a query builder for a specific schema on a remote node
    pub fn query_on_node(&self, schema_name: &str, node_id: &str) -> QueryBuilder {
        QueryBuilder::new(
            schema_name,
            self.connection.clone(),
            self.auth.clone(),
            Some(node_id.to_string()),
        )
    }

    /// Get a mutation builder for a specific schema
    pub fn mutate(&self, schema_name: &str) -> MutationBuilder {
        MutationBuilder::new(
            schema_name,
            self.connection.clone(),
            self.auth.clone(),
            None,
        )
    }

    /// Get a mutation builder for a specific schema on a remote node
    pub fn mutate_on_node(&self, schema_name: &str, node_id: &str) -> MutationBuilder {
        MutationBuilder::new(
            schema_name,
            self.connection.clone(),
            self.auth.clone(),
            Some(node_id.to_string()),
        )
    }

    /// Discover available schemas on the local node
    pub async fn discover_local_schemas(&self) -> AppSdkResult<Vec<String>> {
        self.schema_discovery.get_local_schemas().await
    }

    /// Discover available nodes in the network
    pub async fn discover_nodes(&self) -> AppSdkResult<Vec<NodeInfo>> {
        self.network_manager.discover_nodes().await
    }

    /// Discover available schemas on a remote node
    pub async fn discover_remote_schemas(&self, node_id: &str) -> AppSdkResult<Vec<String>> {
        self.schema_discovery.get_remote_schemas(node_id).await
    }

    /// Get detailed information about a schema
    pub async fn get_schema_details(
        &self,
        schema_name: &str,
        node_id: Option<&str>,
    ) -> AppSdkResult<Value> {
        self.schema_discovery.get_schema_details(schema_name, node_id).await
    }

    /// Create a new schema on the local node
    /// 
    /// This method creates a new schema with the specified configuration.
    /// It uses the SchemaBuilder to construct the schema with the provided
    /// fields, permissions, and payment configurations.
    /// 
    /// # Arguments
    /// 
    /// * `schema` - The schema to create
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or failure
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use datafold_sdk::DataFoldClient;
    /// use datafold_sdk::schema_builder::SchemaBuilder;
    /// use datafold_sdk::schema::FieldType;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DataFoldClient::new("app_id", "private_key", "public_key");
    /// 
    /// let schema = SchemaBuilder::new("user_profile")
    ///     .add_field("username", |field| {
    ///         field.field_type(FieldType::Single)
    ///             .required(true)
    ///             .description("User's unique username")
    ///     })
    ///     .add_field("email", |field| {
    ///         field.field_type(FieldType::Single)
    ///             .required(true)
    ///             .description("User's email address")
    ///     })
    ///     .build()?;
    /// 
    /// client.create_schema(schema).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_schema(&self, schema: Schema) -> AppSdkResult<()> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "create_schema",
            serde_json::json!({
                "schema": schema,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        let _ = response; // Acknowledge the response to avoid unused variable warning
        
        // Clear the schema cache
        let mut cache = self.schema_cache.lock().await;
        cache.clear();
        
        Ok(())
    }

    /// Update an existing schema on the local node
    /// 
    /// This method updates an existing schema with the specified configuration.
    /// It allows modifying the schema's fields, permissions, and payment configurations.
    /// 
    /// # Arguments
    /// 
    /// * `schema` - The updated schema
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or failure
    pub async fn update_schema(&self, schema: Schema) -> AppSdkResult<()> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "update_schema",
            serde_json::json!({
                "schema": schema,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        let _ = response; // Acknowledge the response to avoid unused variable warning
        
        // Clear the schema cache
        let mut cache = self.schema_cache.lock().await;
        cache.clear();
        
        Ok(())
    }

    /// Delete a schema from the local node
    /// 
    /// This method deletes a schema with the specified name.
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema to delete
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or failure
    pub async fn delete_schema(&self, schema_name: &str) -> AppSdkResult<()> {
        // Create the request
        let request = AppRequest::new(
            &self.auth.app_id,
            None,
            "delete_schema",
            serde_json::json!({
                "schema_name": schema_name,
            }),
            &self.auth.private_key,
        );

        // Send the request
        let response = self.send_request(request).await?;
        let _ = response; // Acknowledge the response to avoid unused variable warning
        
        // Clear the schema cache
        let mut cache = self.schema_cache.lock().await;
        cache.clear();
        
        Ok(())
    }

    /// Get a schema builder for creating a new schema
    /// 
    /// This method returns a SchemaBuilder for constructing a new schema
    /// with fields, permissions, and payment configurations.
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema to create
    /// 
    /// # Returns
    /// 
    /// A SchemaBuilder instance
    /// 
    /// # Examples
    /// 
    /// ```no_run
    /// use datafold_sdk::DataFoldClient;
    /// use datafold_sdk::schema::FieldType;
    /// 
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = DataFoldClient::new("app_id", "private_key", "public_key");
    /// 
    /// let schema = client.schema_builder("user_profile")
    ///     .add_field("username", |field| {
    ///         field.field_type(FieldType::Single)
    ///             .required(true)
    ///             .description("User's unique username")
    ///     })
    ///     .add_field("email", |field| {
    ///         field.field_type(FieldType::Single)
    ///             .required(true)
    ///             .description("User's email address")
    ///     })
    ///     .build()?;
    /// 
    /// client.create_schema(schema).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn schema_builder(&self, schema_name: &str) -> SchemaBuilder {
        SchemaBuilder::new(schema_name)
    }

    /// Check if a node is available
    pub async fn is_node_available(&self, node_id: &str) -> AppSdkResult<bool> {
        self.network_manager.is_node_available(node_id).await
    }

    /// Get information about a node
    pub async fn get_node_info(&self, node_id: &str) -> AppSdkResult<NodeInfo> {
        self.network_manager.get_node_info(node_id).await
    }

    /// Get information about all known nodes
    pub async fn get_all_nodes(&self) -> AppSdkResult<Vec<RemoteNodeInfo>> {
        self.network_manager.get_all_nodes().await
    }

    /// Send a raw request to the node
    pub async fn send_request(&self, request: AppRequest) -> AppSdkResult<Value> {
        NetworkUtils::send_request(&self.connection, request).await
    }

    /// Clear the schema cache
    pub async fn clear_cache(&self) -> AppSdkResult<()> {
        let mut cache = self.schema_cache.lock().await;
        cache.clear();
        Ok(())
    }

    /// Get the app ID
    pub fn get_app_id(&self) -> &str {
        &self.auth.app_id
    }

    /// Get the public key
    pub fn get_public_key(&self) -> &str {
        &self.auth.public_key
    }
}

/// Helper function to create a client from environment variables
pub fn create_client_from_env() -> AppSdkResult<DataFoldClient> {
    // Get environment variables
    let app_id = std::env::var("DATAFOLD_APP_ID")
        .map_err(|_| AppSdkError::Client("DATAFOLD_APP_ID environment variable not set".to_string()))?;
    
    let private_key = std::env::var("DATAFOLD_PRIVATE_KEY")
        .map_err(|_| AppSdkError::Client("DATAFOLD_PRIVATE_KEY environment variable not set".to_string()))?;
    
    let public_key = std::env::var("DATAFOLD_PUBLIC_KEY")
        .map_err(|_| AppSdkError::Client("DATAFOLD_PUBLIC_KEY environment variable not set".to_string()))?;
    
    // Create the client
    Ok(DataFoldClient::new(&app_id, &private_key, &public_key))
}
