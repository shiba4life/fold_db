use serde_json::json;
use fold_db::schema::types::{Schema, FieldType};
use fold_db::folddb::FoldDB;
use uuid::Uuid;

pub fn setup_profile_schema(db: &mut FoldDB) -> Result<(), Box<dyn std::error::Error>> {
    // Create schema
    let mut schema = Schema::new("profile".to_string());
    
    // Add fields with default permissions
    let name_field = fold_db::schema::types::SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    schema.add_field("name".to_string(), name_field);

    let bio_field = fold_db::schema::types::SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    schema.add_field("bio".to_string(), bio_field);

    // Load schema and set initial values
    db.load_schema(schema)?;
    db.set_field_value("profile", "name", json!("John Doe"), "system_init".to_string())?;
    db.set_field_value("profile", "bio", json!("A software engineer"), "system_init".to_string())?;

    Ok(())
}
