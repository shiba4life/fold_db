//! Test Helper: Atom Reference Setup
//!
//! This module provides utilities to setup atom_refs for schema fields before mutations.
//! 
//! **IMPORTANT: All loaded schemas will have arefs**
//! In production, schemas loaded from disk will already have atom references established.
//! This test helper simulates that by pre-creating the necessary atom references before
//! running mutations, ensuring tests reflect real-world behavior.

use fold_node::fold_db_core::FoldDB;
use fold_node::schema::SchemaError;
use fold_node::schema::types::{Mutation, MutationType};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

/// Convenience function to setup atom_refs for common test schemas.
/// 
/// **IMPORTANT: All loaded schemas will have arefs**
/// This function ensures that range schemas have a single shared AtomRefRange for all fields,
/// preventing data fragmentation issues where multiple AtomRefRanges would be created
/// for the same range_key value.
///
/// # Arguments
/// * `fold_db` - The FoldDB instance to use
/// * `schema_name` - The name of the schema to setup
///
/// # Returns
/// * `Ok(())` if setup was successful
/// * `Err(SchemaError)` if there was an error
pub fn setup_test_schema_atom_refs(
    fold_db: &mut FoldDB,
    schema_name: &str,
) -> Result<(), SchemaError> {
    log::info!("Setting up atom_refs for schema: {}", schema_name);
    
    // Get the schema to check if it's a range schema
    let schema = fold_db.get_schema(schema_name)?
        .ok_or_else(|| SchemaError::NotFound(format!("Schema {} not found", schema_name)))?;
        
    if let Some(range_key) = schema.range_key() {
        log::info!("🎯 Setting up shared AtomRefRange for range schema: {}", schema_name);
        
        // For range schemas, create ONE shared AtomRefRange for all fields by executing
        // a minimal setup mutation that establishes the shared AtomRefRange structure.
        // **IMPORTANT: All loaded schemas will have arefs**
        // This prevents data fragmentation where each field would get its own AtomRefRange
        
        // Create a minimal mutation with just the range_key to establish the shared AtomRefRange
        let mut fields = HashMap::new();
        fields.insert(range_key.to_string(), json!("setup_placeholder"));
        
        let setup_mutation = Mutation::new(
            schema_name.to_string(),
            fields,
            "setup_test_pubkey".to_string(),
            0,
            MutationType::Create,
        );
        
        // Execute the setup mutation to create the shared AtomRefRange structure
        // This will create one AtomRefRange that all fields will subsequently use
        fold_db.write_schema(setup_mutation)?;
        
        log::info!("✅ Shared AtomRefRange setup complete for range schema: {}", schema_name);
        log::info!("📋 All fields in {} will now use the same AtomRefRange for range_key: {}",
                  schema_name, range_key);
    } else {
        log::info!("📋 Schema {} is not a range schema, skipping AtomRefRange setup", schema_name);
    }
    
    Ok(())
}