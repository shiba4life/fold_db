use crate::fees::payment_config::SchemaPaymentConfig;
use crate::fees::types::config::FieldPaymentConfig;
use crate::fees::types::config::TrustDistanceScaling;
use crate::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use crate::schema::types::field::FieldType;
use crate::schema::types::SchemaError;
use crate::schema::types::Transform;
use crate::transform::parser::TransformParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete JSON schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    pub name: String,
    #[serde(default = "crate::schema::types::schema::default_schema_type")]
    pub schema_type: crate::schema::types::schema::SchemaType,
    pub fields: HashMap<String, JsonSchemaField>,
    pub payment_config: SchemaPaymentConfig,
    /// SHA256 hash of the schema content for integrity verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
}

/// Represents a field in the JSON schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    #[serde(default)]
    pub ref_atom_uuid: Option<String>,
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    #[serde(default = "default_field_type")]
    pub field_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<JsonTransform>,
}

/// JSON representation of a transform
///
/// Only the required pieces of information are kept. Any unknown
/// fields in the incoming JSON will cause a deserialization error so
/// that stale attributes such as `reversible` or `signature` do not
/// silently pass through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JsonTransform {
    /// The transform logic expressed in the DSL
    pub logic: String,

    /// Explicit list of input fields in `Schema.field` format
    #[serde(default)]
    pub inputs: Vec<String>,

    /// Output field for this transform in `Schema.field` format
    pub output: String,
}

/// JSON representation of permission policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonPermissionPolicy {
    #[serde(rename = "read_policy")]
    pub read: TrustDistance,
    #[serde(rename = "write_policy")]
    pub write: TrustDistance,
    #[serde(rename = "explicit_read_policy")]
    pub explicit_read: Option<ExplicitCounts>,
    #[serde(rename = "explicit_write_policy")]
    pub explicit_write: Option<ExplicitCounts>,
}

/// JSON representation of field payment config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFieldPaymentConfig {
    pub base_multiplier: f64,
    pub trust_distance_scaling: TrustDistanceScaling,
    pub min_payment: Option<u64>,
}

impl From<JsonPermissionPolicy> for PermissionsPolicy {
    fn from(json: JsonPermissionPolicy) -> Self {
        Self {
            read_policy: json.read,
            write_policy: json.write,
            explicit_read_policy: json.explicit_read,
            explicit_write_policy: json.explicit_write,
        }
    }
}

impl From<JsonFieldPaymentConfig> for FieldPaymentConfig {
    fn from(json: JsonFieldPaymentConfig) -> Self {
        Self {
            base_multiplier: json.base_multiplier,
            trust_distance_scaling: json.trust_distance_scaling,
            min_payment: json.min_payment,
        }
    }
}

impl From<JsonTransform> for Transform {
    fn from(json: JsonTransform) -> Self {
        let mut transform = Transform::new(json.logic, json.output);
        transform.set_inputs(json.inputs);
        transform
    }
}

fn default_field_type() -> FieldType {
    FieldType::Single
}

impl JsonSchemaDefinition {
    /// Validates the schema definition according to the rules.
    ///
    /// # Errors
    /// Returns a `SchemaError::InvalidField` if:
    /// - The schema's base multiplier is not positive
    /// - Any field's base multiplier is not positive
    /// - Any field's min factor is less than 1.0
    /// - Any field's min payment is zero when specified
    pub fn validate(&self) -> Result<(), SchemaError> {
        // Base multiplier must be positive
        if self.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                "Schema base_multiplier must be positive".to_string(),
            ));
        }

        // Validate each field
        for (field_name, field) in &self.fields {
            Self::validate_field(field_name, field)?;
        }

        Ok(())
    }

    fn validate_field(field_name: &str, field: &JsonSchemaField) -> Result<(), SchemaError> {
        // Validate payment config
        if field.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(format!(
                "Field {field_name} base_multiplier must be positive"
            )));
        }

        // Validate trust distance scaling
        match &field.payment_config.trust_distance_scaling {
            TrustDistanceScaling::Linear { min_factor, .. }
            | TrustDistanceScaling::Exponential { min_factor, .. } => {
                if *min_factor < 1.0 {
                    return Err(SchemaError::InvalidField(format!(
                        "Field {field_name} min_factor must be >= 1.0"
                    )));
                }
            }
            TrustDistanceScaling::None => {}
        }

        if let Some(min_payment) = field.payment_config.min_payment {
            if min_payment == 0 {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} min_payment cannot be zero"
                )));
            }
        }

        // Validate transform if present
        if let Some(transform) = &field.transform {
            // Logic cannot be empty
            if transform.logic.is_empty() {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} transform logic cannot be empty"
                )));
            }

            // Parse transform logic using the DSL parser
            let parser = TransformParser::new();
            parser
                .parse_expression(&transform.logic)
                .map_err(|e| {
                    SchemaError::InvalidField(format!(
                        "Error parsing transform for field {field_name}: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }
}
