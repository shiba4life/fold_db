use crate::schema::types::errors::SchemaError;
use crate::schema::types::schema::Schema;
use crate::schema::types::field::variant::FieldVariant;
use crate::schema::types::field::common::Field;
use serde_json::{Value as JsonValue, Value};
use std::sync::Arc;
use log::{info, error};

/// Unified field value resolver that consolidates duplicate field retrieval logic
pub struct FieldValueResolver;

impl FieldValueResolver {
    /// Unified field value resolution from schema using database operations
    /// Consolidates the duplicate implementations from execution.rs, mod.rs, and field_retrieval/service.rs
    pub fn resolve_field_value(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema: &Schema,
        field_name: &str,
    ) -> Result<JsonValue, SchemaError> {
        info!("🔍 FieldValueResolver: Looking up field '{}' in schema '{}'", field_name, schema.name);
        
        // Get field definition from schema
        let field = schema.fields.get(field_name)
            .ok_or_else(|| {
                error!("❌ Field '{}' not found in schema '{}'", field_name, schema.name);
                SchemaError::InvalidField(format!("Field '{}' not found in schema '{}'", field_name, schema.name))
            })?;
        
        info!("✅ Field '{}' found in schema '{}'", field_name, schema.name);
        
        // Extract ref_atom_uuid from field variant
        let ref_atom_uuid = Self::extract_ref_atom_uuid(field, field_name)?;
        info!("🔗 Field ref_atom_uuid: {}", ref_atom_uuid);
        
        // Load AtomRef from database
        let atom_ref = Self::load_atom_ref(db_ops, &ref_atom_uuid)?;
        
        // Get atom_uuid from AtomRef
        let atom_uuid = atom_ref.get_atom_uuid();
        info!("🔗 AtomRef points to atom: {}", atom_uuid);
        
        // Load Atom from database
        let atom = Self::load_atom(db_ops, atom_uuid)?;
        
        info!("✅ Atom loaded successfully");
        let content = atom.content().clone();
        info!("📦 Atom content: {}", content);
        
        Ok(content)
    }
    
    /// Extract ref_atom_uuid from field variant with consistent error handling
    fn extract_ref_atom_uuid(field: &FieldVariant, field_name: &str) -> Result<String, SchemaError> {
        let ref_atom_uuid = field.ref_atom_uuid()
            .ok_or_else(|| {
                error!("❌ Field '{}' has no ref_atom_uuid", field_name);
                SchemaError::InvalidField(format!("Field '{}' has no ref_atom_uuid", field_name))
            })?
            .clone();
        Ok(ref_atom_uuid)
    }
    
    /// Load AtomRef from database with consistent error handling
    fn load_atom_ref(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        ref_atom_uuid: &str,
    ) -> Result<crate::atom::AtomRef, SchemaError> {
        info!("🔍 Loading AtomRef from database...");
        db_ops.get_item(&format!("ref:{}", ref_atom_uuid))?
            .ok_or_else(|| {
                error!("❌ AtomRef '{}' not found", ref_atom_uuid);
                SchemaError::InvalidField(format!("AtomRef '{}' not found", ref_atom_uuid))
            })
    }
    
    /// Load Atom from database with consistent error handling
    fn load_atom(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        atom_uuid: &str,
    ) -> Result<crate::atom::Atom, SchemaError> {
        info!("🔍 Loading Atom from database...");
        db_ops.get_item(&format!("atom:{}", atom_uuid))?
            .ok_or_else(|| {
                error!("❌ Atom '{}' not found", atom_uuid);
                SchemaError::InvalidField(format!("Atom '{}' not found", atom_uuid))
            })
    }
    
    /// Convenience method that returns Value instead of JsonValue for compatibility
    pub fn resolve_field_value_as_value(
        db_ops: &Arc<crate::db_operations::DbOperations>,
        schema: &Schema,
        field_name: &str,
    ) -> Result<Value, SchemaError> {
        Self::resolve_field_value(db_ops, schema, field_name)
    }
}