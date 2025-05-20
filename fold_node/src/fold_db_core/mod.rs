pub mod atom_manager;
pub mod collection_manager;
pub mod context;
pub mod field_manager;
pub mod transform_manager;
pub mod transform_orchestrator;

use std::sync::Arc;
use std::collections::HashMap;
use crate::atom::{Atom, AtomRefBehavior};
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::types::{Mutation, MutationType, Query, Transform, TransformRegistration};
use crate::schema::SchemaCore;
use crate::schema::{Schema, SchemaError};
use serde_json;
use serde_json::Value;
use uuid::Uuid;
use regex::Regex;

use self::atom_manager::AtomManager;
use self::collection_manager::CollectionManager;
use self::field_manager::FieldManager;
use self::transform_manager::TransformManager;
use self::transform_orchestrator::TransformOrchestrator;

/// The main database coordinator that manages schemas, permissions, and data storage.
pub struct FoldDB {
    pub(crate) atom_manager: AtomManager,
    pub(crate) field_manager: FieldManager,
    pub(crate) collection_manager: CollectionManager,
    pub(crate) schema_manager: Arc<SchemaCore>,
    pub(crate) transform_manager: Arc<TransformManager>,
    pub(crate) transform_orchestrator: Arc<TransformOrchestrator>,
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
        let schema_manager = Arc::new(
            SchemaCore::new(path)
                .map_err(|e| sled::Error::Unsupported(e.to_string()))?,
        );
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

        let field_value_manager = FieldManager::new(atom_manager.clone());
        let schema_manager_clone = Arc::clone(&schema_manager);
        let get_field_fn = Arc::new(move |schema_name: &str, field_name: &str| {
            match schema_manager_clone.get_schema(schema_name)? {
                Some(schema) => field_value_manager.get_field_value(&schema, field_name),
                None => Err(SchemaError::InvalidField(format!("Field not found: {}.{}", schema_name, field_name))),
            }
        });

        let transform_manager = Arc::new(TransformManager::new(
            transforms_tree,
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
            get_field_fn,
        ));

        field_manager
            .set_transform_manager(Arc::clone(&transform_manager))
            .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
        let orchestrator = Arc::new(TransformOrchestrator::new(transform_manager.clone()));
        field_manager
            .set_orchestrator(Arc::clone(&orchestrator))
            .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
        let _ = schema_manager.load_schemas_from_disk();

