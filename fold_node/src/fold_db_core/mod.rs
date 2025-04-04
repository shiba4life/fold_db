pub mod atom_manager;
pub mod field_manager;
pub mod collection_manager;
pub mod context;

use serde_json::Value;
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::SchemaCore;
use crate::schema::types::{Mutation, Query, MutationType};
use crate::schema::{Schema, SchemaError};
use crate::atom::{Atom, AtomRefBehavior};

use self::atom_manager::AtomManager;
use self::field_manager::FieldManager;
use self::collection_manager::CollectionManager;

/// The main database coordinator that manages schemas, permissions, and data storage.
pub struct FoldDB {
    pub(crate) atom_manager: AtomManager,
    pub(crate) field_manager: FieldManager,
    pub(crate) collection_manager: CollectionManager,
    pub(crate) schema_manager: SchemaCore,
    permission_wrapper: PermissionWrapper,
}

impl FoldDB {
    /// Creates a new FoldDB instance with the specified storage path.
    pub fn new(path: &str) -> sled::Result<Self> {
        let db = match sled::open(path) {
            Ok(db) => db,
            Err(e) => {
                if e.to_string().contains("No such file or directory") {
                    sled::open(path)?
                } else {
                    return Err(e);
                }
            }
        };

        let db_ops = DbOperations::new(db);
        let atom_manager = AtomManager::new(db_ops);
        let field_manager = FieldManager::new(atom_manager.clone());
        let collection_manager = CollectionManager::new(field_manager.clone());
        let schema_manager = SchemaCore::new(path);
        let _ = schema_manager.load_schemas_from_disk();

        Ok(Self {
            atom_manager,
            field_manager,
            collection_manager,
            schema_manager,
            permission_wrapper: PermissionWrapper::new(),
        })
    }

    pub fn load_schema(&mut self, schema: Schema) -> Result<(), SchemaError> {
        let name = schema.name.clone();
        self.schema_manager.load_schema(schema)?;
        
        // Get the atom refs that need to be persisted
        let atom_refs = self.schema_manager.map_fields(&name)?;
        
        // Persist each atom ref
        for atom_ref in atom_refs {
            let aref_uuid = atom_ref.uuid().to_string();
            let atom_uuid = atom_ref.get_atom_uuid().clone();
            
            // Store the atom ref in the database
            self.atom_manager.update_atom_ref(
                &aref_uuid,
                atom_uuid,
                "system".to_string(),
            ).map_err(|e| SchemaError::InvalidData(format!("Failed to persist atom ref: {}", e)))?;
        }
        
        Ok(())
    }

    pub fn allow_schema(&mut self, schema_name: &str) -> Result<(), SchemaError> {
        let exists = self.schema_manager.schema_exists(schema_name)?;
        if !exists {
            return Err(SchemaError::NotFound(format!(
                "Schema {} not found",
                schema_name
            )));
        }
        Ok(())
    }

    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        query.fields.iter().map(|field_name| {
            let perm_result = self.permission_wrapper.check_query_field_permission(
                &query,
                field_name,
                &self.schema_manager,
            );

            if !perm_result.allowed {
                return Err(perm_result.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }

            let schema = match self.schema_manager.get_schema(&query.schema_name) {
                Ok(Some(schema)) => schema,
                Ok(None) => return Err(SchemaError::NotFound(format!("Schema {} not found", query.schema_name))),
                Err(e) => return Err(e),
            };

            self.field_manager.get_field_value(&schema, field_name)
        }).collect()
    }

    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        if mutation.fields_and_values.is_empty() {
            return Err(SchemaError::InvalidData("No fields to write".to_string()));
        }

        let schema = self.schema_manager.get_schema(&mutation.schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name)))?;

        for (field_name, value) in mutation.fields_and_values.iter() {
            let perm_result = self.permission_wrapper.check_mutation_field_permission(
                &mutation,
                field_name,
                &self.schema_manager,
            );

            if !perm_result.allowed {
                return Err(perm_result.error.unwrap_or(SchemaError::InvalidPermission(
                    "Unknown permission error".to_string(),
                )));
            }

            match mutation.mutation_type {
                MutationType::Create => {
                    let mut schema_clone = schema.clone();
                    self.field_manager.set_field_value(
                        &mut schema_clone,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                    )?;
                }
                MutationType::Update => {
                    self.field_manager.update_field(
                        &schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                    )?;
                }
                MutationType::Delete => {
                    self.field_manager.delete_field(
                        &schema,
                        field_name,
                        mutation.pub_key.clone(),
                    )?;
                }
                MutationType::AddToCollection(ref id) => {
                    self.collection_manager.add_collection_field_value(
                        &schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::UpdateToCollection(ref id) => {
                    self.collection_manager.update_collection_field_value(
                        &schema,
                        field_name,
                        value.clone(),
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
                MutationType::DeleteFromCollection(ref id) => {
                    self.collection_manager.delete_collection_field_value(
                        &schema,
                        field_name,
                        mutation.pub_key.clone(),
                        id.clone(),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn get_atom_history(&self, aref_uuid: &str) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        self.atom_manager.get_atom_history(aref_uuid)
    }
}
