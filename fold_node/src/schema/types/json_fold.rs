use crate::fees::payment_config::SchemaPaymentConfig;
use crate::fees::types::config::TrustDistanceScaling;
use crate::schema::types::fields::FieldType;
use crate::schema::types::{SchemaError, Fold, FieldVariant, SingleField, CollectionField, RangeField};
use crate::schema::types::field::Field;
use crate::schema::types::json_schema::{JsonPermissionPolicy, JsonFieldPaymentConfig, JsonTransform};
use crate::transform::parser::TransformParser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFoldDefinition {
    pub name: String,
    pub fields: HashMap<String, JsonFoldField>,
    pub payment_config: SchemaPaymentConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonFoldField {
    pub permission_policy: JsonPermissionPolicy,
    pub ref_atom_uuid: String,
    pub payment_config: JsonFieldPaymentConfig,
    pub field_mappers: HashMap<String, String>,
    #[serde(default = "default_field_type")]
    pub field_type: FieldType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<JsonTransform>,
}

fn default_field_type() -> FieldType {
    FieldType::Single
}

impl JsonFoldDefinition {
    pub fn validate(&self) -> Result<(), SchemaError> {
        if self.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(
                "Fold base_multiplier must be positive".to_string(),
            ));
        }

        for (field_name, field) in &self.fields {
            Self::validate_field(field_name, field)?;
        }
        Ok(())
    }

    fn validate_field(field_name: &str, field: &JsonFoldField) -> Result<(), SchemaError> {
        if field.payment_config.base_multiplier <= 0.0 {
            return Err(SchemaError::InvalidField(format!(
                "Field {field_name} base_multiplier must be positive"
            )));
        }

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

        if let Some(transform) = &field.transform {
            if transform.logic.is_empty() {
                return Err(SchemaError::InvalidField(format!(
                    "Field {field_name} transform logic cannot be empty"
                )));
            }
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

impl TryFrom<JsonFoldDefinition> for Fold {
    type Error = SchemaError;

    fn try_from(def: JsonFoldDefinition) -> Result<Self, Self::Error> {
        def.validate()?;
        let fields = def
            .fields
            .into_iter()
            .map(|(name, field)| (name, convert_field(field)))
            .collect();
        Ok(Fold {
            name: def.name,
            fields,
            payment_config: def.payment_config,
        })
    }
}

fn convert_field(json_field: JsonFoldField) -> FieldVariant {
    let mut variant = match json_field.field_type {
        FieldType::Single => {
            let mut f = SingleField::new(
                json_field.permission_policy.into(),
                json_field.payment_config.into(),
                json_field.field_mappers,
            );
            FieldVariant::Single(f)
        }
        FieldType::Collection => {
            let mut f = CollectionField::new(
                json_field.permission_policy.into(),
                json_field.payment_config.into(),
                json_field.field_mappers,
            );
            FieldVariant::Collection(f)
        }
        FieldType::Range => {
            let mut f = RangeField::new(
                json_field.permission_policy.into(),
                json_field.payment_config.into(),
                json_field.field_mappers,
            );
            FieldVariant::Range(f)
        }
    };

    variant.set_ref_atom_uuid(json_field.ref_atom_uuid);
    if let Some(t) = json_field.transform {
        variant.set_transform(t.into());
    }
    variant
}
