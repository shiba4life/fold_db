use fold_db::schema::{Schema, SchemaManager};
use fold_db::schema::types::fields::SchemaField;
use fold_db::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::schema::mapper::{SchemaMapper, MappingRule};
use uuid::Uuid;

fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(
        1.0,
        TrustDistanceScaling::None,
        None,
    ).unwrap()
}

#[test]
fn test_schema_manager_apply_mapping_rename() {
    let manager = SchemaManager::new();
    
    // Create source schema
    let mut source_schema = Schema::new("source_schema".to_string());
    let source_field = SchemaField {
        ref_atom_uuid: "test-uuid-1".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    source_schema.add_field("old_name".to_string(), source_field);
    
    // Create target schema
    let mut target_schema = Schema::new("target_schema".to_string());
    let target_field = SchemaField {
        ref_atom_uuid: "test-uuid-2".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    target_schema.add_field("new_name".to_string(), target_field);
    
    // Add mapper to source schema
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![MappingRule::Rename {
            source_field: "old_name".to_string(),
            target_field: "new_name".to_string(),
        }]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema.clone()).unwrap();
    
    // Apply mapping
    manager.apply_schema_mappers(&source_schema).unwrap();
    
    // Verify mapping
    let updated_target = manager.get_schema("target_schema").unwrap().unwrap();
    assert_eq!(
        updated_target.fields.get("new_name").unwrap().ref_atom_uuid,
        "test-uuid-1"
    );
}

#[test]
fn test_schema_manager_apply_mapping_drop() {
    let manager = SchemaManager::new();
    
    // Create source schema with mapper
    let mut source_schema = Schema::new("source_schema".to_string());
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![MappingRule::Drop {
            field: "to_drop".to_string(),
        }]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Create target schema with field to drop
    let mut target_schema = Schema::new("target_schema".to_string());
    let field = SchemaField {
        ref_atom_uuid: "test-uuid".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    target_schema.add_field("to_drop".to_string(), field);
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema).unwrap();
    
    // Apply mapping
    manager.apply_schema_mappers(&source_schema).unwrap();
    
    // Verify field was dropped
    let updated_target = manager.get_schema("target_schema").unwrap().unwrap();
    assert!(!updated_target.fields.contains_key("to_drop"));
}

#[test]
fn test_schema_manager_apply_mapping_map() {
    let manager = SchemaManager::new();
    
    // Create source schema
    let mut source_schema = Schema::new("source_schema".to_string());
    let source_field = SchemaField {
        ref_atom_uuid: "test-uuid-1".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    source_schema.add_field("source_field".to_string(), source_field);
    
    // Create target schema
    let mut target_schema = Schema::new("target_schema".to_string());
    let target_field = SchemaField {
        ref_atom_uuid: "test-uuid-2".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    target_schema.add_field("target_field".to_string(), target_field);
    
    // Add mapper to source schema
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![MappingRule::Map {
            source_field: "source_field".to_string(),
            target_field: "target_field".to_string(),
            function: Some("to_lowercase".to_string()),
        }]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema).unwrap();
    
    // Apply mapping
    manager.apply_schema_mappers(&source_schema).unwrap();
    
    // Verify mapping
    let updated_target = manager.get_schema("target_schema").unwrap().unwrap();
    assert_eq!(
        updated_target.fields.get("target_field").unwrap().ref_atom_uuid,
        "test-uuid-1"
    );
}

#[test]
fn test_schema_manager_apply_mapping_missing_schema() {
    let manager = SchemaManager::new();
    
    // Create source schema with mapper to non-existent target
    let mut source_schema = Schema::new("source_schema".to_string());
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "non_existent_schema".to_string(),
        vec![MappingRule::Map {
            source_field: "source_field".to_string(),
            target_field: "target_field".to_string(),
            function: None,
        }]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Load only source schema
    manager.load_schema(source_schema.clone()).unwrap();
    
    // Attempt to apply mapping should fail
    let result = manager.apply_schema_mappers(&source_schema);
    assert!(result.is_err());
}

#[test]
fn test_schema_manager_apply_mapping_invalid_field() {
    let manager = SchemaManager::new();
    
    // Create source schema with mapper referencing non-existent field
    let mut source_schema = Schema::new("source_schema".to_string());
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![MappingRule::Map {
            source_field: "non_existent_field".to_string(),
            target_field: "target_field".to_string(),
            function: None,
        }]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Create target schema
    let target_schema = Schema::new("target_schema".to_string());
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema).unwrap();
    
    // Attempt to apply mapping should fail due to missing field
    let result = manager.apply_schema_mappers(&source_schema);
    assert!(result.is_err());
}

#[test]
fn test_schema_creation() {
    let schema_name = "test_schema".to_string();
    let schema = Schema::new(schema_name.clone());
    
    assert_eq!(schema.name, schema_name);
    assert!(schema.fields.is_empty());
    assert!(schema.schema_mappers.is_empty());
}

#[test]
fn test_schema_field_management() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "test_field".to_string();
    let field = SchemaField {
        ref_atom_uuid: Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };

    // Add field
    schema.add_field(field_name.clone(), field.clone());
    
    // Verify field was added
    assert!(schema.fields.contains_key(&field_name));
    assert_eq!(schema.fields.get(&field_name).unwrap().ref_atom_uuid, field.ref_atom_uuid);
}