        Ok(Self {
            atom_manager,
            field_manager,
            collection_manager,
            schema_manager,
            transform_manager,
            transform_orchestrator: orchestrator,
            permission_wrapper: PermissionWrapper::new(),
            metadata_tree,
            permissions_tree,
        })
    }

    /// Registers a transform with its input and output atom references
    pub fn register_transform(&mut self, registration: TransformRegistration) -> Result<(), SchemaError> {
        self.transform_manager.register_transform(registration)
    }

    fn parse_output_field(
        &self,
        schema: &Schema,
        field_name: &str,
        field: &crate::schema::types::SchemaField,
        transform: &Transform,
    ) -> Result<String, SchemaError> {
        let (out_schema_name, out_field_name) = match transform.get_output().split_once('.') {
            Some((s, f)) => (s.to_string(), f.to_string()),
            None => (schema.name.clone(), field_name.to_string()),
        };

        if out_schema_name == schema.name && out_field_name == field_name {
            field.get_ref_atom_uuid().ok_or_else(|| {
                SchemaError::InvalidData(format!("Field {} missing atom reference", field_name))
            })
        } else {
            match self
                .schema_manager
                .get_schema(&out_schema_name)?
                .and_then(|s| s.fields.get(&out_field_name).cloned())
            {
                Some(of) => of.get_ref_atom_uuid().ok_or_else(|| {
                    SchemaError::InvalidData(format!(
                        "Field {}.{} missing atom reference",
                        out_schema_name, out_field_name
                    ))
                }),
                None => field.get_ref_atom_uuid().ok_or_else(|| {
                    SchemaError::InvalidData(format!("Field {} missing atom reference", field_name))
                }),
            }
        }
    }

    fn collect_input_arefs(
        &self,
        schema: &Schema,
        transform: &Transform,
        cross_re: &Regex,
    ) -> Result<(Vec<String>, Vec<String>), SchemaError> {
        let mut input_arefs = Vec::new();
        let mut trigger_fields = Vec::new();
        let mut seen_cross = std::collections::HashSet::new();

        let inputs = transform.get_inputs();
        if !inputs.is_empty() {
            for input in inputs {
                if let Some((schema_name, field_dep)) = input.split_once('.') {
                    seen_cross.insert(field_dep.to_string());
                    trigger_fields.push(format!("{}.{}", schema_name, field_dep));
                    if let Some(dep_schema) = self.schema_manager.get_schema(schema_name)? {
                        if let Some(dep_field) = dep_schema.fields.get(field_dep) {
                            if let Some(dep_aref) = dep_field.get_ref_atom_uuid() {
                                input_arefs.push(dep_aref);
                            }
                        }
                    }
                }
            }
        } else {
            for cap in cross_re.captures_iter(&transform.logic) {
                let schema_name = cap[1].to_string();
                let field_dep = cap[2].to_string();
                seen_cross.insert(field_dep.clone());
                trigger_fields.push(format!("{}.{}", schema_name, field_dep));
                if let Some(dep_schema) = self.schema_manager.get_schema(&schema_name)? {
                    if let Some(dep_field) = dep_schema.fields.get(&field_dep) {
                        if let Some(dep_aref) = dep_field.get_ref_atom_uuid() {
                            input_arefs.push(dep_aref);
                        }
                    }
                }
            }
        }

        for dep in transform.analyze_dependencies() {
            if seen_cross.contains(&dep) {
                continue;
            }
            let schema_name = schema.name.clone();
            let field_dep = dep;

            trigger_fields.push(format!("{}.{}", schema_name, field_dep));

            if let Some(dep_schema) = self.schema_manager.get_schema(&schema_name)? {
                if let Some(dep_field) = dep_schema.fields.get(&field_dep) {
                    if let Some(dep_aref) = dep_field.get_ref_atom_uuid() {
                        input_arefs.push(dep_aref);
                    }
                }
            }
        }

        Ok((input_arefs, trigger_fields))
    }

    fn register_transform_internal(
        &self,
        schema: &Schema,
        field_name: &str,
        transform: &Transform,
        input_arefs: Vec<String>,
        mut trigger_fields: Vec<String>,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        let transform_id = format!("{}.{}", schema.name, field_name);
        trigger_fields.push(transform_id.clone());
        let registration = TransformRegistration {
            transform_id: transform_id.clone(),
            transform: transform.clone(),
            input_arefs,
            trigger_fields,
            output_aref,
            schema_name: schema.name.clone(),
            field_name: field_name.to_string(),
        };
        self.transform_manager.register_transform(registration)?;
        let _ = self.transform_manager.execute_transform_now(&transform_id);
        Ok(())
    }

    fn register_transforms_for_schema(&self, schema: &Schema) -> Result<(), SchemaError> {
        let cross_re = Regex::new(r"([A-Za-z0-9_]+)\.([A-Za-z0-9_]+)").unwrap();

        for (field_name, field) in &schema.fields {
            if let Some(transform) = field.get_transform() {
                let output_aref = self.parse_output_field(schema, field_name, field, transform)?;
                let (input_arefs, trigger_fields) =
                    self.collect_input_arefs(schema, transform, &cross_re)?;
                self.register_transform_internal(
                    schema,
                    field_name,
                    transform,
                    input_arefs,
                    trigger_fields,
                    output_aref,
                )?;
            }
        }

        Ok(())
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

        if let Some(loaded_schema) = self.schema_manager.get_schema(&name)? {
            self.register_transforms_for_schema(&loaded_schema)?;
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

            let _ = self.transform_orchestrator.add_task(&schema.name, field_name);
        }
        Ok(())
    }

    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        self.atom_manager.get_atom_history(aref_uuid)
    }

    /// Returns the number of queued transform tasks.
    pub fn orchestrator_len(&self) -> Result<usize, SchemaError> {
        self.transform_orchestrator.len()
    }

    /// List all registered transforms.
    pub fn list_transforms(&self) -> Result<HashMap<String, Transform>, SchemaError> {
        self.transform_manager.list_transforms()
    }

    /// Execute a transform immediately and return the result.
    pub fn run_transform(&self, transform_id: &str) -> Result<Value, SchemaError> {
        self.transform_manager.execute_transform_now(transform_id)
    }

    /// Unload a schema and remove its associated transforms.
    pub fn remove_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        let schema = match self.schema_manager.get_schema(schema_name)? {
            Some(s) => s,
            None => {
                return Err(SchemaError::NotFound(format!(
                    "Schema {} not found",
                    schema_name
                )))
            }
        };

        for field_name in schema.fields.keys() {
            let transform_id = format!("{}.{}", schema.name, field_name);
            let _ = self.transform_manager.unregister_transform(&transform_id)?;
        }

        match self.schema_manager.unload_schema(schema_name)? {
            true => Ok(()),
            false => Err(SchemaError::NotFound(format!(
                "Schema {} not found",
                schema_name
            ))),
        }
    }
}
