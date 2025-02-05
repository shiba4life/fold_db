use std::collections::HashMap;
use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum SchemaError {
    NotFound(String),
    InvalidField(String),
    InvalidPermission(String),
    InvalidTransform(String),
    InvalidData(String),
    InvalidDSL(String),
    MappingError(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SchemaError::NotFound(msg) => write!(f, "Schema not found: {}", msg),
            SchemaError::InvalidField(msg) => write!(f, "Invalid field: {}", msg),
            SchemaError::InvalidPermission(msg) => write!(f, "Invalid permission: {}", msg),
            SchemaError::InvalidTransform(msg) => write!(f, "Invalid transform: {}", msg),
            SchemaError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            SchemaError::InvalidDSL(msg) => write!(f, "Invalid DSL: {}", msg),
            SchemaError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
        }
    }
}

impl std::error::Error for SchemaError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    Read,
    Write,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyLevel {
    Public,
    Private,
    Explicit,
    ExplicitOnce,
    ExplicitMany,
}

impl Count {
    pub const Limited: fn(u32) -> Count = |limit| Count {
        read: limit,
        write: limit,
    };
    
    pub const Unlimited: fn() -> Count = || Count {
        read: u32::MAX,
        write: u32::MAX,
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Count {
    pub read: u32,
    pub write: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitCounts {
    pub counts: HashMap<String, Count>, // pub_key -> counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    pub read_policy: PolicyLevel,
    pub write_policy: PolicyLevel,
    pub explicit_counts: Option<ExplicitCounts>,
}

impl PermissionsPolicy {
    pub fn new(read_policy: PolicyLevel, write_policy: PolicyLevel) -> Self {
        Self {
            read_policy,
            write_policy,
            explicit_counts: None,
        }
    }

    pub fn set_explicit_permissions(&mut self, pub_key: String, r: Count, w: Count) {
        let mut counts = self.explicit_counts.get_or_insert(ExplicitCounts {
            counts: HashMap::new(),
        });
        counts.counts.insert(pub_key, Count {
            read: r.read,
            write: w.write,
        });
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    Single,    // Regular field with a single value
    Collection // Field containing multiple values
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub permission_setting: String, // X:0, W1, etc
    pub ref_atom_uuid: String,
    pub field_type: FieldType,
    pub explicit_access: HashMap<String, AccessCounts>, // pub_key -> access counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessCounts {
    pub w: u32,
    pub r: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub fields: HashMap<String, SchemaField>,
    pub transforms: Vec<String>, // Transform names/identifiers
}

impl Schema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            transforms: Vec::new(),
        }
    }

    pub fn add_field(&mut self, field_name: String, field: SchemaField) {
        self.fields.insert(field_name, field);
    }

    pub fn add_transform(&mut self, transform: String) {
        self.transforms.push(transform);
    }
}

impl SchemaField {
    pub fn new(permission_setting: String, ref_atom_uuid: String, field_type: FieldType) -> Self {
        Self {
            permission_setting,
            ref_atom_uuid,
            field_type,
            explicit_access: HashMap::new(),
        }
    }

    pub fn add_explicit_access(&mut self, pub_key: String, w: u32, r: u32) {
        self.explicit_access.insert(pub_key, AccessCounts { w, r });
    }
}
