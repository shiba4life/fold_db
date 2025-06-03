use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub schema_name: String,
    pub fields: Vec<String>,
    pub pub_key: String,
    pub trust_distance: u32,
    pub filter: Option<Value>,
}

impl Query {
    #[must_use]
    pub fn new(
        schema_name: String,
        fields: Vec<String>,
        pub_key: String,
        trust_distance: u32,
    ) -> Self {
        Self {
            schema_name,
            fields,
            pub_key,
            trust_distance,
            filter: None,
        }
    }

    #[must_use]
    pub fn new_with_filter(
        schema_name: String,
        fields: Vec<String>,
        pub_key: String,
        trust_distance: u32,
        filter: Option<Value>,
    ) -> Self {
        Self {
            schema_name,
            fields,
            pub_key,
            trust_distance,
            filter,
        }
    }
}

#[derive(Debug, Clone, Serialize, ValueEnum)]
pub enum MutationType {
    Create,
    Update,
    Delete,
    #[clap(skip)]
    AddToCollection(String),
    #[clap(skip)]
    UpdateToCollection(String),
    #[clap(skip)]
    DeleteFromCollection(String),
}

impl<'de> Deserialize<'de> for MutationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "create" => Ok(MutationType::Create),
            "update" => Ok(MutationType::Update),
            "delete" => Ok(MutationType::Delete),
            s if s.starts_with("add_to_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::AddToCollection(id))
            }
            s if s.starts_with("update_to_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::UpdateToCollection(id))
            }
            s if s.starts_with("delete_from_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::DeleteFromCollection(id))
            }
            _ => Err(serde::de::Error::custom("unknown mutation type")),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Mutation {
    pub schema_name: String,
    pub fields_and_values: HashMap<String, Value>,
    pub pub_key: String,
    pub trust_distance: u32,
    pub mutation_type: MutationType,
}

impl Mutation {
    #[must_use]
    pub const fn new(
        schema_name: String,
        fields_and_values: HashMap<String, Value>,
        pub_key: String,
        trust_distance: u32,
        mutation_type: MutationType,
    ) -> Self {
        Self {
            schema_name,
            fields_and_values,
            pub_key,
            trust_distance,
            mutation_type,
        }
    }

