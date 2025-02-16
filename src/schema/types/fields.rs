use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::PermissionsPolicy;
use crate::fees::types::config::FieldPaymentConfig;
use std::collections::HashMap;
use uuid::Uuid;

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
    ref_atom_uuid: Option<String>,
    
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
    /// 
    /// # Returns
    /// 
    /// A new SchemaField instance with the specified configurations
    #[must_use]
    pub fn new(permission_policy: PermissionsPolicy, payment_config: FieldPaymentConfig, 
        field_mappers: HashMap<String, String>) -> Self {
        Self {
            permission_policy,
            payment_config,
            ref_atom_uuid: Some(Uuid::new_v4().to_string()),
            field_mappers,
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

    pub fn get_ref_atom_uuid(&self) -> Option<String> {
        self.ref_atom_uuid.clone()
    }

    /// Sets the field mappings for schema transformation.
    /// 
    /// This builder method configures how this field maps to fields
    /// in other schemas, enabling:
    /// - Schema evolution and versioning
    /// - Data transformation between schemas
    /// - Field value inheritance
    /// 
    /// # Arguments
    /// 
    /// * `field_mappers` - Map of schema names to field names defining transformations
    /// 
    /// # Returns
    /// 
    /// The field instance with the mappings configured
    pub fn with_field_mappers(mut self, field_mappers: HashMap<String, String>) -> Self {
        self.field_mappers = field_mappers;
        self
    }

    pub fn set_ref_atom_uuid(&mut self, ref_atom_uuid: String) {
        self.ref_atom_uuid = Some(ref_atom_uuid);
    }
}
