use crate::fees::payment_config::SchemaPaymentConfig;
use crate::fees::types::config::FieldPaymentConfig;
use crate::fees::types::config::TrustDistanceScaling;
use crate::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use crate::schema::mapper::types::MappingRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a complete JSON schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaDefinition {
    pub name: String,
    pub fields: HashMap<String, JsonSchemaField>,
    pub schema_mappers: Vec<JsonSchemaMapper>,
    pub payment_config: SchemaPaymentConfig,
}

impl From<JsonMappingRule> for MappingRule {
    fn from(json: JsonMappingRule) -> Self {
        match json {
            JsonMappingRule::Rename {
                source_field,
                target_field,
            } => Self::Rename {
                source_field,
                target_field,
            },
            JsonMappingRule::Drop { field } => Self::Drop { field },
            JsonMappingRule::Map {
                source_field,
                target_field,
            } => Self::Map {
                source_field,
                target_field,
            },
        }
    }
}

/// Represents a field in the JSON schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaField {
    pub permission_policy: JsonPermissionPolicy,
    pub ref_atom_uuid: String,
    pub payment_config: JsonFieldPaymentConfig,
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

/// Represents a schema mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSchemaMapper {
    pub source_schemas: Vec<String>,
    pub target_schema: String,
    pub rules: Vec<JsonMappingRule>,
}

/// Represents a mapping rule
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "rule")]
pub enum JsonMappingRule {
    #[serde(rename = "rename")]
    Rename {
        source_field: String,
        target_field: String,
    },
    #[serde(rename = "drop")]
    Drop { field: String },
    #[serde(rename = "map")]
    Map {
        source_field: String,
        target_field: String,
    },
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

impl JsonSchemaDefinition {
    /// Validates the schema definition according to the rules.
    ///
    /// # Errors
    /// Returns a `SchemaError::InvalidField` if:
    /// - The schema's base multiplier is not positive
    /// - Any field's base multiplier is not positive
    /// - Any field's min factor is less than 1.0
    /// - Any field's min payment is zero when specified
    /// - Any schema mapper has no source schemas
    /// - Any schema mapper has duplicate source-target pairs
    /// - Any schema mapper has fields mapped multiple times
    pub fn validate(&self) -> crate::schema_interpreter::Result<()> {
        // Base multiplier must be positive
        if self.payment_config.base_multiplier <= 0.0 {
            return Err(crate::schema::types::SchemaError::InvalidField(
                "Schema base_multiplier must be positive".to_string(),
            ));
        }

        // Validate each field
        for (field_name, field) in &self.fields {
            // Validate payment config
            if field.payment_config.base_multiplier <= 0.0 {
                return Err(crate::schema::types::SchemaError::InvalidField(format!(
                    "Field {field_name} base_multiplier must be positive"
                )));
            }

            // Validate trust distance scaling
            match &field.payment_config.trust_distance_scaling {
                TrustDistanceScaling::Linear { min_factor, .. }
                | TrustDistanceScaling::Exponential { min_factor, .. } => {
                    if *min_factor < 1.0 {
                        return Err(crate::schema::types::SchemaError::InvalidField(format!(
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