    /// Convert this mutation into a RangeSchemaMutation by populating the range_key in every field's value.
    /// The range_key field itself is left as-is (primitive value), while other fields get the range_key inserted.
    ///
    /// This method ensures that:
    /// - The mutation contains the required range_key field
    /// - The range_key value is valid (not null or empty)
    /// - All non-range_key fields that are objects get the range_key value applied
    pub fn to_range_schema_mutation(
        &self,
        schema: &crate::schema::types::Schema,
    ) -> Result<Self, crate::schema::types::SchemaError> {
        use serde_json::Value;
        if let Some(range_key) = schema.range_key() {
            // MANDATORY: Get the value for the range_key field from the mutation
            let range_key_value = self.fields_and_values.get(range_key).ok_or_else(|| {
                crate::schema::types::SchemaError::InvalidData(format!(
                    "Range schema mutation for '{}' is missing required range_key field '{}'. All range schema mutations must provide a value for the range_key.",
                    self.schema_name, range_key
                ))
            })?;

            // Validate the range_key value is not null
            if range_key_value.is_null() {
                return Err(crate::schema::types::SchemaError::InvalidData(format!(
                    "Range schema mutation for '{}' has null value for range_key field '{}'. Range key must have a valid value.",
                    self.schema_name, range_key
                )));
            }

            // If range_key value is a string, ensure it's not empty
            if let Some(str_value) = range_key_value.as_str() {
                if str_value.trim().is_empty() {
                    return Err(crate::schema::types::SchemaError::InvalidData(format!(
                        "Range schema mutation for '{}' has empty string value for range_key field '{}'. Range key must have a non-empty value.",
                        self.schema_name, range_key
                    )));
                }
            }

            // For each field except the range_key field itself, insert/overwrite the range_key in its value
            let mut new_fields_and_values = self.fields_and_values.clone();
            for (field_name, value) in new_fields_and_values.iter_mut() {
                // Skip the range_key field - it should remain as a primitive value
                if field_name == range_key {
                    continue;
                }

                // Only update if the value is an object
                if let Value::Object(obj) = value {
                    obj.insert(range_key.to_string(), range_key_value.clone());
                }
            }

            Ok(Self {
                schema_name: self.schema_name.clone(),
                fields_and_values: new_fields_and_values,
                pub_key: self.pub_key.clone(),
                trust_distance: self.trust_distance,
                mutation_type: self.mutation_type.clone(),
            })
        } else {
            Err(crate::schema::types::SchemaError::InvalidData(format!(
                "Schema '{}' is not a RangeSchema. Only range schemas support range_key propagation.",
                self.schema_name
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::types::field::{FieldVariant, RangeField};
    use crate::schema::types::operations::MutationType;
    use crate::schema::types::Schema;
    use crate::schema::types::SchemaError;
    use crate::testing::{FieldPaymentConfig, PermissionsPolicy};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_to_range_schema_mutation_populates_range_key() {
        // Create a RangeSchema with range_key "user_id" and two fields
        let mut schema = Schema::new_range("TestRange".to_string(), "user_id".to_string());
        let rf = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        schema
            .fields
            .insert("user_id".to_string(), FieldVariant::Range(rf.clone()));
        schema
            .fields
            .insert("score".to_string(), FieldVariant::Range(rf));

        // Create a mutation with user_id and score fields
        let mut fields_and_values = HashMap::new();
        fields_and_values.insert("user_id".to_string(), json!(123));
        fields_and_values.insert("score".to_string(), json!({"points": 42}));

        let mutation = Mutation {
            schema_name: "TestRange".to_string(),
            fields_and_values,
            pub_key: "test".to_string(),
            trust_distance: 0,
            mutation_type: MutationType::Create,
        };

        let result = mutation.to_range_schema_mutation(&schema).unwrap();
        // The score field should now have "user_id": 123
        let score_val = result.fields_and_values.get("score").unwrap();
        assert_eq!(score_val.get("user_id").unwrap(), &json!(123));
        // The user_id field should remain as 123 (not an object, so not changed)
        assert_eq!(
            result.fields_and_values.get("user_id").unwrap(),
            &json!(123)
        );
    }

    #[test]
    fn test_to_range_schema_mutation_missing_key() {
        // Create a RangeSchema with range_key "user_id"
        let mut schema = Schema::new_range("TestRange".to_string(), "user_id".to_string());
        let rf = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        schema
            .fields
            .insert("user_id".to_string(), FieldVariant::Range(rf.clone()));
        schema
            .fields
            .insert("score".to_string(), FieldVariant::Range(rf));

        // Create a mutation missing the user_id field
        let mut fields_and_values = HashMap::new();
        fields_and_values.insert("score".to_string(), json!({"points": 42}));

        let mutation = Mutation {
            schema_name: "TestRange".to_string(),
            fields_and_values,
            pub_key: "test".to_string(),
            trust_distance: 0,
            mutation_type: MutationType::Create,
        };

        let result = mutation.to_range_schema_mutation(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidData(_))));

        // Verify the error message mentions the missing range_key requirement
        if let Err(SchemaError::InvalidData(msg)) = result {
            assert!(msg.contains("missing required range_key field"));
            assert!(msg.contains("user_id"));
        }
    }

    #[test]
    fn test_to_range_schema_mutation_null_range_key() {
        // Create a RangeSchema with range_key "user_id"
        let mut schema = Schema::new_range("TestRange".to_string(), "user_id".to_string());
        let rf = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        schema
            .fields
            .insert("user_id".to_string(), FieldVariant::Range(rf.clone()));
        schema
            .fields
            .insert("score".to_string(), FieldVariant::Range(rf));

        // Create a mutation with null user_id field
        let mut fields_and_values = HashMap::new();
        fields_and_values.insert("user_id".to_string(), json!(null));
        fields_and_values.insert("score".to_string(), json!({"points": 42}));

        let mutation = Mutation {
            schema_name: "TestRange".to_string(),
            fields_and_values,
            pub_key: "test".to_string(),
            trust_distance: 0,
            mutation_type: MutationType::Create,
        };

        let result = mutation.to_range_schema_mutation(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidData(_))));

        // Verify the error message mentions null value
        if let Err(SchemaError::InvalidData(msg)) = result {
            assert!(msg.contains("null value for range_key field"));
        }
    }

    #[test]
    fn test_to_range_schema_mutation_empty_string_range_key() {
        // Create a RangeSchema with range_key "user_id"
        let mut schema = Schema::new_range("TestRange".to_string(), "user_id".to_string());
        let rf = RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );
        schema
            .fields
            .insert("user_id".to_string(), FieldVariant::Range(rf.clone()));
        schema
            .fields
            .insert("score".to_string(), FieldVariant::Range(rf));

        // Create a mutation with empty string user_id field
        let mut fields_and_values = HashMap::new();
        fields_and_values.insert("user_id".to_string(), json!(""));
        fields_and_values.insert("score".to_string(), json!({"points": 42}));

        let mutation = Mutation {
            schema_name: "TestRange".to_string(),
            fields_and_values,
            pub_key: "test".to_string(),
            trust_distance: 0,
            mutation_type: MutationType::Create,
        };

        let result = mutation.to_range_schema_mutation(&schema);
        assert!(matches!(result, Err(SchemaError::InvalidData(_))));

        // Verify the error message mentions empty string value
        if let Err(SchemaError::InvalidData(msg)) = result {
            assert!(msg.contains("empty string value for range_key field"));
        }
    }
}
