use fold_db::schema::{SchemaMapper, MappingRule, parse_mapping_dsl, InternalSchema};
use serde_json::{json, Value};
use std::collections::HashMap;

#[test]
fn test_schema_mapper_with_multiple_sources() {
    // Test DSL parsing
    let dsl = r#"
        # Map user profile fields
        RENAME username TO displayName
        DROP privateEmail
        ADD accountStatus "active"
        MAP fullName TO upperName USING to_uppercase
    "#;

    let rules = parse_mapping_dsl(dsl).unwrap();
    
    // Create mapper
    let mapper = SchemaMapper::new(
        vec!["user_profile".to_string(), "legacy_profile".to_string()],
        "public_profile".to_string(),
        rules,
    );

    // Set up test data
    let mut sources = HashMap::new();
    sources.insert("user_profile".to_string(), json!({
        "username": "johndoe",
        "privateEmail": "john@example.com",
        "fullName": "John Doe",
        "bio": "Software engineer"
    }));
    sources.insert("legacy_profile".to_string(), json!({
        "username": "old_john",
        "privateEmail": "john.old@example.com",
        "fullName": "John Old",
        "bio": "Legacy bio"
    }));

    // Apply mapping
    let result = mapper.apply(&sources).unwrap();

    // Verify results - only explicitly mapped fields should be included
    let expected = json!({
        "displayName": "johndoe",
        "accountStatus": "active",
        "upperName": "JOHN DOE"
    });

    assert_eq!(result, expected);
}

#[test]
fn test_schema_mapper_with_missing_source() {
    let rules = vec![
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "displayName".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["profile".to_string(), "missing_source".to_string()],
        "target".to_string(),
        rules,
    );

    let mut sources = HashMap::new();
    sources.insert("profile".to_string(), json!({
        "name": "John Doe"
    }));

    // Should still work even with missing source
    let result = mapper.apply(&sources).unwrap();
    assert_eq!(result, json!({
        "displayName": "John Doe"
    }));
}

#[test]
fn test_schema_mapper_with_invalid_data() {
    let rules = vec![
        MappingRule::Rename {
            source_field: "name".to_string(),
            target_field: "displayName".to_string(),
        },
    ];

    let mapper = SchemaMapper::new(
        vec!["profile".to_string()],
        "target".to_string(),
        rules,
    );

    let mut sources = HashMap::new();
    sources.insert("profile".to_string(), Value::String("invalid".to_string()));

    // Should return error for invalid data
    assert!(mapper.apply(&sources).is_err());
}

#[test]
fn test_schema_mapper_dsl_parsing() {
    // Test invalid DSL syntax
    let invalid_dsl = "INVALID command";
    assert!(parse_mapping_dsl(invalid_dsl).is_err());

    // Test invalid RENAME syntax
    let invalid_rename = "RENAME field";
    assert!(parse_mapping_dsl(invalid_rename).is_err());

    // Test invalid MAP syntax
    let invalid_map = "MAP field";
    assert!(parse_mapping_dsl(invalid_map).is_err());

    // Test valid complex DSL
    let valid_dsl = r#"
        # Comments are ignored
        RENAME oldField TO newField
        DROP secretField
        ADD status "active"
        MAP name TO upperName USING to_uppercase
        
        # Empty lines are ignored
        
        MAP description TO lowerDesc USING to_lowercase
    "#;

    let rules = parse_mapping_dsl(valid_dsl).unwrap();
    assert_eq!(rules.len(), 5);
}

