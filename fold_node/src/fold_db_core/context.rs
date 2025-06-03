use super::atom_manager::AtomManager;
use crate::atom::{AtomRef, AtomRefCollection, AtomRefRange, AtomStatus};
use crate::schema::types::field::FieldType;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use serde_json::Value;
use uuid::Uuid;

pub struct AtomContext<'a> {
    schema: &'a Schema,
    field: &'a str,
    source_pub_key: String,
    ref_atom_uuid: Option<String>,
    pub(super) atom_manager: &'a mut AtomManager,
}

impl<'a> AtomContext<'a> {
    pub fn new(
        schema: &'a Schema,
        field: &'a str,
        source_pub_key: String,
        atom_manager: &'a mut AtomManager,
    ) -> Self {
        Self {
            schema,
            field,
            source_pub_key,
            ref_atom_uuid: None,
            atom_manager,
        }
    }

    /// Set the ref_atom_uuid for this context (used when reusing existing AtomRef)
    pub fn set_ref_atom_uuid(&mut self, uuid: String) {
        self.ref_atom_uuid = Some(uuid);
    }

    pub fn get_field_def(&self) -> Result<&'a crate::schema::types::FieldVariant, SchemaError> {
        self.schema
            .fields
            .get(self.field)
            .ok_or_else(|| SchemaError::InvalidField(format!("Field {} not found", self.field)))
    }

    /// Get the expected AtomRef type name for the current field variant
    pub fn get_expected_atom_ref_type(&self) -> Result<&'static str, SchemaError> {
        let field_def = self.get_field_def()?;
        
        match field_def {
            crate::schema::types::FieldVariant::Single(_) => Ok("AtomRef"),
            crate::schema::types::FieldVariant::Collection(_) => Ok("AtomRefCollection"),
            crate::schema::types::FieldVariant::Range(_) => Ok("AtomRefRange"),
        }
    }

    /// Check if an AtomRef exists for the given UUID and matches the field variant type
    pub fn atom_ref_exists(&self, aref_uuid: &str) -> Result<bool, SchemaError> {
        let field_def = self.get_field_def()?;
        
        match field_def {
            crate::schema::types::FieldVariant::Single(_) => {
                let ref_atoms = self.atom_manager.get_ref_atoms();
                let guard = ref_atoms.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_atoms lock: {}", e))
                })?;
                Ok(guard.contains_key(aref_uuid))
            }
            crate::schema::types::FieldVariant::Collection(_) => {
                let ref_collections = self.atom_manager.get_ref_collections();
                let guard = ref_collections.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_collections lock: {}", e))
                })?;
                Ok(guard.contains_key(aref_uuid))
            }
            crate::schema::types::FieldVariant::Range(_) => {
                let ref_ranges = self.atom_manager.get_ref_ranges();
                let guard = ref_ranges.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_ranges lock: {}", e))
                })?;
                Ok(guard.contains_key(aref_uuid))
            }
        }
    }

    pub fn get_or_create_atom_ref(&mut self) -> Result<String, SchemaError> {
        let field_def = self.get_field_def()?;
        if field_def.ref_atom_uuid().is_some() {
            self.ref_atom_uuid = field_def.ref_atom_uuid().map(|uuid| uuid.to_string());
            return Ok(self.ref_atom_uuid.clone().unwrap());
        }

        let aref_uuid = if let Some(uuid) = self.ref_atom_uuid.clone() {
            println!("🔑 Using existing aref_uuid: {}", uuid);
            // Validate that the existing AtomRef matches the field variant type
            if !self.atom_ref_exists(&uuid)? {
                return Err(SchemaError::InvalidData(format!(
                    "AtomRef type mismatch for UUID {}: expected {:?} but AtomRef not found",
                    uuid, self.get_expected_atom_ref_type()?
                )));
            }
            uuid.clone()
        } else {
            let aref_uuid = Uuid::new_v4().to_string();
            
            // Create the appropriate AtomRef type based on field variant
            match field_def {
                crate::schema::types::FieldVariant::Single(_) => {
                    // For single fields, we create a placeholder atom_uuid that will be set when data is stored
                    let placeholder_atom_uuid = format!("placeholder-{}", aref_uuid);
                    let aref = AtomRef::new(placeholder_atom_uuid, self.source_pub_key.clone());
                    let ref_atoms = self.atom_manager.get_ref_atoms();
                    let mut guard = ref_atoms.lock().map_err(|e| {
                        SchemaError::InvalidData(format!("Failed to acquire ref_atoms lock for UUID {}: {}", aref_uuid, e))
                    })?;
                    
                    // Check if AtomRef already exists to prevent double insertion
                    if !guard.contains_key(&aref_uuid) {
                        guard.insert(aref_uuid.clone(), aref);
                        println!("✅ Created AtomRef for Single field with UUID: {}", aref_uuid);
                    }
                }
                crate::schema::types::FieldVariant::Collection(_) => {
                    let collection = AtomRefCollection::new(self.source_pub_key.clone());
                    let ref_collections = self.atom_manager.get_ref_collections();
                    let mut guard = ref_collections.lock().map_err(|e| {
                        SchemaError::InvalidData(format!("Failed to acquire ref_collections lock for UUID {}: {}", aref_uuid, e))
                    })?;
                    
                    // Check if AtomRefCollection already exists to prevent double insertion
                    if !guard.contains_key(&aref_uuid) {
                        guard.insert(aref_uuid.clone(), collection);
                        println!("✅ Created AtomRefCollection for Collection field with UUID: {}", aref_uuid);
                    }
                }
                crate::schema::types::FieldVariant::Range(_) => {
                    let range = AtomRefRange::new(self.source_pub_key.clone());
                    let ref_ranges = self.atom_manager.get_ref_ranges();
                    let mut guard = ref_ranges.lock().map_err(|e| {
                        SchemaError::InvalidData(format!("Failed to acquire ref_ranges lock for UUID {}: {}", aref_uuid, e))
                    })?;
                    
                    // Check if AtomRefRange already exists to prevent double insertion
                    if !guard.contains_key(&aref_uuid) {
                        guard.insert(aref_uuid.clone(), range);
                        println!("✅ Created AtomRefRange for Range field with UUID: {}", aref_uuid);
                    }
                }
            }
            aref_uuid
        };

        self.ref_atom_uuid = Some(aref_uuid.clone());

        Ok(aref_uuid)
    }

    /// Get or create AtomRef with improved error recovery
    pub fn get_or_create_atom_ref_safe(&mut self) -> Result<String, SchemaError> {
        let field_def = self.get_field_def()?;
        
        // First check if field already has a ref_atom_uuid
        if let Some(existing_uuid) = field_def.ref_atom_uuid() {
            let uuid_str = existing_uuid.to_string();
            // Verify the AtomRef actually exists and matches the field type
            if self.atom_ref_exists(&uuid_str)? {
                self.ref_atom_uuid = Some(uuid_str.clone());
                return Ok(uuid_str);
            } else {
                // Ghost UUID detected - field has ref_atom_uuid but no AtomRef exists
                println!("⚠️ Ghost UUID detected: {} - AtomRef missing, creating new one", uuid_str);
                
                // FIX: Create missing AtomRef using the EXISTING ghost UUID instead of generating new one
                // This ensures data continuity and prevents fragmentation in range fields
                self.create_missing_atom_ref_with_uuid(&uuid_str)?;
                self.ref_atom_uuid = Some(uuid_str.clone());
                return Ok(uuid_str);
            }
        }
        
        // Create new AtomRef using the existing logic
        self.get_or_create_atom_ref()
    }

    /// Get existing AtomRef UUID without creating new ones - for mutations only.
    /// 
    /// **CRITICAL: Mutation-Only Method**
    /// 
    /// This method is designed specifically for mutations and will fail if no atom_ref exists.
    /// It ensures that mutations never create new atom_refs, preventing data fragmentation.
    /// 
    /// Returns: The UUID of the existing AtomRef
    pub fn get_existing_atom_ref(&mut self) -> Result<String, SchemaError> {
        let field_def = self.get_field_def()?;
        
        // Check if field already has a ref_atom_uuid
        if let Some(existing_uuid) = field_def.ref_atom_uuid() {
            let uuid_str = existing_uuid.to_string();
            // Verify the AtomRef actually exists and matches the field type
            if self.atom_ref_exists(&uuid_str)? {
                self.ref_atom_uuid = Some(uuid_str.clone());
                return Ok(uuid_str);
            } else {
                // AtomRef not in memory - create it (normal for schemas loaded from disk)
                log::info!("🔧 Creating missing AtomRef for field {} with UUID {}", self.field, uuid_str);
                self.create_missing_atom_ref_with_uuid(&uuid_str)?;
                self.ref_atom_uuid = Some(uuid_str.clone());
                return Ok(uuid_str);
            }
        }
        
        // No existing atom_ref found - fail for mutations
        Err(SchemaError::InvalidData(format!(
            "No existing atom_ref found for field {}. Mutations cannot create new atom_refs.",
            self.field
        )))
    }

    /// Create missing AtomRef using the provided UUID (for ghost UUID recovery)
    /// This ensures that ghost UUIDs point to actual AREFs and prevents data fragmentation
    pub fn create_missing_atom_ref_with_uuid(&mut self, uuid: &str) -> Result<(), SchemaError> {
        let field_def = self.get_field_def()?;
        
        match field_def {
            crate::schema::types::FieldVariant::Single(_) => {
                // For single fields, create placeholder atom_uuid that will be set when data is stored
                let placeholder_atom_uuid = format!("placeholder-{}", uuid);
                let aref = AtomRef::new(placeholder_atom_uuid, self.source_pub_key.clone());
                let ref_atoms = self.atom_manager.get_ref_atoms();
                let mut guard = ref_atoms.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_atoms lock for UUID {}: {}", uuid, e))
                })?;
                
                if !guard.contains_key(uuid) {
                    guard.insert(uuid.to_string(), aref);
                    println!("✅ Recreated missing AtomRef for Single field with ghost UUID: {}", uuid);
                }
            }
            crate::schema::types::FieldVariant::Collection(_) => {
                let collection = AtomRefCollection::new(self.source_pub_key.clone());
                let ref_collections = self.atom_manager.get_ref_collections();
                let mut guard = ref_collections.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_collections lock for UUID {}: {}", uuid, e))
                })?;
                
                if !guard.contains_key(uuid) {
                    guard.insert(uuid.to_string(), collection);
                    println!("✅ Recreated missing AtomRefCollection for Collection field with ghost UUID: {}", uuid);
                }
            }
            crate::schema::types::FieldVariant::Range(_) => {
                let range = AtomRefRange::new(self.source_pub_key.clone());
                let ref_ranges = self.atom_manager.get_ref_ranges();
                let mut guard = ref_ranges.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_ranges lock for UUID {}: {}", uuid, e))
                })?;
                
                if !guard.contains_key(uuid) {
                    guard.insert(uuid.to_string(), range);
                    println!("✅ Recreated missing AtomRefRange for Range field with ghost UUID: {}", uuid);
                }
            }
        }
        
        Ok(())
    }

    pub fn get_prev_atom_uuid(&self, aref_uuid: &str) -> Result<String, SchemaError> {
        let field_def = self.get_field_def()?;

        match field_def {
            crate::schema::types::FieldVariant::Single(_) => {
                let ref_atoms = self.atom_manager.get_ref_atoms();
                let guard = ref_atoms.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_atoms lock for UUID {}: {}", aref_uuid, e))
                })?;
                let aref = guard
                    .get(aref_uuid)
                    .ok_or_else(|| SchemaError::InvalidData(format!("AtomRef {} not found", aref_uuid)))?;
                Ok(aref.get_atom_uuid().to_string())
            }
            crate::schema::types::FieldVariant::Collection(_) => {
                let ref_collections = self.atom_manager.get_ref_collections();
                let guard = ref_collections.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_collections lock for UUID {}: {}", aref_uuid, e))
                })?;
                let _collection = guard.get(aref_uuid).ok_or_else(|| {
                    SchemaError::InvalidData(format!("AtomRefCollection {} not found", aref_uuid))
                })?;
                // For collections, we need to get the latest atom UUID - this might need adjustment
                // For now, return empty string to indicate no previous atom
                Ok(String::new())
            }
            crate::schema::types::FieldVariant::Range(_) => {
                let ref_ranges = self.atom_manager.get_ref_ranges();
                let guard = ref_ranges.lock().map_err(|e| {
                    SchemaError::InvalidData(format!("Failed to acquire ref_ranges lock for UUID {}: {}", aref_uuid, e))
                })?;
                let range = guard.get(aref_uuid).ok_or_else(|| {
                    SchemaError::InvalidData(format!("AtomRefRange {} not found", aref_uuid))
                })?;
                
                // For ranges, return the most recently updated atom UUID
                // Since ranges contain multiple key-value pairs, we get the last inserted one
                if let Some((_key, atom_uuid)) = range.atom_uuids.iter().next_back() {
                    Ok(atom_uuid.clone())
                } else {
                    // No atoms in range yet, return empty string
                    Ok(String::new())
                }
            }
        }
    }

    pub fn get_prev_collection_atom_uuid(
        &self,
        aref_uuid: &str,
        id: &str,
    ) -> Result<String, SchemaError> {
        let ref_collections = self.atom_manager.get_ref_collections();
        let guard = ref_collections.lock().map_err(|e| {
            SchemaError::InvalidData(format!("Failed to acquire ref_collections lock for UUID {}: {}", aref_uuid, e))
        })?;
        let aref = guard
            .get(aref_uuid)
            .ok_or_else(|| SchemaError::InvalidData(format!("AtomRefCollection {} not found", aref_uuid)))?;
        aref.get_atom_uuid(id)
            .ok_or_else(|| SchemaError::InvalidData(format!("Atom with id '{}' not found in collection {}", id, aref_uuid)))
            .map(|uuid| uuid.to_string())
    }

    pub fn create_and_update_atom(
        &mut self,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
    ) -> Result<(), SchemaError> {
        let atom = self
            .atom_manager
            .create_atom(
                &self.schema.name,
                self.source_pub_key.clone(),
                prev_atom_uuid,
                content,
                status,
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        let aref_uuid = self.get_or_create_atom_ref()?;
        let _field_def = self.get_field_def()?;

        self.atom_manager
        .update_atom_ref(
            &aref_uuid,
            atom.uuid().to_string(),
            self.source_pub_key.clone(),
        )
        .map_err(|e| SchemaError::InvalidData(e.to_string()))?;
        Ok(())
    }

    pub fn create_and_update_range_atom(
        &mut self,
        prev_atom_uuid: Option<String>,
        content_key: &str,
        content_value: Value,
        status: Option<AtomStatus>,
    ) -> Result<(), SchemaError> {
        let aref_uuid = self.get_or_create_atom_ref()?;

        let atom = self
        .atom_manager
        .create_atom(
            &self.schema.name,
            self.source_pub_key.clone(),
            prev_atom_uuid, // No previous atom for individual keys
            content_value.clone(),
            status,
        )
        .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        self.atom_manager
            .update_atom_ref_range(
                &aref_uuid,
                atom.uuid().to_string(),
                content_key.to_string(),
                self.source_pub_key.clone(),
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        Ok(())
    }

    pub fn create_and_update_collection_atom(
        &mut self,
        prev_atom_uuid: Option<String>,
        content: Value,
        status: Option<AtomStatus>,
        id: String,
    ) -> Result<(), SchemaError> {
        let atom = self
            .atom_manager
            .create_atom(
                &self.schema.name,
                self.source_pub_key.clone(),
                prev_atom_uuid,
                content,
                status,
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        let aref_uuid = self.get_or_create_atom_ref()?;

        self.atom_manager
            .update_atom_ref_collection(
                &aref_uuid,
                atom.uuid().to_string(),
                id,
                self.source_pub_key.clone(),
            )
            .map_err(|e| SchemaError::InvalidData(e.to_string()))?;

        Ok(())
    }

    pub fn validate_field_type(&self, expected_type: FieldType) -> Result<(), SchemaError> {
        let field_def = self.get_field_def()?;
        let matches = matches!(
            (field_def, &expected_type),
            (
                crate::schema::types::FieldVariant::Single(_),
                &FieldType::Single
            ) | (
                crate::schema::types::FieldVariant::Collection(_),
                &FieldType::Collection
            ) | (
                crate::schema::types::FieldVariant::Range(_),
                &FieldType::Range
            )
        );

        if !matches {
            let msg = match &expected_type {
                FieldType::Single => "Collection fields cannot be updated without id",
                FieldType::Collection => "Single fields cannot be updated with collection id",
                FieldType::Range => "Incorrect field type for range operation",
            };
            return Err(SchemaError::InvalidField(msg.to_string()));
        }
        Ok(())
    }
}
