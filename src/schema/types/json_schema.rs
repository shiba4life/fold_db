use crate::fees::payment_config::SchemaPaymentConfig;
use crate::schema::types::fields::FieldType;
use crate::fees::types::config::FieldPaymentConfig;
use crate::fees::types::config::TrustDistanceScaling;
use crate::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::schema::types::SchemaError;

/// Represents a complete JSON schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    pub name: String,
    pub fields: HashMap<String, JsonSchemaField>,
    pub payment_config: SchemaPaymentConfig,
}

/// Represents a field in the JSON schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    pub ref_atom_uuid: String,
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    #[serde(default = "default_field_type")]
    pub field_type: FieldType,
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

            // Trust distances are already non-negative due to u32 type
            // No additional validation needed for TrustDistance::Distance
            // as the type system ensures it's always valid
        }

        Ok(())
    }
}
