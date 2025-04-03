use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Re-exports of schema-related types from the core library
/// 
/// This module provides access to the Schema class and related types
/// from the core library, allowing SDK users to work with schemas directly.

/// Defines the structure, permissions, and payment requirements for a data collection.
/// 
/// A Schema is the fundamental building block for data organization in the database.
/// It defines:
/// - The collection's name and identity
/// - Field definitions with their types and constraints
/// - Field-level permission policies
/// - Payment requirements for data access
/// - Field mappings for schema transformation
/// 
/// Schemas provide a contract for data storage and access, ensuring:
/// - Consistent data structure
/// - Proper access control
/// - Payment validation
/// - Data transformation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    /// Unique name identifying this schema
    pub name: String,
    /// Collection of fields with their definitions and configurations
    pub fields: HashMap<String, SchemaField>,
    /// Payment configuration for schema-level access control
    pub payment_config: SchemaPaymentConfig,
}

/// Defines the configuration and behavior of a single field within a schema.
/// 
/// SchemaField encapsulates all aspects of a field's behavior:
/// - Access control through permission policies
/// - Payment requirements for field access
/// - Data storage through atom references
/// - Field transformation rules through mappers
/// 
/// Each field can have:
/// - Custom permission policies for read/write access
/// - Specific payment requirements for data access
/// - Links to stored data through atom references
/// - Transformation mappings for schema evolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    /// Permission policy controlling read/write access to this field
    pub permission_policy: PermissionsPolicy,
    
    /// Payment configuration for accessing this field's data
    pub payment_config: FieldPaymentConfig,
    
    /// Reference to the atom containing this field's value
    /// The actual field value is fetched through this reference
    pub ref_atom_uuid: Option<String>,

    /// Type of the field - single value or collection
    pub field_type: FieldType,
    
    /// Mappings for field transformations and schema evolution
    /// Keys are source schema names, values are source field names
    pub field_mappers: HashMap<String, String>,
}

impl SchemaField {
    /// Creates a new SchemaField with the specified permissions and payment config.
    /// 
    /// Initializes a field with:
    /// - Given permission policy for access control
    /// - Specified payment configuration
    /// - No atom reference (no stored value yet)
    /// - Empty field mappings
    /// 
    /// # Arguments
    /// 
    /// * `permission_policy` - Policy controlling field access
    /// * `payment_config` - Configuration for payment calculations
    /// * `field_mappers` - Mappings for field transformations
    /// * `field_type` - Type of the field (single or collection)
    /// 
    /// # Returns
    /// 
    /// A new SchemaField instance with the specified configurations
    pub fn new(
        permission_policy: PermissionsPolicy, 
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
        field_type: Option<FieldType>,
    ) -> Self {
        Self {
            permission_policy,
            payment_config,
            ref_atom_uuid: None,
            field_mappers,
            field_type: field_type.unwrap_or(FieldType::Single),
        }
    }

    /// Sets the reference to the atom containing this field's value.
    /// 
    /// This builder method links the field to its stored data through
    /// an atom reference. The actual value is retrieved using this
    /// reference when the field is accessed.
    /// 
    /// # Arguments
    /// 
    /// * `ref_atom_uuid` - UUID of the atom containing the field's value
    /// 
    /// # Returns
    /// 
    /// The field instance with the atom reference set
    pub fn with_ref_atom_uuid(mut self, ref_atom_uuid: String) -> Self {
        self.ref_atom_uuid = Some(ref_atom_uuid);
        self
    }

    /// Returns whether this field is a collection
    pub fn is_collection(&self) -> bool {
        self.field_type == FieldType::Collection
    }
}

/// Field type - single value or collection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    /// Single value field
    Single,
    /// Collection of values
    Collection,
}

/// Permission policy for field access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    /// Policy for read access
    pub read_policy: TrustDistance,
    /// Policy for write access
    pub write_policy: TrustDistance,
}

/// Trust distance for permission policies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustDistance {
    /// Specific trust distance (lower means higher trust)
    Distance(u32),
    /// Explicit public key list
    PublicKeys(Vec<String>),
    /// No access allowed
    NoAccess,
}

/// Payment configuration for schema-level access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPaymentConfig {
    /// Base multiplier for all operations
    pub base_multiplier: f64,
    /// Trust distance scaling factor
    pub trust_scaling: TrustDistanceScaling,
    /// Payment threshold
    pub payment_threshold: Option<f64>,
}

/// Payment configuration for field-level access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPaymentConfig {
    /// Base multiplier for field operations
    pub base_multiplier: f64,
    /// Trust distance scaling factor
    pub trust_scaling: TrustDistanceScaling,
    /// Payment threshold
    pub payment_threshold: Option<f64>,
}

/// Trust distance scaling for payment calculations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustDistanceScaling {
    /// No scaling based on trust distance
    None,
    /// Linear scaling based on trust distance
    Linear(f64),
    /// Exponential scaling based on trust distance
    Exponential(f64),
}
