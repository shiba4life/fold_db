use fold_db::schema::mapper::{SchemaMapper, MappingRule};
use serde_json::{json, Value};
use std::collections::HashMap;

fn create_test_data() -> HashMap<String, Value> {
    let mut data = HashMap::new();
    data.insert(
        "source1".to_string(),
        json!({
            "name": "John Doe",
            "email": "john@example.com",
            "age": 30,
            "notes": "test notes"
        })
    );
    data.insert(
        "source2".to_string(),
        json!({
            "user_name": "Jane Smith",
            "contact": "jane@example.com",
            "years": 25,
            "extra": "additional info"
        })
    );
    data
}

#[test]
fn test_rename_rule() {
    let rules = vec![
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "full_name".to_string(),
        },
        MappingRule::Rename {
            source_field: "email".to_string(),
            target_field: "contact_email".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("full_name").unwrap(), "John Doe");
    assert_eq!(result.get("contact_email").unwrap(), "john@example.com");
    assert!(result.get("name").is_none()); // Original field should not exist
    assert!(result.get("email").is_none()); // Original field should not exist
}

#[test]
fn test_drop_rule() {
    let rules = vec![
        MappingRule::Drop {
            field: "notes".to_string(),
        },
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "name".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("name").unwrap(), "John Doe");
    assert!(result.get("notes").is_none()); // Dropped field should not exist
}

#[test]
fn test_add_rule() {
    let rules = vec![
        MappingRule::Add {
            target_field: "type".to_string(),
            value: json!("user"),
        },
        MappingRule::Add {
            target_field: "active".to_string(),
            value: json!(true),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("type").unwrap(), "user");
    assert_eq!(result.get("active").unwrap(), true);
}

#[test]
fn test_map_rule() {
    let rules = vec![
        MappingRule::Map {
            source_field: "name".to_string(),
            target_field: "uppercase_name".to_string(),
            function: "to_uppercase".to_string(),
        },
        MappingRule::Map {
            source_field: "email".to_string(),
            target_field: "lowercase_email".to_string(),
            function: "to_lowercase".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("uppercase_name").unwrap(), "JOHN DOE");
    assert_eq!(result.get("lowercase_email").unwrap(), "john@example.com");
}

#[test]
fn test_multiple_sources() {
    let rules = vec![
        // Map from source1
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "name1".to_string(),
        },
        // Map from source2
        MappingRule::Rename {
            source_field: "user_name".to_string(),
            target_field: "name2".to_string(),
        },
        // Add static field
        MappingRule::Add {
            target_field: "source".to_string(),
            value: json!("multiple"),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string(), "source2".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("name1").unwrap(), "John Doe");
    assert_eq!(result.get("name2").unwrap(), "Jane Smith");
    assert_eq!(result.get("source").unwrap(), "multiple");
}

#[test]
fn test_combined_rules() {
    let rules = vec![
        // Rename and transform
        MappingRule::Map {
            source_field: "name".to_string(),
            target_field: "uppercase_name".to_string(),
            function: "to_uppercase".to_string(),
        },
        // Add static field
        MappingRule::Add {
            target_field: "type".to_string(),
            value: json!("user"),
        },
        // Drop unnecessary field
        MappingRule::Drop {
            field: "notes".to_string(),
        },
        // Simple rename
        MappingRule::Rename {
            source_field: "email".to_string(),
            target_field: "contact_email".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let result = mapper.apply(&create_test_data()).unwrap();
    
    assert_eq!(result.get("uppercase_name").unwrap(), "JOHN DOE");
    assert_eq!(result.get("type").unwrap(), "user");
    assert_eq!(result.get("contact_email").unwrap(), "john@example.com");
    assert!(result.get("notes").is_none());
}

#[test]
fn test_invalid_source_data() {
    let rules = vec![
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "full_name".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["source1".to_string()],
        "target".to_string(),
        rules,
    );

    let mut invalid_data = HashMap::new();
    invalid_data.insert("source1".to_string(), json!("not an object"));

    let result = mapper.apply(&invalid_data);
    assert!(result.is_err());
}
