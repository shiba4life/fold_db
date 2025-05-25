use serde_json::Value;
use std::collections::HashMap;
use serde::Serialize;

use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::types::{Field, Mutation, Operation, Query, Transform};
use crate::schema::{Schema, SchemaError, SchemaValidator};
use crate::schema::core::SchemaState;

use super::DataFoldNode;

#[derive(Clone, Serialize)]
pub struct SchemaWithState {
    #[serde(flatten)]
    pub schema: Schema,
    pub state: SchemaState,
}

impl DataFoldNode {
    /// Ensure a schema is loaded into memory if its state is `Loaded` on disk.
    fn ensure_schema_loaded(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        if db.schema_manager.get_schema(schema_name)?.is_some() {
            return Ok(());
        }
        if matches!(db.schema_manager.get_schema_state(schema_name), Some(SchemaState::Loaded)) {
            let path = self
                .config
                .storage_path
                .join("schemas")
                .join(format!("{}.json", schema_name));
            db.schema_manager
                .load_schema_from_file(path.to_str().ok_or_else(|| FoldDbError::Config("Invalid schema path".into()))?)
                .map_err(|e| FoldDbError::Database(e.to_string()))?;
        }
        Ok(())
    }
    /// Loads a schema into the database and grants this node permission.
    pub fn load_schema(&mut self, schema: Schema) -> FoldDbResult<()> {
        let schema_name = schema.name.clone();
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;

        // Apply transform output fix and validate before loading
        let mut schema = schema;
        for (fname, field) in schema.fields.iter_mut() {
            if let Some(transform) = field.transform() {
                let mut transform = transform.clone();
                if transform.get_output().starts_with("test.") {
                    transform.set_output(format!("{}.{}", schema_name, fname));
                    field.set_transform(transform);
                }
            }
        }

        let validator = SchemaValidator::new(&db.schema_manager);
        validator.validate(&schema)?;
        db.load_schema(schema)?;
        drop(db);
        self.grant_schema_permission(&schema_name)?;
        Ok(())
    }

    /// Executes an operation (query or mutation) on the database.
    pub fn execute_operation(&mut self, operation: Operation) -> FoldDbResult<Value> {
        match operation {
            Operation::Query { schema, fields, filter } => {
                let query = Query {
                    schema_name: schema,
                    fields,
                    pub_key: String::new(),
                    trust_distance: 0,
                    filter,
                };
                let results = self.query(query)?;
                let unwrapped: Vec<Value> = results
                    .into_iter()
                    .map(|r| r.unwrap_or_else(|e| serde_json::json!({"error": e.to_string()})))
                    .collect();
                Ok(serde_json::to_value(&unwrapped)?)
            }
            Operation::Mutation { schema, data, mutation_type } => {
                let fields_and_values = match data {
                    Value::Object(map) => map.into_iter().collect(),
                    _ => {
                        return Err(FoldDbError::Config(
                            "Mutation data must be an object".into(),
                        ))
                    }
                };

                let mutation = Mutation {
                    schema_name: schema,
                    fields_and_values,
                    pub_key: String::new(),
                    trust_distance: 0,
                    mutation_type,
                };
                self.mutate(mutation)?;
                Ok(Value::Null)
            }
        }
    }

    /// Retrieves a schema by its ID.
    pub fn get_schema(&self, schema_id: &str) -> FoldDbResult<Option<Schema>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.schema_manager.get_schema(schema_id)?)
    }

    /// Lists all loaded schemas in the database.
    pub fn list_schemas(&self) -> FoldDbResult<Vec<Schema>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let schema_names = db.schema_manager.list_loaded_schemas()?;
        let mut schemas = Vec::new();
        for name in schema_names {
            if let Some(schema) = db.schema_manager.get_schema(&name)? {
                schemas.push(schema);
            }
        }
        Ok(schemas)
    }

    /// Lists all loaded schemas along with their load state.
    pub fn list_schemas_with_state(&self) -> FoldDbResult<Vec<SchemaWithState>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let schema_names = db.schema_manager.list_loaded_schemas()?;
        let mut schemas = Vec::new();
        for name in schema_names {
            if let Some(schema) = db.schema_manager.get_schema(&name)? {
                let state = db
                    .schema_manager
                    .get_schema_state(&name)
                    .unwrap_or(SchemaState::Loaded);
                schemas.push(SchemaWithState { schema, state });
            }
        }
        Ok(schemas)
    }

    /// List the names of all schemas available on disk.
    pub fn list_available_schemas(&self) -> FoldDbResult<Vec<String>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.schema_manager.list_available_schemas()?)
    }

    /// Executes a query against the database.
    pub fn query(&mut self, mut query: Query) -> FoldDbResult<Vec<Result<Value, SchemaError>>> {
        if !self.check_schema_permission(&query.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                query.schema_name
            )));
        }
        self.ensure_schema_loaded(&query.schema_name)?;
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.query_schema(query))
    }

    /// Executes a mutation on the database.
    pub fn mutate(&mut self, mutation: Mutation) -> FoldDbResult<()> {
        if !self.check_schema_permission(&mutation.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                mutation.schema_name
            )));
        }
        self.ensure_schema_loaded(&mutation.schema_name)?;
        let mut db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.write_schema(mutation)?;
        Ok(())
    }

    /// Retrieves the version history for a specific atom reference.
    pub fn get_history(&self, aref_uuid: &str) -> FoldDbResult<Vec<Value>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        let history = db
            .atom_manager
            .get_atom_history(aref_uuid)
            .map_err(|e| FoldDbError::Database(e.to_string()))?;
        Ok(history.into_iter().map(|a| a.content().clone()).collect())
    }

    /// Mark a schema as unloaded without removing it from disk.
    pub fn unload_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        db.unload_schema(schema_name).map_err(|e| e.into())
    }


    /// List all registered transforms.
    pub fn list_transforms(&self) -> FoldDbResult<HashMap<String, Transform>> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.list_transforms()?)
    }

    /// Execute a transform by id and return the result.
    pub fn run_transform(&mut self, transform_id: &str) -> FoldDbResult<Value> {
        let db = self
            .db
            .lock()
            .map_err(|_| FoldDbError::Config("Cannot lock database mutex".into()))?;
        Ok(db.run_transform(transform_id)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::datafold_node::config::NodeConfig;
    use crate::schema::Schema;

    fn create_node(path: &std::path::Path) -> DataFoldNode {
        let config = NodeConfig {
            storage_path: path.into(),
            default_trust_distance: 1,
            network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
        };
        DataFoldNode::new(config).unwrap()
    }

    #[test]
    fn load_and_list_schema() {
        let dir = tempdir().unwrap();
        let mut node = create_node(dir.path());
        let schema = Schema::new("Test".to_string());
        node.load_schema(schema).unwrap();
        let schemas = node.list_schemas().unwrap();
        assert_eq!(schemas.len(), 1);
    }

    #[test]
    fn load_schema_invalid_fails() {
        let dir = tempdir().unwrap();
        let mut node = create_node(dir.path());

        let mut schema = Schema::new("Bad".to_string());
        schema.payment_config.base_multiplier = 0.0;

        let res = node.load_schema(schema);
        assert!(res.is_err());
    }

}
