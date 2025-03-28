use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// Permissions for an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermissions {
    /// Schemas the app can access
    pub allowed_schemas: HashSet<String>,
    
    /// Fields the app can read/write per schema
    pub field_permissions: HashMap<String, FieldPermissions>,
    
    /// Remote nodes the app can access
    pub allowed_remote_nodes: HashSet<String>,
    
    /// Maximum trust distance for queries
    pub max_trust_distance: u32,
    
    /// Rate limits for operations
    pub rate_limits: OperationRateLimits,
}

impl AppPermissions {
    /// Create new app permissions with default values
    pub fn new() -> Self {
        Self {
            allowed_schemas: HashSet::new(),
            field_permissions: HashMap::new(),
            allowed_remote_nodes: HashSet::new(),
            max_trust_distance: 1,
            rate_limits: OperationRateLimits::default(),
        }
    }

    /// Allow access to a schema
    pub fn allow_schema(mut self, schema_name: &str) -> Self {
        self.allowed_schemas.insert(schema_name.to_string());
        self
    }

    /// Allow access to multiple schemas
    pub fn allow_schemas(mut self, schema_names: &[&str]) -> Self {
        for name in schema_names {
            self.allowed_schemas.insert(name.to_string());
        }
        self
    }

    /// Set field permissions for a schema
    pub fn with_field_permissions(mut self, schema_name: &str, permissions: FieldPermissions) -> Self {
        self.field_permissions.insert(schema_name.to_string(), permissions);
        self
    }

    /// Allow access to a remote node
    pub fn allow_remote_node(mut self, node_id: &str) -> Self {
        self.allowed_remote_nodes.insert(node_id.to_string());
        self
    }

    /// Allow access to multiple remote nodes
    pub fn allow_remote_nodes(mut self, node_ids: &[&str]) -> Self {
        for id in node_ids {
            self.allowed_remote_nodes.insert(id.to_string());
        }
        self
    }

    /// Set the maximum trust distance
    pub fn with_max_trust_distance(mut self, distance: u32) -> Self {
        self.max_trust_distance = distance;
        self
    }

    /// Set rate limits
    pub fn with_rate_limits(mut self, rate_limits: OperationRateLimits) -> Self {
        self.rate_limits = rate_limits;
        self
    }

    /// Check if the app has permission to access a schema
    pub fn can_access_schema(&self, schema_name: &str) -> bool {
        self.allowed_schemas.contains(schema_name)
    }

    /// Check if the app has permission to read a field
    pub fn can_read_field(&self, schema_name: &str, field_name: &str) -> bool {
        if let Some(permissions) = self.field_permissions.get(schema_name) {
            permissions.readable_fields.contains(field_name)
        } else {
            // If no specific field permissions are defined, allow all fields
            self.can_access_schema(schema_name)
        }
    }

    /// Check if the app has permission to write a field
    pub fn can_write_field(&self, schema_name: &str, field_name: &str) -> bool {
        if let Some(permissions) = self.field_permissions.get(schema_name) {
            permissions.writable_fields.contains(field_name)
        } else {
            // If no specific field permissions are defined, allow all fields
            self.can_access_schema(schema_name)
        }
    }

    /// Check if the app has permission to access a remote node
    pub fn can_access_remote_node(&self, node_id: &str) -> bool {
        self.allowed_remote_nodes.contains(node_id)
    }
}

/// Field-level permissions for a schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPermissions {
    /// Fields that can be read
    pub readable_fields: HashSet<String>,
    
    /// Fields that can be written
    pub writable_fields: HashSet<String>,
}

impl FieldPermissions {
    /// Create new field permissions with no fields
    pub fn new() -> Self {
        Self {
            readable_fields: HashSet::new(),
            writable_fields: HashSet::new(),
        }
    }

    /// Allow reading a field
    pub fn allow_read(mut self, field_name: &str) -> Self {
        self.readable_fields.insert(field_name.to_string());
        self
    }

    /// Allow reading multiple fields
    pub fn allow_reads(mut self, field_names: &[&str]) -> Self {
        for name in field_names {
            self.readable_fields.insert(name.to_string());
        }
        self
    }

    /// Allow writing a field
    pub fn allow_write(mut self, field_name: &str) -> Self {
        self.writable_fields.insert(field_name.to_string());
        self
    }

    /// Allow writing multiple fields
    pub fn allow_writes(mut self, field_names: &[&str]) -> Self {
        for name in field_names {
            self.writable_fields.insert(name.to_string());
        }
        self
    }

    /// Allow reading and writing a field
    pub fn allow_read_write(self, field_name: &str) -> Self {
        self.allow_read(field_name).allow_write(field_name)
    }

    /// Allow reading and writing multiple fields
    pub fn allow_read_writes(self, field_names: &[&str]) -> Self {
        self.allow_reads(field_names).allow_writes(field_names)
    }
}

/// Rate limits for operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationRateLimits {
    /// Maximum number of queries per minute
    pub max_queries_per_minute: u32,
    
    /// Maximum number of mutations per minute
    pub max_mutations_per_minute: u32,
    
    /// Maximum number of remote operations per minute
    pub max_remote_ops_per_minute: u32,
    
    /// Maximum number of concurrent operations
    pub max_concurrent_ops: u32,
}

impl Default for OperationRateLimits {
    fn default() -> Self {
        Self {
            max_queries_per_minute: 100,
            max_mutations_per_minute: 20,
            max_remote_ops_per_minute: 50,
            max_concurrent_ops: 10,
        }
    }
}

impl OperationRateLimits {
    /// Create new rate limits with custom values
    pub fn new(
        max_queries_per_minute: u32,
        max_mutations_per_minute: u32,
        max_remote_ops_per_minute: u32,
        max_concurrent_ops: u32,
    ) -> Self {
        Self {
            max_queries_per_minute,
            max_mutations_per_minute,
            max_remote_ops_per_minute,
            max_concurrent_ops,
        }
    }
}
