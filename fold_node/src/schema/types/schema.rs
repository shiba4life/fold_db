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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
    use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};
    use crate::schema::types::field::{Field, SingleField};
    use uuid::Uuid;

    fn create_field(policy: PermissionsPolicy) -> FieldVariant {
        let mut single_field = SingleField::new(
            policy,
            create_default_payment_config(),
            HashMap::new(),
        );
        single_field.set_ref_atom_uuid(Uuid::new_v4().to_string());
        FieldVariant::Single(single_field)
    }

    fn multi_field_schema() -> Schema {
        let mut schema = Schema::new("test_schema".to_string());
        let fields = vec![
            ("public_field", PermissionsPolicy::default()),
            (
                "protected_field",
                PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(2)),
            ),
            (
                "private_field",
                PermissionsPolicy::new(TrustDistance::Distance(3), TrustDistance::Distance(3)),
            ),
        ];

        for (name, policy) in fields {
            schema.add_field(name.to_string(), create_field(policy));
        }
        schema
    }

    fn create_default_payment_config() -> FieldPaymentConfig {
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
    }

    #[test]
    fn test_schema_creation() {
        let schema_name = "test_schema".to_string();
        let schema = Schema::new(schema_name.clone());

        assert_eq!(schema.name, schema_name);
        assert!(schema.fields.is_empty());
    }

    #[test]
    fn test_schema_field_management() {
        let mut schema = Schema::new("test_schema".to_string());
        let field_name = "test_field".to_string();
        let mut field = create_field(PermissionsPolicy::default());
        field.set_ref_atom_uuid("test-uuid".to_string());

        // Add field
        schema.add_field(field_name.clone(), field.clone());

        // Verify field was added
        assert!(schema.fields.contains_key(&field_name));
        let stored_field = schema.fields.get(&field_name).unwrap();
        assert_eq!(
            stored_field.ref_atom_uuid(),
            Some(&"test-uuid".to_string())
        );
        assert!(stored_field.field_mappers().is_empty());
    }

    #[test]
    fn test_schema_field_permissions() {
        let mut schema = Schema::new("test_schema".to_string());
        let field_name = "protected_field".to_string();

        let field = create_field(PermissionsPolicy::new(
            TrustDistance::Distance(2),
            TrustDistance::Distance(3),
        ));

        schema.add_field(field_name.clone(), field.clone());

        // Verify permissions
        let stored_field = schema.fields.get(&field_name).unwrap();
        match &stored_field.permission_policy().read_policy {
            TrustDistance::Distance(d) => assert_eq!(*d, 2),
            _ => panic!("Expected Distance variant"),
        }
        match &stored_field.permission_policy().write_policy {
            TrustDistance::Distance(d) => assert_eq!(*d, 3),
            _ => panic!("Expected Distance variant"),
        }
    }

    #[test]
    fn test_schema_field_mappers() {
        let mut schema = Schema::new("test_schema".to_string());
        let field_name = "mapped_field".to_string();

        let mut field_mappers = HashMap::new();
        field_mappers.insert("transform".to_string(), "uppercase".to_string());

        let mut single_field = SingleField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            field_mappers.clone(),
        );
        single_field.set_ref_atom_uuid(Uuid::new_v4().to_string());
        let field = FieldVariant::Single(single_field);

        schema.add_field(field_name.clone(), field);

        // Verify field mappers
        let stored_field = schema.fields.get(&field_name).unwrap();
        assert_eq!(*stored_field.field_mappers(), field_mappers);
    }

    #[test]
    fn test_multi_field_count() {
        let schema = multi_field_schema();
        assert_eq!(schema.fields.len(), 3);
    }

    #[test]
    fn test_multi_field_permissions() {
        let schema = multi_field_schema();

        match schema
            .fields
            .get("public_field")
            .unwrap()
            .permission_policy()
            .read_policy
        {
            TrustDistance::Distance(d) => assert_eq!(d, 0),
            _ => panic!("Expected Distance variant"),
        }
        match schema
            .fields
            .get("protected_field")
            .unwrap()
            .permission_policy()
            .read_policy
        {
            TrustDistance::Distance(d) => assert_eq!(d, 1),
            _ => panic!("Expected Distance variant"),
        }
        match schema
            .fields
            .get("private_field")
            .unwrap()
            .permission_policy()
            .read_policy
        {
            TrustDistance::Distance(d) => assert_eq!(d, 3),
            _ => panic!("Expected Distance variant"),
        }
    }

    #[test]
    fn test_schema_deserialization_with_field_transforms() {
        let json_input = "{
            \"name\": \"test_schema_with_transforms\",
            \"fields\": {
                \"calculated_field\": {
                    \"permission_policy\": {
                        \"read_policy\": { \"Distance\": 0 },
                        \"write_policy\": { \"Distance\": 0 }
                    },
                    \"payment_config\": {
                        \"base_multiplier\": 0.5,
                        \"trust_distance_scaling\": \"None\",
                        \"min_payment\": null
                    },
                    \"ref_atom_uuid\": null,
                    \"field_type\": \"Single\",
                    \"field_mappers\": {},
                    \"transform\": \"transform temp_calc { logic: { return 1; } }\"
                }
            },
            \"payment_config\": {
                \"base_multiplier\": 1.0,
                \"min_payment_threshold\": 0
            }
        }";

        let schema: Schema = serde_json::from_str(json_input).expect("Failed to deserialize schema");

        assert_eq!(schema.name, "test_schema_with_transforms");
        assert_eq!(schema.fields.len(), 1);

        let calculated_field = schema.fields.get("calculated_field").expect("calculated_field not found");
        assert!(calculated_field.transform().is_some());
        assert_eq!(calculated_field.transform().unwrap().logic, "return 1");
    }
}
