use std::sync::Arc;
use fold_db::folddb::FoldDB;
use fold_db::schema::types::{Schema, SchemaField};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_basic_mapping() {
    let mut db = FoldDB::new(&crate::get_test_db_path("basic_mapping")).unwrap();

    // Create source schema
    let mut source_schema = Schema::new("source".to_string());
    let name_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
    );
    let age_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
    );
    
    source_schema.add_field("name".to_string(), name_field);
    source_schema.add_field("age".to_string(), age_field);
    
    // Create target schema with transforms
    let mut target_schema = Schema::new("target".to_string());
    target_schema.add_transform("RENAME name TO full_name".to_string());
    target_schema.add_transform("RENAME age TO years".to_string());

    // Load schemas
    db.load_schema(source_schema).unwrap();
    db.load_schema(target_schema).unwrap();

    // Write source data
    db.set_field_value(
        "source",
        "name",
        json!("John Doe"),
        "test".to_string(),
    ).unwrap();
    db.set_field_value(
        "source",
        "age",
        json!(30),
        "test".to_string(),
    ).unwrap();

    // Query target schema
    let full_name = db.get_field_value("target", "full_name").unwrap();
    let years = db.get_field_value("target", "years").unwrap();

    assert_eq!(full_name, json!("John Doe"));
    assert_eq!(years, json!(30));
}
