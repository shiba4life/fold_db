pub mod atom_manager;
pub mod collection_manager;
pub mod context;
pub mod field_manager;
pub mod field_retrieval;
pub mod transform_manager;
pub mod transform_orchestrator;
mod query;
mod mutation;
mod transform_management;
mod init;

use std::sync::Arc;
use std::path::Path;
use std::collections::HashMap;
use log::info;
use crate::atom::{Atom, AtomRefBehavior};
use crate::db_operations::DbOperations;
use crate::permissions::PermissionWrapper;
use crate::schema::SchemaCore;
use crate::schema::{Schema, SchemaError};
use crate::schema::core::SchemaState;

use self::atom_manager::AtomManager;
use self::collection_manager::CollectionManager;
use self::field_manager::FieldManager;
use self::transform_manager::TransformManager;
use self::transform_orchestrator::TransformOrchestrator;
use self::init::{build_closure_fns, init_transform_manager, init_orchestrator};

/// The main database coordinator that manages schemas, permissions, and data storage.
pub struct FoldDB {
    pub(crate) atom_manager: AtomManager,
    pub(crate) field_manager: FieldManager,
    pub(crate) collection_manager: CollectionManager,
    pub(crate) schema_manager: Arc<SchemaCore>,
    pub(crate) transform_manager: Arc<TransformManager>,
    pub(crate) transform_orchestrator: Arc<TransformOrchestrator>,
    /// Shared database operations
    pub(crate) db_ops: Arc<DbOperations>,
    permission_wrapper: PermissionWrapper,
}

impl FoldDB {
    /// Retrieves or generates and persists the node identifier.
    pub fn get_node_id(&self) -> Result<String, sled::Error> {
        self
            .db_ops
            .get_node_id()
            .map_err(|e| sled::Error::Unsupported(e.to_string()))
    }

    /// Retrieves the list of permitted schemas for the given node.
    pub fn get_schema_permissions(&self, node_id: &str) -> Vec<String> {
        self
            .db_ops
            .get_schema_permissions(node_id)
            .unwrap_or_default()
    }

    /// Sets the permitted schemas for the given node.
    pub fn set_schema_permissions(&self, node_id: &str, schemas: &[String]) -> sled::Result<()> {
        self
            .db_ops
            .set_schema_permissions(node_id, schemas)
            .map_err(|e| sled::Error::Unsupported(e.to_string()))
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

        let db_ops = DbOperations::new(db.clone())
            .map_err(|e| sled::Error::Unsupported(e.to_string()))?;
        let orchestrator_tree = db_ops.orchestrator_tree.clone();
        
        let atom_manager = AtomManager::new(db_ops.clone());
        let field_manager = FieldManager::new(atom_manager.clone());
        let collection_manager = CollectionManager::new(field_manager.clone());
        let schema_manager = Arc::new(
            SchemaCore::new(path, Arc::new(db_ops.clone()))
                .map_err(|e| sled::Error::Unsupported(e.to_string()))?,
        );
        let closures = build_closure_fns(&atom_manager, &schema_manager);
        let transform_manager = init_transform_manager(Arc::new(db_ops.clone()), closures)?;
        let orchestrator = init_orchestrator(&field_manager, transform_manager.clone(), orchestrator_tree)?;

        info!("Loading schema states from disk during FoldDB initialization");
        if let Err(e) = schema_manager.load_schema_states_from_disk() {
            info!("Failed to load schema states: {}", e);
        } else {
            // After loading schema states, we need to ensure AtomRefs are properly mapped and persisted
            // for all approved schemas
            if let Ok(approved_schemas) = schema_manager.list_schemas_by_state(SchemaState::Approved) {
                for schema_name in approved_schemas {
                    if let Ok(atom_refs) = schema_manager.map_fields(&schema_name) {
                        // Persist each atom ref
                        for atom_ref in atom_refs {
                            let aref_uuid = atom_ref.uuid().to_string();
                            let atom_uuid = atom_ref.get_atom_uuid().clone();
                            
                            // Store the atom ref in the database
                            if let Err(e) = atom_manager.update_atom_ref(&aref_uuid, atom_uuid, "system".to_string()) {
                                info!("Failed to persist atom ref for schema '{}': {}", schema_name, e);
                            }
                        }
                    }
                }
            }
        }

        Ok(Self {
            atom_manager,
            field_manager,
            collection_manager,
            schema_manager,
            transform_manager,
            transform_orchestrator: orchestrator,
            db_ops: Arc::new(db_ops.clone()),
            permission_wrapper: PermissionWrapper::new(),
        })
    }


    // ========== CONSOLIDATED SCHEMA API - DELEGATES TO SCHEMA_CORE ==========
    
