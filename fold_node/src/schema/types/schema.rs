use crate::fees::SchemaPaymentConfig;
use crate::schema::types::field::FieldVariant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fields: HashMap<String, FieldVariant>,
    /// Payment configuration for schema-level access control
    pub payment_config: SchemaPaymentConfig,
}

impl Schema {
    /// Creates a new Schema with the specified name.
    ///
    /// Initializes an empty schema with:
    /// - No fields
    /// - Default payment configuration
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for this schema
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    /// Sets the fields for this schema.
    ///
    /// This builder method allows setting all fields at once,
    /// useful when creating a schema with a predefined set of fields.
    ///
    /// # Arguments
    ///
    /// * `fields` - Map of field names to their definitions
    ///
    /// # Returns
    ///
    /// The schema instance with updated fields
    pub fn with_fields(mut self, fields: HashMap<String, FieldVariant>) -> Self {
        self.fields = fields;
        self
    }

    /// Sets the payment configuration for this schema.
    ///
    /// This builder method configures how payments are calculated
    /// for operations on this schema's data.
    ///
    /// # Arguments
    ///
    /// * `payment_config` - Configuration for payment calculations
    ///
    /// # Returns
    ///
    /// The schema instance with updated payment configuration
    pub fn with_payment_config(mut self, payment_config: SchemaPaymentConfig) -> Self {
        self.payment_config = payment_config;
        self
    }

    /// Adds a single field to the schema.
    ///
    /// This method allows incrementally building the schema by adding
    /// fields one at a time. Each field includes:
    /// - Permission policy for access control
    /// - Payment configuration for data access
    /// - Optional reference to stored data
    /// - Optional field mappings for transformations
    ///
    /// # Arguments
    ///
    /// * `field_name` - Name of the field to add
    /// * `field` - Field definition and configuration
    pub fn add_field(&mut self, field_name: String, field: FieldVariant) {
        self.fields.insert(field_name, field);
    }
}

