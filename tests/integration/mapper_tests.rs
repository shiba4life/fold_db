use fold_db::folddb::FoldDB;
use fold_db::schema::types::{Schema, SchemaField, FieldType};
use fold_db::schema::mapper::{SchemaMapper, MappingRule};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_basic_mapping() {
    let mut db = FoldDB::new(&crate::get_test_db_path("basic_mapping")).unwrap();

    // Create source schema
    let mut source_schema = Schema::new("source".to_string());
    let name_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    let age_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    
    source_schema.add_field("name".to_string(), name_field);
    source_schema.add_field("age".to_string(), age_field);
    
    // Create target schema with fields and transforms
    let mut target_schema = Schema::new("target".to_string());
    
    // Add fields to target schema
    let full_name_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    let years_field = SchemaField::new(
        "W1".to_string(),
        Uuid::new_v4().to_string(),
        FieldType::Single,
    );
    
    target_schema.add_field("full_name".to_string(), full_name_field);
    target_schema.add_field("years".to_string(), years_field);
    
    // Add transforms to map source fields to target fields
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

    // Create source data
    let mut source_data = HashMap::new();
    source_data.insert("source".to_string(), json!({
        "name": "John Doe",
        "age": 30
    }));

    // Create and apply schema mapper
    let mapper = SchemaMapper::new(
        vec!["source".to_string()],
        "target".to_string(),
        vec![
            MappingRule::Rename {
                source_field: "name".to_string(),
                target_field: "full_name".to_string(),
            },
            MappingRule::Rename {
                source_field: "age".to_string(),
                target_field: "years".to_string(),
            },
        ],
    );

    let result = mapper.apply(&source_data).unwrap();
    
    // Verify mapped data
    assert_eq!(result.get("full_name").unwrap(), &json!("John Doe"));
    assert_eq!(result.get("years").unwrap(), &json!(30));
}
