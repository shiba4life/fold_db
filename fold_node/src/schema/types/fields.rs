use serde::{Deserialize, Serialize, Deserializer};
use std::fmt;
use std::collections::HashMap;
use uuid::Uuid;

use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::Transform;
use crate::schema::transform::parser::TransformParser; // Import TransformParser

#[derive(Debug, Clone, Serialize, serde::Deserialize, PartialEq)] // Explicitly use serde::Deserialize
#[serde(rename_all = "PascalCase")]
pub enum FieldType {
    Single,
    Collection,
}

impl fmt::Display for FieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldType::Single => write!(f, "single"),
            FieldType::Collection => write!(f, "collection"),
        }
    }
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
#[derive(Debug, Clone, Serialize)] // Removed Deserialize
pub struct SchemaField {
    /// Permission policy controlling read/write access to this field
    pub permission_policy: PermissionsPolicy,

    /// Payment configuration for accessing this field's data
    pub payment_config: FieldPaymentConfig,

    /// Reference to the atom containing this field's value
    /// The actual field value is fetched through this reference
    ref_atom_uuid: Option<String>,

    /// Type of the field - single value or collection
    field_type: FieldType,

    /// Mappings for field transformations and schema evolution
    /// Keys are source schema names, values are source field names
    pub field_mappers: HashMap<String, String>,

    /// Optional transform for this field
    /// Defines how data from source fields is processed to produce a derived value
    pub transform: Option<Transform>,
}

// Manual implementation of Deserialize for SchemaField to parse the transform string
impl<'de> Deserialize<'de> for SchemaField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct SchemaFieldHelper {
            permission_policy: PermissionsPolicy,
            payment_config: FieldPaymentConfig,
            ref_atom_uuid: Option<String>,
            field_type: Option<FieldType>, // Make field_type optional for deserialization
            field_mappers: HashMap<String, String>,
            transform: Option<String>, // Deserialize transform as a string
        }

        let helper = SchemaFieldHelper::deserialize(deserializer)?;

        let parsed_transform = helper.transform.map(|transform_logic| {
            let parser = TransformParser::new();
            match parser.parse_transform(&transform_logic) {
                Ok(declaration) => Ok(Transform::from_declaration(declaration)),
                Err(e) => Err(serde::de::Error::custom(format!("Error parsing transform: {}", e))), // Include SchemaError details
            }
        }).transpose()?; // Use transpose to convert Option<Result<T, E>> to Result<Option<T>, E>


        Ok(SchemaField {
            permission_policy: helper.permission_policy,
            payment_config: helper.payment_config,
            ref_atom_uuid: helper.ref_atom_uuid,
            field_type: helper.field_type.unwrap_or(FieldType::Single), // Provide default if missing
            field_mappers: helper.field_mappers,
            transform: parsed_transform,
        })
    }
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
    pub fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
        field_type: Option<FieldType>,
    ) -> Self {
        Self {
            permission_policy,
            payment_config,
            ref_atom_uuid: Some(Uuid::new_v4().to_string()),
            field_mappers,
            field_type: field_type.unwrap_or(FieldType::Single),
            transform: None,
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

    /// Returns whether this field is a collection
    #[must_use]
    pub fn is_collection(&self) -> bool {
        self.field_type == FieldType::Collection
    }

    /// Returns the type of this field
    #[must_use]
    pub fn field_type(&self) -> &FieldType {
        &self.field_type
    }

    /// Sets the type of this field
    pub fn set_field_type(&mut self, field_type: FieldType) {
        self.field_type = field_type;
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

    /// Sets a transform for this field.
    ///
    /// This builder method adds a transform that defines how data from
    /// source fields is processed to produce a derived value.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to apply to this field
    ///
    /// # Returns
    ///
    /// The field instance with the transform set
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Gets the transform for this field, if any.
    ///
    /// # Returns
    ///
    /// An optional reference to the transform
    pub fn get_transform(&self) -> Option<&Transform> {
        self.transform.as_ref()
    }

    /// Sets the transform for this field.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to apply to this field
    pub fn set_transform(&mut self, transform: Transform) {
        self.transform = Some(transform);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::TrustDistanceScaling;
    use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};
    use serde_json;

    fn create_default_payment_config() -> FieldPaymentConfig {
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
    }

    #[test]
    fn test_field_creation() {
        let field = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        );
        assert!(field.ref_atom_uuid.is_some());
        assert!(field.field_mappers.is_empty());
        assert_eq!(field.field_type, FieldType::Single);
        assert!(field.transform.is_none());
    }

    #[test]
    fn test_field_with_atom_uuid() {
        let field = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_ref_atom_uuid("test-uuid".to_string());
        assert_eq!(field.get_ref_atom_uuid(), Some("test-uuid".to_string()));
    }

    #[test]
    fn test_field_with_mappers() {
        let mut mappers = HashMap::new();
        mappers.insert("source_schema".to_string(), "source_field".to_string());
        let field = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_field_mappers(mappers.clone());
        assert_eq!(field.field_mappers, mappers);
    }

    #[test]
    fn test_field_type() {
        let field_single = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        );
        assert_eq!(field_single.field_type(), &FieldType::Single);
        assert!(!field_single.is_collection());

        let field_collection = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Collection),
        );
        assert_eq!(field_collection.field_type(), &FieldType::Collection);
        assert!(field_collection.is_collection());
    }

    #[test]
    fn test_field_with_transform() {
        let transform = Transform::new("return field1 + field2;".to_string(), false, None, false);
        let field = SchemaField::new(
            PermissionsPolicy::default(),
            create_default_payment_config(),
            HashMap::new(),
            Some(FieldType::Single),
        )
        .with_transform(transform.clone());
        assert!(field.get_transform().is_some());
        assert_eq!(field.get_transform().unwrap().logic, "return field1 + field2;".to_string());
    }

    #[test]
    fn test_field_deserialization_with_transform() {
        let json_input = r#"{
            "permission_policy": {
                "read_policy": { "Distance": 0 },
                "write_policy": { "Distance": 0 }
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "trust_distance_scaling": "None",
                "min_payment": null
            },
            "ref_atom_uuid": "some-uuid",
            "field_type": "Single",
            "field_mappers": {},
            "transform": "transform temp { output: Placeholder<Any> as \"out\" logic: { return field1 * 2; } }"
        }"#;

        let field: SchemaField = serde_json::from_str(json_input).unwrap();
        assert!(field.transform.is_some());
        let transform = field.transform.unwrap();
        assert_eq!(transform.logic, "return (field1 * 2)".to_string());
        // Further assertions can be added to check the parsed AST if needed
    }

    #[test]
    fn test_field_deserialization_without_transform() {
        let json_input = r#"{
            "permission_policy": {
                "read_policy": { "Distance": 0 },
                "write_policy": { "Distance": 0 }
            },
            "payment_config": {
                "base_multiplier": 1.0,
                "trust_distance_scaling": "None",
                "min_payment": null
            },
            "ref_atom_uuid": "some-uuid",
            "field_type": "Single",
            "field_mappers": {}
        }"#;

        let field: SchemaField = serde_json::from_str(json_input).unwrap();
        assert!(field.transform.is_none());
    }
}