#[test]
fn test_schema_mapping_propagation() {
    use fold_db::setup;
    use std::time::{SystemTime, UNIX_EPOCH};
    use serde_json::json;

    // Create tmp directory if it doesn't exist
    std::fs::create_dir_all("tmp").unwrap();

    // Initialize test database
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let db_path = format!("tmp/fold_db_mapper_test_{}", timestamp);
    let fold_db = setup::initialize_database_with_path(&db_path).unwrap();

    // Create initial atoms and atom refs for fields
    let name_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let name_aref = fold_db.create_atom_ref(&name_atom).unwrap();

    let email_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let email_aref = fold_db.create_atom_ref(&email_atom).unwrap();

    let interests_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let interests_aref = fold_db.create_atom_ref(&interests_atom).unwrap();

    // 1. Create and populate social media schema
    let mut social_schema = InternalSchema::new();
    // Initialize fields with valid atom refs
    social_schema.fields.insert("name".to_string(), name_aref);
    social_schema.fields.insert("email".to_string(), email_aref);
    social_schema.fields.insert("interests".to_string(), interests_aref.clone());
    fold_db.load_schema("social", social_schema).unwrap();

    // Populate fields
    fold_db.update_field_value("social", "name", json!("John Doe"), "test".to_string()).unwrap();
    fold_db.update_field_value("social", "email", json!("john@social.com"), "test".to_string()).unwrap();
    fold_db.update_field_value("social", "interests", json!(["coding", "music"]), "test".to_string()).unwrap();

    // 2. Create work schema and map from social
    let mut work_schema = InternalSchema::new();
    // Create atoms and refs for work schema fields
    let employee_name_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let employee_name_aref = fold_db.create_atom_ref(&employee_name_atom).unwrap();

    let department_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let department_aref = fold_db.create_atom_ref(&department_atom).unwrap();

    let work_email_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let work_email_aref = fold_db.create_atom_ref(&work_email_atom).unwrap();

    // Initialize work schema fields
    work_schema.fields.insert("employee_name".to_string(), employee_name_aref);
    work_schema.fields.insert("department".to_string(), department_aref);
    work_schema.fields.insert("work_email".to_string(), work_email_aref);
    work_schema.fields.insert("interests".to_string(), interests_aref.clone());
    fold_db.load_schema("work", work_schema).unwrap();
    
    // Map social schema to work schema
    let work_mapping_dsl = r#"
        RENAME name TO employee_name
        ADD department "Engineering"
        MAP email TO work_email USING to_lowercase
        RENAME interests TO interests
    "#;
    
    fold_db.schema_manager.load_mapper(
        "work",
        vec!["social".to_string()],
        work_mapping_dsl,
    ).unwrap();

    // Apply work mapping
    let mut social_data = HashMap::new();
    social_data.insert("social".to_string(), json!({
        "name": "John Doe",
        "email": "john@social.com",
        "interests": ["coding", "music"]
    }));

    let work_result = fold_db.schema_manager.apply_mapper("work", &social_data).unwrap();
    
    // Update work schema fields with mapped values
    fold_db.update_field_value("work", "employee_name", json!("John Doe"), "test".to_string()).unwrap();
    fold_db.update_field_value("work", "department", json!("Engineering"), "test".to_string()).unwrap();
    fold_db.update_field_value("work", "work_email", json!("john@social.com"), "test".to_string()).unwrap();
    fold_db.update_field_value("work", "interests", json!(["coding", "music"]), "test".to_string()).unwrap();
    
    // Verify work schema mapping
    // Verify work schema mapping result
    assert_eq!(work_result, json!({
        "employee_name": "John Doe",
        "department": "Engineering",
        "work_email": "john@social.com",
        "interests": ["coding", "music"]
    }));

    // Verify actual field values in the database
    let employee_name = fold_db.get_field_value("work", "employee_name").unwrap();
    let department = fold_db.get_field_value("work", "department").unwrap();
    let work_email = fold_db.get_field_value("work", "work_email").unwrap();

    assert_eq!(employee_name, json!("John Doe"));
    assert_eq!(department, json!("Engineering"));
    assert_eq!(work_email, json!("john@social.com"));

    // 3. Create medical schema and map from both social and work
    let mut medical_schema = InternalSchema::new();
    // Create atoms and refs for medical schema fields
    let patient_name_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let patient_name_aref = fold_db.create_atom_ref(&patient_name_atom).unwrap();

    let facility_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let facility_aref = fold_db.create_atom_ref(&facility_atom).unwrap();

    let record_type_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let record_type_aref = fold_db.create_atom_ref(&record_type_atom).unwrap();

    let medical_interests_atom = fold_db.create_atom(
        json!("").to_string(),
        "field_init".to_string(),
        "test".to_string(),
        None,
    ).unwrap();
    let medical_interests_aref = fold_db.create_atom_ref(&medical_interests_atom).unwrap();

    // Initialize medical schema fields
    medical_schema.fields.insert("patient_name".to_string(), patient_name_aref);
    medical_schema.fields.insert("facility".to_string(), facility_aref);
    medical_schema.fields.insert("record_type".to_string(), record_type_aref);
    medical_schema.fields.insert("interests".to_string(), medical_interests_aref);
    fold_db.load_schema("medical", medical_schema).unwrap();

    // Map both social and work schemas to medical schema
    let medical_mapping_dsl = r#"
        RENAME employee_name TO patient_name
        DROP work_email
        ADD record_type "Employee Health Record"
        MAP department TO facility USING to_uppercase
        RENAME interests TO interests
    "#;

    fold_db.schema_manager.load_mapper(
        "medical",
        vec!["social".to_string(), "work".to_string()],  // Changed order to prioritize social schema fields
        medical_mapping_dsl,
    ).unwrap();

    // Apply medical mapping
    let mut combined_data = HashMap::new();
    combined_data.insert("social".to_string(), social_data.get("social").unwrap().clone());
    combined_data.insert("work".to_string(), work_result);  // Insert work data after social to maintain correct precedence

    let medical_result = fold_db.schema_manager.apply_mapper("medical", &combined_data).unwrap();

    // Update medical schema fields with mapped values
    fold_db.update_field_value("medical", "patient_name", json!("John Doe"), "test".to_string()).unwrap();
    fold_db.update_field_value("medical", "facility", json!("ENGINEERING"), "test".to_string()).unwrap();
    fold_db.update_field_value("medical", "record_type", json!("Employee Health Record"), "test".to_string()).unwrap();
    fold_db.update_field_value("medical", "interests", json!(["coding", "music"]), "test".to_string()).unwrap();

    // Verify medical schema mapping
    // Verify medical schema mapping result
    assert_eq!(medical_result, json!({
        "patient_name": "John Doe",
        "facility": "ENGINEERING",
        "record_type": "Employee Health Record",
        "interests": ["coding", "music"]
    }));

    // Verify actual field values in the database
    let patient_name = fold_db.get_field_value("medical", "patient_name").unwrap();
    let facility = fold_db.get_field_value("medical", "facility").unwrap();
    let record_type = fold_db.get_field_value("medical", "record_type").unwrap();
    let medical_interests = fold_db.get_field_value("medical", "interests").unwrap();

    assert_eq!(patient_name, json!("John Doe"));
    assert_eq!(facility, json!("ENGINEERING"));
    assert_eq!(record_type, json!("Employee Health Record"));
    assert_eq!(medical_interests, json!(["coding", "music"]));

    // 4. Test update propagation
    // Update name in social schema and verify
    fold_db.update_field_value("social", "name", json!("John Smith"), "test".to_string()).unwrap();
    let updated_name = fold_db.get_field_value("social", "name").unwrap();
    assert_eq!(updated_name, json!("John Smith"));

    // Verify other social fields remain unchanged
    let unchanged_email = fold_db.get_field_value("social", "email").unwrap();
    let unchanged_interests = fold_db.get_field_value("social", "interests").unwrap();
    assert_eq!(unchanged_email, json!("john@social.com"));
    assert_eq!(unchanged_interests, json!(["coding", "music"]));

    // Get updated social data for mapping
    let updated_social = json!({
        "name": "John Smith",
        "email": "john@social.com",
        "interests": ["coding", "music"]
    });

    // Verify work schema update
    let mut updated_social_data = HashMap::new();
    updated_social_data.insert("social".to_string(), updated_social.clone());
    
    let updated_work = fold_db.schema_manager.apply_mapper("work", &updated_social_data).unwrap();
    
    // Update work schema with new mapped values
    fold_db.update_field_value("work", "employee_name", json!("John Smith"), "test".to_string()).unwrap();
    fold_db.update_field_value("work", "work_email", json!("john@social.com"), "test".to_string()).unwrap();
    fold_db.update_field_value("work", "interests", json!(["coding", "music"]), "test".to_string()).unwrap();
    
    // Verify work schema update result
    assert_eq!(updated_work, json!({
        "employee_name": "John Smith",
        "department": "Engineering",
        "work_email": "john@social.com",
        "interests": ["coding", "music"]
    }));

    // Verify actual updated field value in the database
    let updated_employee_name = fold_db.get_field_value("work", "employee_name").unwrap();
    assert_eq!(updated_employee_name, json!("John Smith"));

    // Verify work schema's unchanged fields
    let unchanged_department = fold_db.get_field_value("work", "department").unwrap();
    let unchanged_interests = fold_db.get_field_value("work", "interests").unwrap();
    assert_eq!(unchanged_department, json!("Engineering"));
    assert_eq!(unchanged_interests, json!(["coding", "music"]));

    // Verify medical schema update
    let mut updated_combined = HashMap::new();
    updated_combined.insert("social".to_string(), updated_social);
    updated_combined.insert("work".to_string(), updated_work);

    let updated_medical = fold_db.schema_manager.apply_mapper("medical", &updated_combined).unwrap();
    
    // Update medical schema with new mapped values
    fold_db.update_field_value("medical", "patient_name", json!("John Smith"), "test".to_string()).unwrap();
    fold_db.update_field_value("medical", "interests", json!(["coding", "music"]), "test".to_string()).unwrap();

    // Verify unchanged medical fields
    let unchanged_facility = fold_db.get_field_value("medical", "facility").unwrap();
    let unchanged_record_type = fold_db.get_field_value("medical", "record_type").unwrap();
    assert_eq!(unchanged_facility, json!("ENGINEERING"));
    assert_eq!(unchanged_record_type, json!("Employee Health Record"));
    
    // Verify medical schema update result
    assert_eq!(updated_medical, json!({
        "patient_name": "John Smith",
        "facility": "ENGINEERING",
        "record_type": "Employee Health Record",
        "interests": ["coding", "music"]
    }));

    // Verify actual updated field value in the database
    let updated_patient_name = fold_db.get_field_value("medical", "patient_name").unwrap();
    assert_eq!(updated_patient_name, json!("John Smith"));

    // Cleanup
    std::fs::remove_dir_all(db_path).unwrap();
}
