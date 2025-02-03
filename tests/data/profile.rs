use fold_db::schema::InternalSchema;
use fold_db::folddb::FoldDB;
use std::collections::HashMap;

pub fn create_test_profile(fold_db: &FoldDB) -> Result<(String, String), Box<dyn std::error::Error>> {
    // Create profile atoms
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

    // Create atom refs for profile
    let username_aref = fold_db.create_atom_ref(&username_atom)?;
    let bio_aref = fold_db.create_atom_ref(&bio_atom)?;

    // Create and load schema
    let mut user_profile_fields = HashMap::new();
    user_profile_fields.insert("username".to_string(), username_aref.clone());
    user_profile_fields.insert("bio".to_string(), bio_aref.clone());
    
    fold_db.load_schema(
        "user_profile",
        InternalSchema {
            fields: user_profile_fields,
        },
    ).map_err(|e| e.to_string())?;

    Ok((username_aref, bio_aref))
}
