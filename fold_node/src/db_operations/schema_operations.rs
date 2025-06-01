use crate::schema::SchemaError;
use crate::schema::core::SchemaState;
use crate::schema::Schema;
use super::core::DbOperations;

impl DbOperations {
    /// Stores a schema state using generic tree operations
    pub fn store_schema_state(&self, schema_name: &str, state: SchemaState) -> Result<(), SchemaError> {
        self.store_in_tree(&self.schema_states_tree, schema_name, &state)
    }

    /// Gets a schema state using generic tree operations
    pub fn get_schema_state(&self, schema_name: &str) -> Result<Option<SchemaState>, SchemaError> {
        self.get_from_tree(&self.schema_states_tree, schema_name)
    }

    /// Lists all schemas with a specific state
    pub fn list_schemas_by_state(&self, target_state: SchemaState) -> Result<Vec<String>, SchemaError> {
        let all_states: Vec<(String, SchemaState)> = self.list_items_in_tree(&self.schema_states_tree)?;
        Ok(all_states
            .into_iter()
            .filter(|(_, state)| *state == target_state)
            .map(|(name, _)| name)
            .collect())
    }

    /// Stores a schema definition using generic tree operations
    pub fn store_schema(&self, schema_name: &str, schema: &Schema) -> Result<(), SchemaError> {
        self.store_in_tree(&self.schemas_tree, schema_name, schema)
    }

    /// Gets a schema definition using generic tree operations
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<Schema>, SchemaError> {
        self.get_from_tree(&self.schemas_tree, schema_name)
    }

    /// Lists all stored schemas using generic tree operations
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.list_keys_in_tree(&self.schemas_tree)
    }

    /// Deletes a schema definition
    pub fn delete_schema(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.delete_from_tree(&self.schemas_tree, schema_name)
    }

    /// Deletes a schema state
    pub fn delete_schema_state(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.delete_from_tree(&self.schema_states_tree, schema_name)
    }

    // NOTE: add_schema_to_available_directory has been removed to eliminate duplication.
    // Use SchemaCore::add_schema_to_available_directory instead, which provides:
    // - Comprehensive validation
    // - Hash-based de-duplication
    // - Conflict resolution
    // - Proper integration with the schema system

    /// Checks if a schema exists
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.exists_in_tree(&self.schemas_tree, schema_name)
    }

    /// Checks if a schema state exists
    pub fn schema_state_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.exists_in_tree(&self.schema_states_tree, schema_name)
    }

    /// Gets all schema states as a HashMap
    pub fn get_all_schema_states(&self) -> Result<std::collections::HashMap<String, SchemaState>, SchemaError> {
        let items: Vec<(String, SchemaState)> = self.list_items_in_tree(&self.schema_states_tree)?;
        Ok(items.into_iter().collect())
    }
}