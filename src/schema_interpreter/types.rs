use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::{TrustDistance, ExplicitCounts, PermissionsPolicy};
use crate::fees::types::config::FieldPaymentConfig;
use crate::fees::payment_config::SchemaPaymentConfig;
use crate::fees::types::config::TrustDistanceScaling;
use crate::schema::mapper::types::MappingRule;

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
            JsonMappingRule::Rename { source_field, target_field } => {
                MappingRule::Rename { source_field, target_field }
            }
            JsonMappingRule::Drop { field } => {
                MappingRule::Drop { field }
            }
            JsonMappingRule::Map { source_field, target_field, function } => {
                MappingRule::Map { 
                    source_field,
                    target_field,
                    function
                }
            }
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
    pub read_policy: TrustDistance,
    pub write_policy: TrustDistance,
    pub explicit_read_policy: Option<ExplicitCounts>,
    pub explicit_write_policy: Option<ExplicitCounts>,
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
    Drop {
        field: String,
    },
    #[serde(rename = "map")]
    Map {
        source_field: String,
        target_field: String,
        function: Option<String>,
    },
}

impl From<JsonPermissionPolicy> for PermissionsPolicy {
    fn from(json: JsonPermissionPolicy) -> Self {
        Self {
            read_policy: json.read_policy,
            write_policy: json.write_policy,
            explicit_read_policy: json.explicit_read_policy,
            explicit_write_policy: json.explicit_write_policy,
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
    /// Validates the schema definition according to the rules
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
                return Err(crate::schema::types::SchemaError::InvalidField(
                    format!("Field {} base_multiplier must be positive", field_name),
                ));
            }

            // Validate trust distance scaling
            match &field.payment_config.trust_distance_scaling {
                TrustDistanceScaling::Linear { min_factor, .. } |
                TrustDistanceScaling::Exponential { min_factor, .. } => {
                    if *min_factor < 1.0 {
                        return Err(crate::schema::types::SchemaError::InvalidField(
                            format!("Field {} min_factor must be >= 1.0", field_name),
                        ));
                    }
                }
                TrustDistanceScaling::None => {}
            }

            // Validate trust distances are non-negative
            if let TrustDistance::Distance(d) = field.permission_policy.read_policy {
                if d < 0 {
                    return Err(crate::schema::types::SchemaError::InvalidField(
                        format!("Field {} read_policy distance must be non-negative", field_name),
                    ));
                }
            }
            if let TrustDistance::Distance(d) = field.permission_policy.write_policy {
                if d < 0 {
                    return Err(crate::schema::types::SchemaError::InvalidField(
                        format!("Field {} write_policy distance must be non-negative", field_name),
                    ));
                }
            }
        }

        Ok(())
    }
}