    /// Fetch available schemas from files (example schemas directory)
    /// DEPRECATED: Use get_schema_status() instead
    pub fn fetch_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.schema_manager.fetch_available_schemas()
    }

    /// Get comprehensive schema status (NEW UNIFIED METHOD)
    pub fn get_schema_status(&self) -> Result<crate::schema::core::SchemaLoadingReport, SchemaError> {
        self.schema_manager.get_schema_status()
    }

    /// Refresh schemas from all sources (NEW UNIFIED METHOD)
    pub fn refresh_schemas(&self) -> Result<crate::schema::core::SchemaLoadingReport, SchemaError> {
        self.schema_manager.discover_and_load_all_schemas()
    }

    /// Approve a schema for queries and mutations
    pub fn approve_schema(&mut self, schema_name: &str) -> Result<(), SchemaError> {
        self.schema_manager.approve_schema(schema_name)?;

        // Get the atom refs that need to be persisted
        let atom_refs = self.schema_manager.map_fields(schema_name)?;

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

        // Get the updated schema with proper ARefs and register transforms
        if let Some(loaded_schema) = self.schema_manager.get_schema(schema_name)? {
            info!("Registering transforms for approved schema '{}' with {} fields", schema_name, loaded_schema.fields.len());
            self.register_transforms_for_schema(&loaded_schema)?;
        }

        Ok(())
    }

    /// Block a schema from queries and mutations
    pub fn block_schema(&mut self, schema_name: &str) -> Result<(), SchemaError> {
        self.schema_manager.block_schema(schema_name)
    }

    /// Load schema state from sled
    pub fn load_schema_state(&self) -> Result<HashMap<String, SchemaState>, SchemaError> {
        self.schema_manager.load_schema_state()
    }

    /// Load available schemas from sled and files
    /// DEPRECATED: Use initialize_schema_system() instead
    pub fn load_available_schemas(&self) -> Result<(), SchemaError> {
        self.schema_manager.load_available_schemas()
    }

    /// Initialize schema system (NEW UNIFIED METHOD)
    pub fn initialize_schema_system(&self) -> Result<(), SchemaError> {
        self.schema_manager.initialize_schema_system()
    }

    /// Load schema from JSON string (creates Available schema)
    pub fn load_schema_from_json(&mut self, json_str: &str) -> Result<(), SchemaError> {
        self.schema_manager.load_schema_from_json(json_str)
    }

    /// Load schema from file (creates Available schema)
    pub fn load_schema_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), SchemaError> {
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| SchemaError::InvalidData("Invalid file path".to_string()))?;
        self.schema_manager.load_schema_from_file(path_str)
    }

    /// Check if a schema can be queried (must be Approved)
    pub fn can_query_schema(&self, schema_name: &str) -> bool {
        self.schema_manager.can_query_schema(schema_name)
    }

    /// Check if a schema can be mutated (must be Approved)
    pub fn can_mutate_schema(&self, schema_name: &str) -> bool {
        self.schema_manager.can_mutate_schema(schema_name)
    }

    /// Get schemas by state
    pub fn list_schemas_by_state(&self, state: SchemaState) -> Result<Vec<String>, SchemaError> {
        self.schema_manager.list_schemas_by_state(state)
    }

    /// Get all available schemas (any state)
    pub fn list_all_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.schema_manager.list_all_schemas()
    }

    /// Legacy method - now creates Available schema
    pub fn add_schema_available(&mut self, schema: Schema) -> Result<(), SchemaError> {
        self.schema_manager.add_schema_available(schema)
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


    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        self.atom_manager.get_atom_history(aref_uuid)
    }

    /// Mark a schema as unloaded without removing transforms.
    pub fn unload_schema(&self, schema_name: &str) -> Result<(), SchemaError> {
        self.schema_manager.unload_schema(schema_name)
    }

    /// Get a schema by name - public accessor for testing
    pub fn get_schema(&self, schema_name: &str) -> Result<Option<crate::schema::Schema>, SchemaError> {
        self.schema_manager.get_schema(schema_name)
    }

    /// List all loaded (approved) schemas
    pub fn list_loaded_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.schema_manager.list_loaded_schemas()
    }

    /// List all available schemas (any state)
    pub fn list_available_schemas(&self) -> Result<Vec<String>, SchemaError> {
        self.schema_manager.list_available_schemas()
    }

    /// Get the current state of a schema
    pub fn get_schema_state(&self, schema_name: &str) -> Option<SchemaState> {
        self.schema_manager.get_schema_state(schema_name)
    }

    /// Check if a schema exists
    pub fn schema_exists(&self, schema_name: &str) -> Result<bool, SchemaError> {
        self.schema_manager.schema_exists(schema_name)
    }

    /// List all schemas with their states
    pub fn list_schemas_with_state(&self) -> Result<HashMap<String, SchemaState>, SchemaError> {
        self.load_schema_state()
    }

    /// Provides access to the underlying database operations
    pub fn db_ops(&self) -> Arc<DbOperations> {
        Arc::clone(&self.db_ops)
    }
}
