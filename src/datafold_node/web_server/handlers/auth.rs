use std::sync::Arc;
use warp::{Rejection, Reply};
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use crate::datafold_node::node::DataFoldNode;
use crate::datafold_node::web_server::types::{ApiSuccessResponse, ApiErrorResponse};
use crate::datafold_node::web_server::auth::{WebAuthManager, PublicKey};
use crate::error::FoldDbError;

/// Request to register a new public key
#[derive(Debug, Deserialize)]
pub struct RegisterKeyRequest {
    /// The public key to register
    pub public_key: String,
    /// The trust level to assign to the key
    pub trust_level: u32,
    /// Admin token for authorization (optional)
    pub admin_token: Option<String>,
}

/// Response for key registration
#[derive(Debug, Serialize)]
pub struct RegisterKeyResponse {
    /// The registered public key
    pub public_key: String,
    /// The assigned trust level
    pub trust_level: u32,
}

/// Handle key registration
pub async fn handle_register_key(
    body: RegisterKeyRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
    auth_manager: Arc<Mutex<WebAuthManager>>,
) -> Result<impl Reply, Rejection> {
    // Validate admin token if provided
    if let Some(token) = &body.admin_token {
        // In a real implementation, we would validate the admin token
        // For now, we'll just check if it's a non-empty string
        if token.is_empty() {
            return Err(warp::reject::custom(ApiErrorResponse::new("Invalid admin token")));
        }
    } else {
        // If no admin token is provided, only allow registration with low trust level
        if body.trust_level < 5 {
            return Err(warp::reject::custom(ApiErrorResponse::new(
                "Admin token required for trust level < 5",
            )));
        }
    }

    // Register the key
    let mut auth_manager = auth_manager.lock().await;
    match auth_manager.register_key(PublicKey(body.public_key.clone()), body.trust_level) {
        Ok(_) => {
            let response = RegisterKeyResponse {
                public_key: body.public_key,
                trust_level: body.trust_level,
            };
            Ok(warp::reply::json(&ApiSuccessResponse::new(response)))
        }
        Err(e) => Err(warp::reject::custom(ApiErrorResponse::new(format!(
            "Failed to register key: {}",
            e
        )))),
    }
}

/// Handle schema operations with authentication
pub async fn handle_schema_with_auth(
    trust_level: u32,
    schema: crate::schema::types::schema::Schema,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for schema operations
    if trust_level > 3 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for schema operations",
        )));
    }

    // Call the original handler
    super::schema::handle_schema(schema, node).await
}

/// Handle execute operations with authentication
pub async fn handle_execute_with_auth(
    trust_level: u32,
    query: crate::datafold_node::web_server::types::QueryRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for execute operations
    // For execute operations, we'll check the specific operation in the handler
    // based on the operation type and fields being accessed

    // Call the original handler
    super::schema::handle_execute(query, node).await
}

/// Handle delete schema operations with authentication
pub async fn handle_delete_schema_with_auth(
    name: String,
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for delete schema operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for delete schema operations",
        )));
    }

    // Call the original handler
    super::schema::handle_delete_schema(name, node).await
}

/// Handle init network operations with authentication
pub async fn handle_init_network_with_auth(
    trust_level: u32,
    config: crate::datafold_node::web_server::types::NetworkInitRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for init network operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for init network operations",
        )));
    }

    // Call the original handler
    super::network::handle_init_network(config, node).await
}

/// Handle start network operations with authentication
pub async fn handle_start_network_with_auth(
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for start network operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for start network operations",
        )));
    }

    // Call the original handler
    super::network::handle_start_network(node).await
}

/// Handle stop network operations with authentication
pub async fn handle_stop_network_with_auth(
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for stop network operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for stop network operations",
        )));
    }

    // Call the original handler
    super::network::handle_stop_network(node).await
}

/// Handle discover nodes operations with authentication
pub async fn handle_discover_nodes_with_auth(
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for discover nodes operations
    if trust_level > 3 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for discover nodes operations",
        )));
    }

    // Call the original handler
    super::network::handle_discover_nodes(node).await
}

/// Handle connect to node operations with authentication
pub async fn handle_connect_to_node_with_auth(
    trust_level: u32,
    request: crate::datafold_node::web_server::types::ConnectToNodeRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for connect to node operations
    if trust_level > 3 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for connect to node operations",
        )));
    }

    // Call the original handler
    super::network::handle_connect_to_node(request, node).await
}

/// Handle register app operations with authentication
pub async fn handle_register_app_with_auth(
    trust_level: u32,
    manifest: crate::datafold_node::app::AppManifest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for register app operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for register app operations",
        )));
    }

    // Call the original handler
    super::app::handle_register_app(manifest, node).await
}

/// Handle start app operations with authentication
pub async fn handle_start_app_with_auth(
    name: String,
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for start app operations
    if trust_level > 3 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for start app operations",
        )));
    }

    // Call the original handler
    super::app::handle_start_app(name, node).await
}

/// Handle stop app operations with authentication
pub async fn handle_stop_app_with_auth(
    name: String,
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for stop app operations
    if trust_level > 3 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for stop app operations",
        )));
    }

    // Call the original handler
    super::app::handle_stop_app(name, node).await
}

/// Handle unload app operations with authentication
pub async fn handle_unload_app_with_auth(
    name: String,
    trust_level: u32,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for unload app operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for unload app operations",
        )));
    }

    // Call the original handler
    super::app::handle_unload_app(name, node).await
}

/// Handle register API operations with authentication
pub async fn handle_register_api_with_auth(
    trust_level: u32,
    api: crate::datafold_node::web_server::handlers::app::RegisterApiRequest,
    node: Arc<tokio::sync::Mutex<DataFoldNode>>,
) -> Result<impl Reply, Rejection> {
    // Check if trust level is sufficient for register API operations
    if trust_level > 2 {
        return Err(warp::reject::custom(ApiErrorResponse::new(
            "Insufficient trust level for register API operations",
        )));
    }

    // Call the original handler
    super::app::handle_register_api(api, node).await
}
