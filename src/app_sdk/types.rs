use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

/// Connection to a DataFold node
#[derive(Debug, Clone)]
pub enum NodeConnection {
    /// Unix socket connection (preferred for security)
    UnixSocket(String),
    /// Shared memory region
    SharedMemory(SharedMemoryRegion),
    /// Named pipe (for Windows)
    NamedPipe(String),
}

/// Shared memory region for communication
#[derive(Debug, Clone)]
pub struct SharedMemoryRegion {
    /// Name of the shared memory region
    pub name: String,
    /// Size of the shared memory region in bytes
    pub size: usize,
}

/// Authentication credentials for the app
#[derive(Debug, Clone)]
pub struct AuthCredentials {
    /// App identifier
    pub app_id: String,
    /// App's private key for signing requests
    pub private_key: String,
    /// App's public key for verification
    pub public_key: String,
}

/// Cache for schema information
#[derive(Debug, Clone, Default)]
pub struct SchemaCache {
    /// Map of schema names to schema information
    schemas: HashMap<String, SchemaInfo>,
    /// Map of node IDs to available schemas
    node_schemas: HashMap<String, HashSet<String>>,
}

impl SchemaCache {
    /// Create a new schema cache
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            node_schemas: HashMap::new(),
        }
    }

    /// Add a schema to the cache
    pub fn add_schema(&mut self, schema: SchemaInfo) {
        self.schemas.insert(schema.name.clone(), schema);
    }

    /// Get a schema from the cache
    pub fn get_schema(&self, name: &str) -> Option<&SchemaInfo> {
        self.schemas.get(name)
    }

    /// Add a node's available schemas to the cache
    pub fn add_node_schemas(&mut self, node_id: &str, schemas: HashSet<String>) {
        self.node_schemas.insert(node_id.to_string(), schemas);
    }

    /// Get a node's available schemas from the cache
    pub fn get_node_schemas(&self, node_id: &str) -> Option<&HashSet<String>> {
        self.node_schemas.get(node_id)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.schemas.clear();
        self.node_schemas.clear();
    }
}

/// Information about a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    /// Schema name
    pub name: String,
    /// Schema fields
    pub fields: Vec<FieldInfo>,
    /// Schema description
    pub description: Option<String>,
}

/// Information about a field in a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldInfo {
    /// Field name
    pub name: String,
    /// Field type
    pub field_type: String,
    /// Field description
    pub description: Option<String>,
    /// Whether the field is required
    pub required: bool,
}

/// Information about a node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node identifier
    pub id: String,
    /// Trust distance to the node
    pub trust_distance: u32,
}

/// Information about a remote node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteNodeInfo {
    /// Node identifier
    pub id: String,
    /// Trust distance to the node
    pub trust_distance: u32,
    /// Available schemas on the node
    pub available_schemas: Vec<String>,
}

/// Filter for queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    /// Field to filter on
    pub field: String,
    /// Operator to use
    pub operator: FilterOperator,
    /// Value to compare against
    pub value: Value,
}

/// Filter operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    #[serde(rename = "eq")]
    Equals,
    #[serde(rename = "ne")]
    NotEquals,
    #[serde(rename = "gt")]
    GreaterThan,
    #[serde(rename = "lt")]
    LessThan,
    #[serde(rename = "gte")]
    GreaterThanOrEquals,
    #[serde(rename = "lte")]
    LessThanOrEquals,
    #[serde(rename = "contains")]
    Contains,
    #[serde(rename = "starts_with")]
    StartsWith,
    #[serde(rename = "ends_with")]
    EndsWith,
}

impl QueryFilter {
    /// Create a new equals filter
    pub fn eq(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::Equals,
            value,
        }
    }

    /// Create a new not equals filter
    pub fn ne(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::NotEquals,
            value,
        }
    }

    /// Create a new greater than filter
    pub fn gt(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::GreaterThan,
            value,
        }
    }

    /// Create a new less than filter
    pub fn lt(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::LessThan,
            value,
        }
    }

    /// Create a new greater than or equals filter
    pub fn gte(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::GreaterThanOrEquals,
            value,
        }
    }

    /// Create a new less than or equals filter
    pub fn lte(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::LessThanOrEquals,
            value,
        }
    }

    /// Create a new contains filter
    pub fn contains(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::Contains,
            value,
        }
    }

    /// Create a new starts with filter
    pub fn starts_with(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::StartsWith,
            value,
        }
    }

    /// Create a new ends with filter
    pub fn ends_with(field: &str, value: Value) -> Self {
        Self {
            field: field.to_string(),
            operator: FilterOperator::EndsWith,
            value,
        }
    }
}

/// Result of a query operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// Results of the query
    pub results: Vec<Value>,
    /// Errors that occurred during the query
    pub errors: Vec<String>,
}

/// Result of a mutation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    /// Whether the mutation was successful
    pub success: bool,
    /// ID of the created/updated entity
    pub id: Option<String>,
    /// Error message if the mutation failed
    pub error: Option<String>,
}

/// Request from an app to the DataFold node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRequest {
    /// App identifier
    pub app_id: String,
    /// Unix timestamp
    pub timestamp: u64,
    /// Target node (None for local node)
    pub target_node_id: Option<String>,
    /// Operation to perform
    pub operation: String,
    /// Operation parameters
    pub params: Value,
    /// Signature using app's private key
    pub signature: String,
}

impl AppRequest {
    /// Create a new app request
    pub fn new(
        app_id: &str,
        target_node_id: Option<String>,
        operation: &str,
        params: Value,
        private_key: &str,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Create the request without the signature
        let mut request = Self {
            app_id: app_id.to_string(),
            timestamp,
            target_node_id,
            operation: operation.to_string(),
            params,
            signature: String::new(),
        };

        // Sign the request
        request.signature = request.sign(private_key);

        request
    }

    /// Sign the request using the app's private key
    fn sign(&self, private_key: &str) -> String {
        // In a real implementation, this would use the private key to sign the request
        // For now, we'll just return a placeholder
        format!("signed-{}-{}", self.app_id, self.timestamp)
    }
}

/// Communication channel between app and node
#[derive(Debug, Clone)]
pub enum AppChannel {
    /// Unix socket (preferred for security)
    UnixSocket(String),
    /// Shared memory region
    SharedMemory(SharedMemoryRegion),
    /// Named pipe (for Windows)
    NamedPipe(String),
}
