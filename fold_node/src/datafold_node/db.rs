use serde_json::Value;
use std::collections::HashMap;

use crate::error::{FoldDbError, FoldDbResult};
use crate::schema::types::{Fold, Mutation, Operation, Query, Transform};
use crate::schema::{Schema, SchemaError, SchemaValidator};

use super::DataFoldNode;

impl DataFoldNode {
    /// Loads a schema into the database and grants this node permission.
    pub fn load_schema(&mut self, schema: Schema) -> FoldDbResult<()> {
        let schema_name = schema.name.clone();
        self.with_db_mut(|db| {
            let mut schema = schema;
            for (fname, field) in schema.fields.iter_mut() {
                if let Some(transform) = field.transform.as_mut() {
                    if transform.get_output().starts_with("test.") {
                        transform.set_output(format!("{}.{}", schema_name, fname));
                    }
                }
            }

            let validator = SchemaValidator::new(&db.schema_manager);
            validator.validate(&schema)?;
            db.load_schema(schema)?;
            Ok(())
        })?;
        self.grant_schema_permission(&schema_name)?;
        Ok(())
    }

    /// Executes an operation (query or mutation) on the database.
    pub fn execute_operation(&mut self, operation: Operation) -> FoldDbResult<Value> {
        match operation {
            Operation::Query { schema, fields, filter: _ } => {
                let query = Query {
                    schema_name: schema,
                    fields,
                    pub_key: String::new(),
                    trust_distance: 0,
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
        self.with_db(|db| Ok(db.schema_manager.get_schema(schema_id)?))
    }

    /// Lists all loaded schemas in the database.
    pub fn list_schemas(&self) -> FoldDbResult<Vec<Schema>> {
        self.with_db(|db| {
            let schema_names = db.schema_manager.list_loaded_schemas()?;
            let mut schemas = Vec::new();
            for name in schema_names {
                if let Some(schema) = db.schema_manager.get_schema(&name)? {
                    schemas.push(schema);
                }
            }
            Ok(schemas)
        })
    }

    /// List the names of all schemas available on disk.
    pub fn list_available_schemas(&self) -> FoldDbResult<Vec<String>> {
        self.with_db(|db| Ok(db.schema_manager.list_available_schemas()?))
    }

    /// Executes a query against the database.
    pub fn query(&self, mut query: Query) -> FoldDbResult<Vec<Result<Value, SchemaError>>> {
        if !self.check_schema_permission(&query.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                query.schema_name
            )));
        }
        if query.trust_distance == 0 {
            query.trust_distance = self.config.default_trust_distance;
        }
        self.with_db(|db| Ok(db.query_schema(query)))
    }

    /// Executes a mutation on the database.
    pub fn mutate(&mut self, mutation: Mutation) -> FoldDbResult<()> {
        if !self.check_schema_permission(&mutation.schema_name)? {
            return Err(FoldDbError::Config(format!(
                "Permission denied for schema {}",
                mutation.schema_name
            )));
        }
        self.with_db_mut(|db| {
            db.write_schema(mutation)?;
            Ok(())
        })
    }

    /// Retrieves the version history for a specific atom reference.
    pub fn get_history(&self, aref_uuid: &str) -> FoldDbResult<Vec<Value>> {
        self.with_db(|db| {
            let history = db
                .atom_manager
                .get_atom_history(aref_uuid)
                .map_err(|e| FoldDbError::Database(e.to_string()))?;
            Ok(history.into_iter().map(|a| a.content().clone()).collect())
        })
    }

    /// Mark a schema as unloaded without removing it from disk.
    pub fn unload_schema(&mut self, schema_name: &str) -> FoldDbResult<()> {
        self.with_db(|db| db.unload_schema(schema_name).map_err(|e| e.into()))
    }

    /// Load a fold into the database.
    pub fn load_fold(&self, fold: Fold) -> FoldDbResult<()> {
        self.with_db(|db| db.load_fold(fold).map_err(|e| e.into()))
    }

    /// Get a fold by name.
    pub fn get_fold(&self, name: &str) -> FoldDbResult<Option<Fold>> {
        self.with_db(|db| Ok(db.get_fold(name)?))
    }

    /// List all loaded folds.
    pub fn list_folds(&self) -> FoldDbResult<Vec<String>> {
        self.with_db(|db| Ok(db.list_folds()?))
    }

    /// List all loaded folds.
    pub fn list_loaded_folds(&self) -> FoldDbResult<Vec<String>> {
        self.with_db(|db| Ok(db.list_loaded_folds()?))
    }

    /// List all folds available on disk.
    pub fn list_available_folds(&self) -> FoldDbResult<Vec<String>> {
        self.with_db(|db| Ok(db.list_available_folds()?))
    }

    /// Unload a fold from memory.
    pub fn unload_fold(&self, name: &str) -> FoldDbResult<()> {
        self.with_db(|db| db.unload_fold(name).map_err(|e| e.into()))
    }


    /// List all registered transforms.
    pub fn list_transforms(&self) -> FoldDbResult<HashMap<String, Transform>> {
        self.with_db(|db| Ok(db.list_transforms()?))
    }

    /// Execute a transform by id and return the result.
    pub fn run_transform(&mut self, transform_id: &str) -> FoldDbResult<Value> {
        self.with_db(|db| Ok(db.run_transform(transform_id)?))
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
