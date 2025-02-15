use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::atom::{Atom, AtomRef};
use crate::permissions::PermissionWrapper;
use crate::schema::schema_manager::SchemaManager;
use crate::schema::types::{Mutation, Query};
use crate::schema::{Schema, SchemaError};

/// The main database coordinator that manages schemas, permissions, and data storage.
/// 
/// FoldDB serves as the central point of coordination for all database operations,
/// managing the interactions between:
/// - Schema validation and management
/// - Permission checking and access control
/// - Atomic data storage and versioning
/// - Field-level data operations
/// 
/// The database uses an embedded sled instance for persistent storage while maintaining
/// in-memory caches for frequently accessed atoms and atom references.
pub struct FoldDB {
    /// The underlying sled database instance for persistent storage
    pub db: sled::Db,
    /// In-memory cache of atoms for faster access
    pub atoms: HashMap<String, Atom>,
    /// In-memory cache of atom references for faster access
    pub ref_atoms: HashMap<String, AtomRef>,
    /// Manager for schema validation and transformation
    pub schema_manager: SchemaManager,
    /// Wrapper for handling permission checks
    permission_wrapper: PermissionWrapper,
}

impl FoldDB {
    /// Creates a new FoldDB instance with the specified storage path.
    /// 
    /// # Arguments
    /// 
    /// * `path` - The filesystem path where the sled database will store its data
    /// 
    /// # Returns
    /// 
    /// A Result containing the new FoldDB instance or a sled error
    pub fn new(path: &str) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self {
            db,
            atoms: HashMap::new(),
            ref_atoms: HashMap::new(),
            schema_manager: SchemaManager::new(),
            permission_wrapper: PermissionWrapper::new(),
        })
    }

    /// Loads and validates a schema into the database.
    /// 
    /// This method:
    /// 1. Validates the schema structure
    /// 2. Runs any specified transformations
    /// 3. Maps schema fields to their storage locations
    /// 4. Makes the schema available for queries and mutations
    /// 
    /// # Arguments
    /// 
    /// * `schema` - The schema to load and validate
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing a SchemaError
    pub fn load_schema(&mut self, schema: Schema) -> Result<(), SchemaError> {
        let name = schema.name.clone();
        self.schema_manager.load_schema(schema)?;
        self.schema_manager.map_fields(&name)?;
        Ok(())
    }

    /// Enables querying and writing operations for a schema.
    /// 
    /// The schema must have been previously loaded using `load_schema`.
    /// This separation allows for schema validation before enabling operations.
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema to enable
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing a SchemaError if the schema doesn't exist
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

    /// Executes a query against a schema, checking permissions for each field.
    /// 
    /// This method:
    /// 1. Validates permissions for each requested field
    /// 2. Retrieves allowed field values
    /// 3. Returns results for each field individually
    /// 
    /// # Arguments
    /// 
    /// * `query` - The query containing schema name, fields, and authentication info
    /// 
    /// # Returns
    /// 
    /// A vector of Results, one for each requested field, containing either
    /// the field value or a SchemaError
    pub fn query_schema(&self, query: Query) -> Vec<Result<Value, SchemaError>> {
        // Process each field, checking permissions individually
        query
            .fields
            .iter()
            .map(|field_name| {
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

                self.get_field_value(
                    &query.schema_name,
                    field_name,
                    &query.pub_key,
                    query.trust_distance,
                )
            })
            .collect()
    }

    /// Writes data to a schema, checking permissions for each field.
    /// 
    /// This method:
    /// 1. Validates permissions for each field being written
    /// 2. Creates new Atoms for the updated values
    /// 3. Updates AtomRefs to point to the new versions
    /// 4. Maintains the version history chain
    /// 
    /// # Arguments
    /// 
    /// * `mutation` - The mutation containing schema name, field values, and authentication info
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing a SchemaError
    pub fn write_schema(&mut self, mutation: Mutation) -> Result<(), SchemaError> {
        // Process each field, checking permissions individually
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

            self.set_field_value(
                &mutation.schema_name,
                field_name,
                value.clone(),
                mutation.pub_key.clone(),
            )?;
        }
        Ok(())
    }

    /// Retrieves the latest version of an Atom through its AtomRef.
    /// 
    /// First checks the in-memory cache, then falls back to disk storage.
    /// 
    /// # Arguments
    /// 
    /// * `aref_uuid` - UUID of the AtomRef pointing to the desired Atom
    /// 
    /// # Returns
    /// 
    /// A Result containing either the latest Atom or an error if not found
    fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        // Try in-memory cache first
        if let Some(aref) = self.ref_atoms.get(aref_uuid) {
            if let Some(atom) = self.atoms.get(aref.get_atom_uuid().unwrap()) {
                return Ok(atom.clone());
            }
        }

        // Try from disk
        let aref_bytes = self
            .db
            .get(aref_uuid.as_bytes())?
            .ok_or("AtomRef not found")?;
        let aref: AtomRef = serde_json::from_slice(&aref_bytes)?;

        let atom_bytes = self
            .db
            .get(aref.get_atom_uuid().unwrap().as_bytes())?
            .ok_or("Atom not found")?;
        let atom: Atom = serde_json::from_slice(&atom_bytes)?;

        Ok(atom)
    }

    /// Retrieves the complete version history for an Atom.
    /// 
    /// Follows the chain of prev_atom_uuid references to build the full history,
    /// starting from the most recent version and working backwards.
    /// 
    /// # Arguments
    /// 
    /// * `aref_uuid` - UUID of the AtomRef pointing to the most recent version
    /// 
    /// # Returns
    /// 
    /// A Result containing a vector of Atoms in chronological order (newest first)
    pub fn get_atom_history(
        &self,
        aref_uuid: &str,
    ) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
        let mut history = Vec::new();
        let mut current_atom = self.get_latest_atom(aref_uuid)?;

        history.push(current_atom.clone());

        while let Some(prev_uuid) = current_atom.prev_atom_uuid() {
            let atom_bytes = self
                .db
                .get(prev_uuid.as_bytes())?
                .ok_or("Previous atom not found")?;
            current_atom = serde_json::from_slice(&atom_bytes)?;
            history.push(current_atom.clone());
        }

        Ok(history)
    }

    /// Retrieves the value of a specific field from a schema.
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema containing the field
    /// * `field` - Name of the field to retrieve
    /// * `_pub_key` - Public key for authentication (currently unused)
    /// * `_trust_distance` - Trust distance for permission calculation (currently unused)
    /// 
    /// # Returns
    /// 
    /// A Result containing either the field value or a SchemaError
    pub fn get_field_value(
        &self,
        schema_name: &str,
        field: &str,
        _pub_key: &str,
        _trust_distance: u32,
    ) -> Result<Value, SchemaError> {
        let schema = self
            .schema_manager
            .get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;

        let field = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // If no ref_atom_uuid is set, return null
        let Some(ref_atom_uuid) = &field.ref_atom_uuid else {
            return Ok(Value::Null);
        };

        match self.get_latest_atom(ref_atom_uuid) {
            Ok(atom) => Ok(atom.content().clone()),
            Err(_) => Ok(Value::Null),
        }
    }

    /// Sets the value of a specific field in a schema.
    /// 
    /// This method:
    /// 1. Creates a new Atom with the updated value
    /// 2. Links it to the previous version if one exists
    /// 3. Updates or creates an AtomRef to point to the new version
    /// 4. Persists changes to both memory and disk
    /// 
    /// # Arguments
    /// 
    /// * `schema_name` - Name of the schema containing the field
    /// * `field` - Name of the field to update
    /// * `content` - New value for the field
    /// * `source_pub_key` - Public key of the entity making the change
    /// 
    /// # Returns
    /// 
    /// A Result indicating success or containing a SchemaError
    pub fn set_field_value(
        &mut self,
        schema_name: &str,
        field: &str,
        content: Value,
        source_pub_key: String,
    ) -> Result<(), SchemaError> {
        let schema = self
            .schema_manager
            .get_schema(schema_name)?
            .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;

        let field = schema
            .fields
            .get(field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", field)))?;

        // If there's no ref_atom_uuid, create a new one
        let aref_uuid = field.ref_atom_uuid.clone().unwrap_or_else(|| {
            let aref_uuid = Uuid::new_v4().to_string();
            let aref = AtomRef::new(aref_uuid.clone());
            self.ref_atoms.insert(aref_uuid.clone(), aref);
            aref_uuid
        });

        let prev_atom_uuid = self
            .ref_atoms
            .get(&aref_uuid)
            .map(|aref| aref.get_atom_uuid().unwrap().clone());

        // Create new atom
        let atom = Atom::new(
            schema_name.to_string(),
            source_pub_key,
            prev_atom_uuid,
            content,
        );

        // Store value and update in-memory cache
        let atom_bytes = serde_json::to_vec(&atom)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize atom: {}", e)))?;
        self.db
            .insert(atom.uuid().as_bytes(), atom_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom: {}", e)))?;
        self.atoms.insert(atom.uuid().to_string(), atom.clone());

        // Update atom ref with new atom UUID
        let mut aref = self
            .ref_atoms
            .get(&aref_uuid)
            .cloned()
            .unwrap_or_else(|| AtomRef::new(atom.uuid().to_string()));

        // Set the new atom UUID
        aref.set_atom_uuid(atom.uuid().to_string());
        self.ref_atoms.insert(aref_uuid.clone(), aref.clone());

        // Store atom ref
        let aref_bytes = serde_json::to_vec(&aref).map_err(|e| {
            SchemaError::InvalidData(format!("Failed to serialize atom ref: {}", e))
        })?;
        self.db
            .insert(aref_uuid.as_bytes(), aref_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to store atom ref: {}", e)))?;

        Ok(())
    }
}
