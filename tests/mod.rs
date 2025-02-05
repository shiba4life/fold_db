pub mod data;
pub mod integration;

use std::fs;
use fold_db::folddb::FoldDB;
use fold_db::schema::types::{Schema, FieldType};
use serde_json::json;
use uuid::Uuid;

fn get_test_db_path(prefix: &str) -> String {
    let tmp_dir = "tmp";
    fs::create_dir_all(tmp_dir).unwrap();
    format!("{}/test_{}_{}", tmp_dir, prefix, Uuid::new_v4())
}

#[test]
fn test_basic_schema() {
    let mut db = FoldDB::new(&get_test_db_path("basic_schema")).unwrap();
    
    // Create schema
    let mut schema = Schema::new("test".to_string());
    
    // Add field
    let field = fold_db::schema::types::SchemaField::new(
        "W1".to_string(),
        uuid::Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    schema.add_field("test_field".to_string(), field);
    
    // Load schema
    db.load_schema(schema).unwrap();
    
    // Set and get field value
    db.set_field_value("test", "test_field", json!("test value"), "test".to_string()).unwrap();
    let value = db.get_field_value("test", "test_field").unwrap();
    
    assert_eq!(value, json!("test value"));
}
