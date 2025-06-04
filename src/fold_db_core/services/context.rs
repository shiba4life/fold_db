use crate::atom::AtomStatus;
use crate::schema::types::field::FieldType;
use crate::schema::types::Field;
use crate::schema::Schema;
use crate::schema::SchemaError;
use crate::fold_db_core::infrastructure::message_bus::MessageBus;
use serde_json::Value;

#[allow(dead_code)]
pub struct AtomContext<'a> {
    schema: &'a Schema,
    field: &'a str,
    source_pub_key: String,
    ref_atom_uuid: Option<String>,
    message_bus: std::sync::Arc<MessageBus>,
}

impl<'a> AtomContext<'a> {
    pub fn new(
        schema: &'a Schema,
        field: &'a str,
        source_pub_key: String,
        message_bus: std::sync::Arc<MessageBus>,
    ) -> Self {
        Self {
            schema,
            field,
            source_pub_key,
            ref_atom_uuid: None,
            message_bus,
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

    /// DEPRECATED: Direct AtomRef access violates event-driven architecture
    /// Use AtomRefQueryRequest/AtomRefQueryResponse events instead
    pub fn atom_ref_exists(&self, _aref_uuid: &str) -> Result<bool, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomRefQueryRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct AtomRef creation violates event-driven architecture
    /// Use AtomRefCreateRequest/AtomRefCreateResponse events instead
    pub fn get_or_create_atom_ref(&mut self) -> Result<String, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomRefCreateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Use event-driven AtomRefCreateRequest via message bus instead
    pub fn get_or_create_atom_ref_safe(&mut self) -> Result<String, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomRefCreateRequest via message bus instead of direct method calls".to_string()
        ))
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

    /// DEPRECATED: Use event-driven AtomRefCreateRequest via message bus instead
    pub fn create_missing_atom_ref_with_uuid(&mut self, _uuid: &str) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomRefCreateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Use event-driven AtomGetRequest via message bus instead
    pub fn get_prev_atom_uuid(&self, _aref_uuid: &str) -> Result<String, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomGetRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Use event-driven AtomGetRequest via message bus instead
    pub fn get_prev_collection_atom_uuid(
        &self,
        _aref_uuid: &str,
        _id: &str,
    ) -> Result<String, SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomGetRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Direct atom operations violate event-driven architecture
    /// Use AtomCreateRequest and AtomRefUpdateRequest events instead
    pub fn create_and_update_atom(
        &mut self,
        _prev_atom_uuid: Option<String>,
        _content: Value,
        _status: Option<AtomStatus>,
    ) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomCreateRequest and AtomRefUpdateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Use event-driven AtomCreateRequest and AtomRefUpdateRequest via message bus instead
    pub fn create_and_update_range_atom(
        &mut self,
        _prev_atom_uuid: Option<String>,
        _content_key: &str,
        _content_value: Value,
        _status: Option<AtomStatus>,
    ) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomCreateRequest and AtomRefUpdateRequest via message bus instead of direct method calls".to_string()
        ))
    }

    /// DEPRECATED: Use event-driven AtomCreateRequest and AtomRefUpdateRequest via message bus instead
    pub fn create_and_update_collection_atom(
        &mut self,
        _prev_atom_uuid: Option<String>,
        _content: Value,
        _status: Option<AtomStatus>,
        _id: String,
    ) -> Result<(), SchemaError> {
        Err(SchemaError::InvalidData(
            "Method deprecated: Use event-driven AtomCreateRequest and AtomRefUpdateRequest via message bus instead of direct method calls".to_string()
        ))
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
