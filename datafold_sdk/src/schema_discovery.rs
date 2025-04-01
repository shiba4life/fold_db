use std::collections::HashSet;
use std::sync::Arc;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::error::{AppSdkError, AppSdkResult};
use crate::types::{
    NodeConnection, AuthCredentials, SchemaCache, SchemaInfo, AppRequest
};

/// Discovery for schemas
#[derive(Debug, Clone)]
pub struct SchemaDiscovery {
    /// Connection to the local node
    #[allow(dead_code)]
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
        // In a real implementation, this would send the request to the node
        // For now, we'll just log that we're sending a request and return a dummy response
        println!("Sending schema discovery request to node: {:?}", request);
        
        // Create a dummy response based on the operation
        match request.operation.as_str() {
            "list_schemas" => {
                Ok(serde_json::json!(["user", "post", "comment"]))
            },
            "get_schema" => {
                let params: Value = serde_json::from_str(&request.params.to_string())?;
                let schema_name = params["schema_name"].as_str().unwrap_or("unknown");
                
                match schema_name {
                    "user" => {
                        Ok(serde_json::json!({
                            "name": "user",
                            "fields": [
                                {
                                    "name": "id",
                                    "field_type": "string",
                                    "description": "Unique identifier",
                                    "required": true
                                },
                                {
                                    "name": "username",
                                    "field_type": "string",
                                    "description": "User's username",
                                    "required": true
                                },
                                {
                                    "name": "email",
                                    "field_type": "string",
                                    "description": "User's email address",
                                    "required": true
                                },
                                {
                                    "name": "full_name",
                                    "field_type": "string",
                                    "description": "User's full name",
                                    "required": false
                                },
                                {
                                    "name": "bio",
                                    "field_type": "string",
                                    "description": "User's biography",
                                    "required": false
                                },
                                {
                                    "name": "created_at",
                                    "field_type": "datetime",
                                    "description": "When the user was created",
                                    "required": true
                                }
                            ],
                            "description": "User profile information"
                        }))
                    },
                    "post" => {
                        Ok(serde_json::json!({
                            "name": "post",
                            "fields": [
                                {
                                    "name": "id",
                                    "field_type": "string",
                                    "description": "Unique identifier",
                                    "required": true
                                },
                                {
                                    "name": "title",
                                    "field_type": "string",
                                    "description": "Post title",
                                    "required": true
                                },
                                {
                                    "name": "content",
                                    "field_type": "string",
                                    "description": "Post content",
                                    "required": true
                                },
                                {
                                    "name": "author_id",
                                    "field_type": "string",
                                    "description": "ID of the post author",
                                    "required": true
                                },
                                {
                                    "name": "created_at",
                                    "field_type": "datetime",
                                    "description": "When the post was created",
                                    "required": true
                                },
                                {
                                    "name": "updated_at",
                                    "field_type": "datetime",
                                    "description": "When the post was last updated",
                                    "required": false
                                }
                            ],
                            "description": "User-created posts"
                        }))
                    },
                    "comment" => {
                        Ok(serde_json::json!({
                            "name": "comment",
                            "fields": [
                                {
                                    "name": "id",
                                    "field_type": "string",
                                    "description": "Unique identifier",
                                    "required": true
                                },
                                {
                                    "name": "content",
                                    "field_type": "string",
                                    "description": "Comment content",
                                    "required": true
                                },
                                {
                                    "name": "author_id",
                                    "field_type": "string",
                                    "description": "ID of the comment author",
                                    "required": true
                                },
                                {
                                    "name": "post_id",
                                    "field_type": "string",
                                    "description": "ID of the post being commented on",
                                    "required": true
                                },
                                {
                                    "name": "created_at",
                                    "field_type": "datetime",
                                    "description": "When the comment was created",
                                    "required": true
                                }
                            ],
                            "description": "Comments on posts"
                        }))
                    },
                    _ => {
                        Err(AppSdkError::Schema(format!("Unknown schema: {}", schema_name)))
                    }
                }
            },
            _ => {
                Err(AppSdkError::Schema(format!("Unknown operation: {}", request.operation)))
            }
        }
    }
}
