use serde_json::Value;

use crate::error::{AppSdkError, AppSdkResult};
use crate::types::{AppRequest, QueryResult, MutationResult, NodeInfo, RemoteNodeInfo, SchemaInfo};

/// Mock implementation for network operations in tests
pub struct NetworkMock;

impl NetworkMock {
    /// Handle mock requests for testing
    pub async fn handle_mock_request(request: &AppRequest) -> AppSdkResult<Value> {
        match request.operation.as_str() {
            "custom_operation" => {
                // Return mock response for custom operation
                Ok(serde_json::json!({
                    "success": true,
                    "result": {
                        "message": "Custom operation processed successfully",
                        "timestamp": chrono::Utc::now().to_rfc3339(),
                    }
                }))
            }
            "list_schemas" => {
                // Return mock schemas
                Ok(serde_json::to_value(vec!["user", "post", "comment"])?)
            }
            "get_schema" => {
                // Return mock schema details
                let schema_name = request.params.get("schema_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                // Return error for non-existent schema
                if schema_name == "non_existent_schema" {
                    return Err(AppSdkError::Client(format!("Schema not found: {}", schema_name)));
                }
                
                let schema_info = SchemaInfo {
                    name: schema_name.to_string(),
                    fields: vec![
                        serde_json::from_value(serde_json::json!({
                            "name": "id",
                            "field_type": "string",
                            "description": "Unique identifier",
                            "required": true
                        }))?,
                        serde_json::from_value(serde_json::json!({
                            "name": "name",
                            "field_type": "string",
                            "description": "Name",
                            "required": true
                        }))?,
                        serde_json::from_value(serde_json::json!({
                            "name": "email",
                            "field_type": "string",
                            "description": "Email address",
                            "required": true
                        }))?,
                    ],
                    description: Some(format!("{} schema", schema_name)),
                };
                
                Ok(serde_json::to_value(schema_info)?)
            }
            "query" => {
                // Return mock query results
                let query_result = QueryResult {
                    results: vec![
                        serde_json::json!({
                            "id": "1",
                            "name": "Test User",
                            "email": "test@example.com"
                        }),
                        serde_json::json!({
                            "id": "2",
                            "name": "Another User",
                            "email": "another@example.com"
                        }),
                    ],
                    errors: vec![],
                };
                
                Ok(serde_json::to_value(query_result)?)
            }
            "mutation" => {
                // Return mock mutation result
                let mutation_result = MutationResult {
                    success: true,
                    id: Some("123".to_string()),
                    error: None,
                };
                
                Ok(serde_json::to_value(mutation_result)?)
            }
            "discover_nodes" => {
                // Return mock nodes
                let nodes = vec![
                    NodeInfo {
                        id: "node1".to_string(),
                        trust_distance: 1,
                    },
                    NodeInfo {
                        id: "node2".to_string(),
                        trust_distance: 2,
                    },
                ];
                
                Ok(serde_json::to_value(nodes)?)
            }
            "check_node_availability" => {
                // Return mock availability
                Ok(serde_json::to_value(true)?)
            }
            "get_node_info" => {
                // Return mock node info
                let node_id = request.params.get("node_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                let node_info = NodeInfo {
                    id: node_id.to_string(),
                    trust_distance: 1,
                };
                
                Ok(serde_json::to_value(node_info)?)
            }
            "get_all_nodes" => {
                // Return mock remote nodes
                let nodes = vec![
                    RemoteNodeInfo {
                        id: "node1".to_string(),
                        trust_distance: 1,
                        available_schemas: vec!["user".to_string(), "post".to_string()],
                    },
                    RemoteNodeInfo {
                        id: "node2".to_string(),
                        trust_distance: 2,
                        available_schemas: vec!["user".to_string(), "comment".to_string()],
                    },
                ];
                
                Ok(serde_json::to_value(nodes)?)
            }
            _ => {
                // Unknown operation
                Err(AppSdkError::Client(format!("Unknown operation: {}", request.operation)))
            }
        }
    }
    
    /// Check if a path is a mock path
    pub fn is_mock_path(path: &str) -> bool {
        path == "mock" || path == "/var/run/datafold/node.sock"
    }
}
