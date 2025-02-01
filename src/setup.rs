use std::collections::HashMap;
use std::sync::Arc;

use crate::folddb::{FoldDB, InternalSchema};

pub fn initialize_database_with_path(path: &str) -> Result<Arc<FoldDB>, Box<dyn std::error::Error>> {
    // Initialize FoldDB without Arc first
    let mut fold_db = FoldDB::new(path, create_initial_schemas())?;

    // Initialize example data
    let (username_aref, bio_aref) = create_initial_data(&mut fold_db)?;

    // Update schema with actual aref_uuids
    update_schema_with_refs(&mut fold_db, username_aref, bio_aref);

    // Wrap in Arc for thread-safe sharing
    Ok(Arc::new(fold_db))
}

fn create_initial_schemas() -> HashMap<String, InternalSchema> {
    let mut internal_schemas = HashMap::new();
    let mut user_profile_fields = HashMap::new();
    
    // Temporary UUIDs that will be replaced with actual aref_uuids
    user_profile_fields.insert("username".to_string(), "aref-uuid-for-username".to_string());
    user_profile_fields.insert("bio".to_string(), "aref-uuid-for-bio".to_string());
    
    internal_schemas.insert(
        "user_profile".to_string(),
        InternalSchema {
            fields: user_profile_fields,
        },
    );

    internal_schemas
}

fn create_initial_data(fold_db: &mut FoldDB) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Create initial atoms
    let username_atom = fold_db.create_atom(
        r#""john_doe""#.to_string(),
        "initial_value".to_string(),
        "system_init".to_string(),
        None,
    )?;
    let bio_atom = fold_db.create_atom(
        r#""Software engineer and Rust enthusiast""#.to_string(),
        "initial_value".to_string(),
        "system_init".to_string(),
        None,
    )?;

    // Create atom refs
    let username_aref = fold_db.create_atom_ref(&username_atom)?;
    let bio_aref = fold_db.create_atom_ref(&bio_atom)?;

    Ok((username_aref, bio_aref))
}

fn update_schema_with_refs(fold_db: &mut FoldDB, username_aref: String, bio_aref: String) {
    let mut user_profile_fields = HashMap::new();
    user_profile_fields.insert("username".to_string(), username_aref);
    user_profile_fields.insert("bio".to_string(), bio_aref);
    
    fold_db.internal_schemas.insert(
        "user_profile".to_string(),
        InternalSchema {
            fields: user_profile_fields,
        },
    );
}
