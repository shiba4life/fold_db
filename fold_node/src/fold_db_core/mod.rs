pub mod atom_manager;
pub mod collection_manager;
pub mod context;
pub mod field_manager;
pub mod transform_manager;

use std::sync::Arc;
use crate::atom::{Atom, AtomRefBehavior};
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::types::{Mutation, MutationType, Query, Transform};
use crate::schema::SchemaCore;
use crate::schema::{Schema, SchemaError};
use serde_json;
use serde_json::Value;
use uuid::Uuid;

use self::atom_manager::AtomManager;
use self::collection_manager::CollectionManager;
use self::field_manager::FieldManager;
use self::transform_manager::TransformManager;

/// The main database coordinator that manages schemas, permissions, and data storage.
pub struct FoldDB {
    pub(crate) atom_manager: AtomManager,
    pub(crate) field_manager: FieldManager,
    pub(crate) collection_manager: CollectionManager,
    pub(crate) schema_manager: SchemaCore,
    pub(crate) transform_manager: Arc<TransformManager>,
    permission_wrapper: PermissionWrapper,
    /// Tree for storing metadata such as node_id
    metadata_tree: sled::Tree,
    /// Tree for storing per-node schema permissions
    permissions_tree: sled::Tree,
}

impl FoldDB {
    /// Retrieves or generates and persists the node identifier.
    pub fn get_node_id(&self) -> Result<String, sled::Error> {
        if let Some(bytes) = self.metadata_tree.get("node_id")? {
            let id = String::from_utf8(bytes.to_vec()).unwrap_or_else(|_| String::new());
            if !id.is_empty() {
                return Ok(id);
            }
        }
        let new_id = Uuid::new_v4().to_string();
        self.metadata_tree.insert("node_id", new_id.as_bytes())?;
        self.metadata_tree.flush()?;
        Ok(new_id)
    }

    /// Retrieves the list of permitted schemas for the given node.
    pub fn get_schema_permissions(&self, node_id: &str) -> Vec<String> {
        if let Ok(Some(bytes)) = self.permissions_tree.get(node_id) {
            if let Ok(list) = serde_json::from_slice::<Vec<String>>(&bytes) {
                return list;
            }
        }
        Vec::new()
    }

    /// Sets the permitted schemas for the given node.
    pub fn set_schema_permissions(&self, node_id: &str, schemas: &[String]) -> sled::Result<()> {
        let bytes = serde_json::to_vec(schemas).unwrap();
        self.permissions_tree.insert(node_id, bytes)?;
        self.permissions_tree.flush()?;
        Ok(())
    }
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

        let metadata_tree = db.open_tree("metadata")?;
        let permissions_tree = db.open_tree("node_id_schema_permissions")?;
        let transforms_tree = db.open_tree("transforms")?;

        let db_ops = DbOperations::new(db.clone());
        let atom_manager = AtomManager::new(db_ops);
        let field_manager = FieldManager::new(atom_manager.clone());
        let collection_manager = CollectionManager::new(field_manager.clone());
        let schema_manager = SchemaCore::new(path);
        let atom_manager_clone = atom_manager.clone();
        let get_atom_fn = Arc::new(move |aref_uuid: &str| {
            atom_manager_clone.get_latest_atom(aref_uuid)
        });

        let atom_manager_clone = atom_manager.clone();
        let create_atom_fn = Arc::new(move |schema_name: &str,
                                           source_pub_key: String,
                                           prev_atom_uuid: Option<String>,
                                           content: Value,
                                           status: Option<crate::atom::AtomStatus>| {
            atom_manager_clone.create_atom(schema_name, source_pub_key, prev_atom_uuid, content, status)
        });

        let atom_manager_clone = atom_manager.clone();
        let update_atom_ref_fn = Arc::new(move |aref_uuid: &str, atom_uuid: String, source_pub_key: String| {
            atom_manager_clone.update_atom_ref(aref_uuid, atom_uuid, source_pub_key)
        });

        let transform_manager = Arc::new(TransformManager::new(
            transforms_tree,
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
        ));

        field_manager.set_transform_manager(Arc::clone(&transform_manager));
        let _ = schema_manager.load_schemas_from_disk();

        Ok(Self {
            atom_manager,
            field_manager,
            collection_manager,
            schema_manager,
            transform_manager,
            permission_wrapper: PermissionWrapper::new(),
            metadata_tree,
            permissions_tree,
        })
    }

    /// Registers a transform with its input and output atom references
    pub fn register_transform(
        &mut self,
        transform_id: String,
        transform: Transform,
        input_arefs: Vec<String>,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        self.transform_manager.register_transform(transform_id, transform, input_arefs, output_aref)
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
            self.atom_manager
                .update_atom_ref(&aref_uuid, atom_uuid, "system".to_string())
                .map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to persist atom ref: {}", e))
                })?;
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
        query
            .fields
            .iter()
            .map(|field_name| {
                // Trust distance 0 bypasses permission checks
                let perm_allowed = if query.trust_distance == 0 {
                    true
                } else {
                    let perm = self.permission_wrapper.check_query_field_permission(
                        &query,
                        field_name,
                        &self.schema_manager,
                    );
                    perm.allowed
                };

                if !perm_allowed {
                    let err = self
                        .permission_wrapper
                        .check_query_field_permission(&query, field_name, &self.schema_manager)
                        .error
                        .unwrap_or(SchemaError::InvalidPermission(
                            "Unknown permission error".to_string(),
                        ));
                    return Err(err);
                }

                let schema = match self.schema_manager.get_schema(&query.schema_name) {
                    Ok(Some(schema)) => schema,
                    Ok(None) => {
                        return Err(SchemaError::NotFound(format!(
                            "Schema {} not found",
                            query.schema_name
                        )))
                    }
                    Err(e) => return Err(e),
                };

                self.field_manager.get_field_value(&schema, field_name)
            })
            .collect()
    }

    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        if mutation.fields_and_values.is_empty() {
            return Err(SchemaError::InvalidData("No fields to write".to_string()));
        }

        let schema = self
            .schema_manager
            .get_schema(&mutation.schema_name)?
            .ok_or_else(|| {
                SchemaError::NotFound(format!("Schema {} not found", mutation.schema_name))
            })?;

        for (field_name, value) in mutation.fields_and_values.iter() {
            let perm = self.permission_wrapper.check_mutation_field_permission(
                &mutation,
                field_name,
                &self.schema_manager,
            );
            // Bypass permission checks when trust distance is zero
            if mutation.trust_distance != 0 && !perm.allowed {
                return Err(perm.error.unwrap_or(SchemaError::InvalidPermission(
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

    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        self.atom_manager.get_atom_history(aref_uuid)
    }
}
