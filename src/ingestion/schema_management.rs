//! Schema preparation, creation, parsing, and field management

use crate::fold_db_core::FoldDB;
use crate::ingestion::{
    openrouter_service::AISchemaResponse,
    schema_stripper::SchemaStripper,
    IngestionError, IngestionResult,
};
use crate::schema::types::{JsonSchemaDefinition, field::{FieldVariant, SingleField}};
use crate::schema::{Schema, SchemaCore};
use log::info;
use serde_json::Value;
use std::sync::Arc;

pub struct SchemaManager {
    schema_stripper: SchemaStripper,
    schema_core: Arc<SchemaCore>,
}

impl SchemaManager {
    pub fn new(schema_core: Arc<SchemaCore>) -> Self {
        Self {
            schema_stripper: SchemaStripper::new(),
            schema_core,
        }
    }

    /// Prepares available schemas for AI recommendation.
    pub async fn prepare_schemas(&self) -> IngestionResult<Value> {
        let available_schemas = self.get_stripped_available_schemas().await?;
        let schema_count = if let Some(obj) = available_schemas.as_object() {
            obj.len()
        } else {
            0
        };
        info!("Retrieved {} available schemas", schema_count);
        Ok(available_schemas)
    }

    /// Sets up the schema to use (existing or newly created).
    pub async fn setup_schema(
        &self,
        ai_response: &AISchemaResponse,
    ) -> IngestionResult<(String, bool)> {
        let schema_name = self.determine_schema_to_use(ai_response).await?;
        let new_schema_created = ai_response.new_schemas.is_some();
        Ok((schema_name, new_schema_created))
    }

    /// Get available schemas stripped of payment and permission data
    async fn get_stripped_available_schemas(&self) -> IngestionResult<Value> {
        // Get all available schemas from SchemaCore
        let available_schema_names = self
            .schema_core
            .list_available_schemas()
            .map_err(IngestionError::SchemaSystemError)?;

        let mut schemas = Vec::new();
        for schema_name in available_schema_names {
            if let Ok(Some(schema)) = self.schema_core.get_schema(&schema_name) {
                schemas.push(schema);
            }
        }

        // Strip payment and permission data
        self.schema_stripper
            .create_ai_schema_representation(&schemas)
    }

    /// Determine which schema to use based on AI response
    async fn determine_schema_to_use(
        &self,
        ai_response: &AISchemaResponse,
    ) -> IngestionResult<String> {
        // If existing schemas were recommended, use the first one
        if !ai_response.existing_schemas.is_empty() {
            let schema_name = &ai_response.existing_schemas[0];
            info!("Using existing schema: {}", schema_name);
            return Ok(schema_name.clone());
        }

        // If a new schema was provided, create it
        if let Some(new_schema_def) = &ai_response.new_schemas {
            let schema_name = self.create_new_schema(new_schema_def).await?;
            info!("Created new schema: {}", schema_name);
            return Ok(schema_name);
        }

        Err(IngestionError::ai_response_validation_error(
            "AI response contains neither existing schemas nor new schema definition",
        ))
    }

    /// Create a new schema from AI response
    async fn create_new_schema(&self, schema_def: &Value) -> IngestionResult<String> {
        info!("Creating new schema from AI definition");

        // Parse the schema definition
        let schema = self.parse_schema_definition(schema_def)?;
        let schema_name = schema.name.clone();

        // Load the schema into SchemaCore
        self.schema_core
            .load_schema_internal(schema)
            .map_err(IngestionError::SchemaSystemError)?;

        // Set the schema to approved state
        self.schema_core
            .approve_schema(&schema_name)
            .map_err(IngestionError::SchemaSystemError)?;

        info!("New schema '{}' created and approved", schema_name);
        Ok(schema_name)
    }

    /// Parse schema definition from AI response
    fn parse_schema_definition(&self, schema_def: &Value) -> IngestionResult<Schema> {
        // Try to parse as a complete Schema first
        if let Ok(schema) = serde_json::from_value::<Schema>(schema_def.clone()) {
            return Ok(schema);
        }

        // Try to parse as JsonSchemaDefinition
        if let Ok(json_schema) = serde_json::from_value::<JsonSchemaDefinition>(schema_def.clone())
        {
            return self
                .schema_core
                .interpret_schema(json_schema)
                .map_err(IngestionError::SchemaSystemError);
        }

        // Try to create a basic schema from the definition
        self.create_basic_schema_from_definition(schema_def)
    }

    /// Create a basic schema from a simple definition
    pub fn create_basic_schema_from_definition(&self, schema_def: &Value) -> IngestionResult<Schema> {
        let obj = schema_def.as_object().ok_or_else(|| {
            IngestionError::schema_parsing_error("Schema definition must be an object")
        })?;

        let name = obj
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| IngestionError::schema_parsing_error("Schema must have a name"))?;

        let mut schema = Schema::new(name.to_string());

        // Add fields if provided
        if let Some(Value::Object(fields)) = obj.get("fields") {
            for (field_name, field_def) in fields {
                // Create a basic single field for each field in the definition
                let field = self.create_basic_field_from_definition(field_def)?;
                schema.add_field(field_name.clone(), field);
            }
        }

        Ok(schema)
    }

    /// Create a basic field from definition
    fn create_basic_field_from_definition(
        &self,
        _field_def: &Value,
    ) -> IngestionResult<FieldVariant> {
        use crate::fees::types::FieldPaymentConfig;
        use crate::permissions::types::policy::PermissionsPolicy;
        use std::collections::HashMap;

        // Create a basic single field with default permissions and payment config
        let field = SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        );

        Ok(FieldVariant::Single(field))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::SchemaCore;
    use std::sync::Arc;
    use tempfile::TempDir;

    #[test]
    fn test_create_basic_schema_from_definition() {
        let temp_dir = TempDir::new().unwrap();
        let test_path = temp_dir
            .path()
            .join("test_schema_def")
            .to_string_lossy()
            .to_string();

        // Try to create components with better error handling
        let schema_core = match SchemaCore::new_for_testing(&test_path) {
            Ok(core) => Arc::new(core),
            Err(_) => {
                eprintln!("Skipping test_create_basic_schema_from_definition: Could not create schema core");
                return;
            }
        };

        let manager = SchemaManager::new(schema_core);

        let schema_def = serde_json::json!({
            "name": "TestSchema",
            "fields": {
                "field1": {"type": "string"},
                "field2": {"type": "number"}
            }
        });

        let result = manager.create_basic_schema_from_definition(&schema_def);
        assert!(result.is_ok());

        let schema = result.unwrap();
        assert_eq!(schema.name, "TestSchema");
        assert_eq!(schema.fields.len(), 2);
    }
}