use serde_json::json;
use std::collections::HashMap;
use crate::schema::mapper::{SchemaMapper, MappingRule};

#[test]
fn test_schema_mapper_apply() {
    let rules = vec![
        MappingRule::Rename {
            source_field: "username".to_string(),
            target_field: "displayName".to_string(),
        },
        MappingRule::Drop {
            field: "privateEmail".to_string(),
        },
        MappingRule::Add {
            target_field: "status".to_string(),
            value: json!("active"),
        },
        MappingRule::Map {
            source_field: "name".to_string(),
            target_field: "upperName".to_string(),
            function: "to_uppercase".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["profile".to_string(), "legacy".to_string()],
        "public_profile".to_string(),
        rules,
    );

    let mut sources = HashMap::new();
    sources.insert("profile".to_string(), json!({
        "username": "john_doe",
        "privateEmail": "john@example.com",
        "name": "John Doe",
        "bio": "Hello!"
    }));
    sources.insert("legacy".to_string(), json!({
        "username": "old_john",
        "privateEmail": "old@example.com",
        "name": "John Old",
        "bio": "Old bio"
    }));

    let result = mapper.apply(&sources).unwrap();
    let expected = json!({
        "displayName": "john_doe",
        "status": "active",
        "upperName": "JOHN DOE"
    });

    assert_eq!(result, expected);
}