#[test]
fn test_schema_mapper_validation() {
    // Test empty rules
    let mapper = SchemaMapper::new(
        "source".to_string(),
        "target".to_string(),
        vec![]
    );
    assert_eq!(mapper.rules.len(), 0);

    // Test duplicate field mappings
    let mapper = SchemaMapper::new(
        "source".to_string(),
        "target".to_string(),
        vec![
            MappingRule::Map {
                source_field: "field1".to_string(),
                target_field: "field1".to_string(),
                function: None,
            },
            MappingRule::Map {
                source_field: "field1".to_string(),
                target_field: "field1".to_string(),
                function: None,
            }
        ]
    );
    assert_eq!(mapper.rules.len(), 2); // Duplicates are allowed at creation time
}

#[test]
fn test_schema_mapper_multiple_rules() {
    let manager = SchemaManager::new();
    
    // Create source schema
    let mut source_schema = Schema::new("source_schema".to_string());
    let field1 = SchemaField {
        ref_atom_uuid: "test-uuid-1".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    let field2 = SchemaField {
        ref_atom_uuid: "test-uuid-2".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    source_schema.add_field("field1".to_string(), field1);
    source_schema.add_field("old_name".to_string(), field2);
    
    // Create target schema
    let mut target_schema = Schema::new("target_schema".to_string());
    let target_field1 = SchemaField {
        ref_atom_uuid: "test-uuid-3".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    let target_field2 = SchemaField {
        ref_atom_uuid: "test-uuid-4".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    target_schema.add_field("field1".to_string(), target_field1);
    target_schema.add_field("new_name".to_string(), target_field2);
    
    // Add mapper with multiple rules
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![
            MappingRule::Map {
                source_field: "field1".to_string(),
                target_field: "field1".to_string(),
                function: None,
            },
            MappingRule::Rename {
                source_field: "old_name".to_string(),
                target_field: "new_name".to_string(),
            }
        ]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema).unwrap();
    
    // Apply mapping
    manager.apply_schema_mappers(&source_schema).unwrap();
    
    // Verify both mappings were applied
    let updated_target = manager.get_schema("target_schema").unwrap().unwrap();
    assert_eq!(
        updated_target.fields.get("field1").unwrap().ref_atom_uuid,
        "test-uuid-1"
    );
    assert_eq!(
        updated_target.fields.get("new_name").unwrap().ref_atom_uuid,
        "test-uuid-2"
    );
}

#[test]
fn test_schema_mapper_conflicting_rules() {
    let manager = SchemaManager::new();
    
    // Create source schema with conflicting rules
    let mut source_schema = Schema::new("source_schema".to_string());
    let field = SchemaField {
        ref_atom_uuid: "test-uuid-1".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    source_schema.add_field("field1".to_string(), field);
    
    // Create mapper with conflicting rules for same field
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![
            MappingRule::Map {
                source_field: "field1".to_string(),
                target_field: "field1".to_string(),
                function: None,
            },
            MappingRule::Drop {
                field: "field1".to_string(),
            }
        ]
    );
    source_schema.add_schema_mapper(mapper);
    
    // Create target schema
    let mut target_schema = Schema::new("target_schema".to_string());
    let target_field = SchemaField {
        ref_atom_uuid: "test-uuid-2".to_string(),
        permission_policy: PermissionsPolicy::default(),
        payment_config: create_default_payment_config(),
    };
    target_schema.add_field("field1".to_string(), target_field);
    
    // Load schemas
    manager.load_schema(source_schema.clone()).unwrap();
    manager.load_schema(target_schema).unwrap();
    
    // Apply mapping should fail due to conflicting rules
    let result = manager.apply_schema_mappers(&source_schema);
    assert!(result.is_err());
}

#[test]
fn test_schema_mapper_management() {
    let mut schema = Schema::new("test_schema".to_string());
    let mapper = SchemaMapper::new(
        "source_schema".to_string(),
        "target_schema".to_string(),
        vec![MappingRule::Map {
            source_field: "old_name".to_string(),
            target_field: "new_name".to_string(),
            function: None,
        }]
    );

    // Add mapper
    schema.add_schema_mapper(mapper.clone());
    
    // Verify mapper was added
    assert_eq!(schema.schema_mappers.len(), 1);
    assert_eq!(schema.schema_mappers[0].source_schema_name, "source_schema");
    assert_eq!(schema.schema_mappers[0].target_schema_name, "target_schema");
}

#[test]
fn test_schema_field_permissions() {
    let mut schema = Schema::new("test_schema".to_string());
    let field_name = "protected_field".to_string();
    
    // Create field with custom permissions
    let field = SchemaField {
        ref_atom_uuid: Uuid::new_v4().to_string(),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(2),
            TrustDistance::Distance(3)
        ),
        payment_config: create_default_payment_config(),
    };

    schema.add_field(field_name.clone(), field.clone());
    
    // Verify permissions
    let stored_field = schema.fields.get(&field_name).unwrap();
    match stored_field.permission_policy.read_policy {
        TrustDistance::Distance(d) => assert_eq!(d, 2),
        _ => panic!("Expected Distance variant"),
    }
    match stored_field.permission_policy.write_policy {
        TrustDistance::Distance(d) => assert_eq!(d, 3),
        _ => panic!("Expected Distance variant"),
    }
}

#[test]
fn test_schema_with_multiple_fields() {
    let mut schema = Schema::new("test_schema".to_string());
    
    // Add multiple fields with different permissions
    let fields = vec![
        ("public_field", PermissionsPolicy::default()),
        ("protected_field", PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(2)
        )),
        ("private_field", PermissionsPolicy::new(
            TrustDistance::Distance(3),
            TrustDistance::Distance(3)
        )),
    ];

    for (name, policy) in fields {
        schema.add_field(
            name.to_string(),
            SchemaField {
                ref_atom_uuid: Uuid::new_v4().to_string(),
                permission_policy: policy,
                payment_config: create_default_payment_config(),
            }
        );
    }

    // Verify all fields were added with correct permissions
    assert_eq!(schema.fields.len(), 3);
    match &schema.fields.get("public_field").unwrap().permission_policy.read_policy {
        TrustDistance::Distance(d) => assert_eq!(*d, 0),
        _ => panic!("Expected Distance variant"),
    }
    match &schema.fields.get("protected_field").unwrap().permission_policy.read_policy {
        TrustDistance::Distance(d) => assert_eq!(*d, 1),
        _ => panic!("Expected Distance variant"),
    }
    match &schema.fields.get("private_field").unwrap().permission_policy.read_policy {
        TrustDistance::Distance(d) => assert_eq!(*d, 3),
        _ => panic!("Expected Distance variant"),
    }
}
