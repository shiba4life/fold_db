use fold_db::FoldDB;
use fold_db::schema::Schema;
use fold_db::schema::types::{Query, Mutation};
use fold_db::schema::types::fields::SchemaField;
use fold_db::permissions::types::policy::{PermissionsPolicy, ExplicitCounts, TrustDistance};
use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::fees::payment_config::SchemaPaymentConfig;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(
        1.0,
        TrustDistanceScaling::None,
        None,
    ).unwrap()
}

#[path = "test_utils/mod.rs"]
mod test_utils;
use test_utils::{cleanup_test_db, get_test_db_path, cleanup_tmp_dir};

fn setup_test_db() -> (FoldDB, String) {
    let db_path = get_test_db_path();
    let db = FoldDB::new(&db_path).expect("Failed to create test database");
    (db, db_path)
}

// Clean up tmp directory after all tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cleanup() {
        cleanup_tmp_dir();
    }
}

#[test]
fn test_permission_based_access() {
    let (mut db, db_path) = setup_test_db();
    let owner_key = "owner_key".to_string();
    let reader_key = "reader_key".to_string();
    let unauthorized_key = "unauthorized_key".to_string();

    // Create schema with different permission levels for fields
    let mut fields = HashMap::new();
    
    // Public field - anyone can read, only owner can write
    let mut public_write_counts = HashMap::new();
    public_write_counts.insert(owner_key.clone(), 1);
    fields.insert(
        "public_field".to_string(),
        SchemaField {
            ref_atom_uuid: Uuid::new_v4().to_string(),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(5), // Very permissive read
                write_policy: TrustDistance::Distance(0), // Restrictive write
                explicit_read_policy: None,
                explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: public_write_counts }),
            },
            payment_config: create_default_payment_config(),
        }
    );

    // Protected field - trusted users can read, explicit users can write
    let mut protected_read_counts = HashMap::new();
    protected_read_counts.insert(reader_key.clone(), 1);
    let mut protected_write_counts = HashMap::new();
    protected_write_counts.insert(owner_key.clone(), 1);
    fields.insert(
        "protected_field".to_string(),
        SchemaField {
            ref_atom_uuid: Uuid::new_v4().to_string(),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(2),
                write_policy: TrustDistance::Distance(0),
                explicit_read_policy: Some(ExplicitCounts { counts_by_pub_key: protected_read_counts }),
                explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: protected_write_counts }),
            },
            payment_config: create_default_payment_config(),
        }
    );

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        transforms: Vec::new(),
        payment_config: SchemaPaymentConfig::default(),
    };

    // Load and allow schema
    db.load_schema(schema).expect("Failed to load schema");
    db.allow_schema("test_schema").expect("Failed to allow schema");

    // Test writing with owner key
    let owner_mutation = Mutation {
        schema_name: "test_schema".to_string(),
        fields_and_values: {
            let mut map = HashMap::new();
            map.insert("public_field".to_string(), json!("public value"));
            map.insert("protected_field".to_string(), json!("protected value"));
            map
        },
        pub_key: owner_key.clone(),
        trust_distance: 1,
    };
    assert!(db.write_schema(owner_mutation).is_ok());

    // Test reading with different keys and trust distances
    
    // Reader with explicit permission can read protected field
    let reader_query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["public_field".to_string(), "protected_field".to_string()],
        pub_key: reader_key.clone(),
        trust_distance: 3,
    };
    let reader_results = db.query_schema(reader_query.clone());
    let reader_results: HashMap<String, _> = reader_query.fields.iter().cloned()
        .zip(reader_results.into_iter())
        .collect();
    
    assert!(reader_results.get("public_field").unwrap().is_ok()); // Can read public field
    assert!(reader_results.get("protected_field").unwrap().is_ok()); // Can read protected field due to explicit permission

    // Unauthorized user can read public field but not protected field
    let unauth_query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["public_field".to_string(), "protected_field".to_string()],
        pub_key: unauthorized_key.clone(),
        trust_distance: 3,
    };
    let unauth_results = db.query_schema(unauth_query.clone());
    let unauth_results: HashMap<String, _> = unauth_query.fields.iter().cloned()
        .zip(unauth_results.into_iter())
        .collect();
    
    assert!(unauth_results.get("public_field").unwrap().is_ok()); // Can read public field
    assert!(unauth_results.get("protected_field").unwrap().is_err()); // Cannot read protected field

    // Test unauthorized write attempt
    let unauth_mutation = Mutation {
        schema_name: "test_schema".to_string(),
        fields_and_values: {
            let mut map = HashMap::new();
            map.insert("public_field".to_string(), json!("unauthorized write"));
            map
        },
        pub_key: unauthorized_key,
        trust_distance: 1,
    };
    assert!(db.write_schema(unauth_mutation).is_err());

    cleanup_test_db(&db_path);
}

#[test]
fn test_schema_versioning_with_permissions() {
    let (mut db, db_path) = setup_test_db();
    let owner_key = "owner_key".to_string();
    let reader_key = "reader_key".to_string();

    // Create schema with versioned field
    let field_uuid = Uuid::new_v4().to_string();
    let mut write_counts = HashMap::new();
    write_counts.insert(owner_key.clone(), 1);

    let mut fields = HashMap::new();
    fields.insert(
        "versioned_field".to_string(),
        SchemaField {
            ref_atom_uuid: field_uuid.clone(),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(2), // Moderate read restriction
                write_policy: TrustDistance::Distance(0), // Only explicit writers
                explicit_read_policy: None,
                explicit_write_policy: Some(ExplicitCounts { counts_by_pub_key: write_counts }),
            },
            payment_config: create_default_payment_config(),
        }
    );

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        transforms: Vec::new(),
        payment_config: SchemaPaymentConfig::default(),
    };

    db.load_schema(schema).expect("Failed to load schema");
    db.allow_schema("test_schema").expect("Failed to allow schema");

    // Create multiple versions with owner key
    for i in 1..=3 {
        let mutation = Mutation {
            schema_name: "test_schema".to_string(),
            fields_and_values: {
                let mut map = HashMap::new();
                map.insert("versioned_field".to_string(), json!(format!("version {}", i)));
                map
            },
            pub_key: owner_key.clone(),
            trust_distance: 1,
        };
        assert!(db.write_schema(mutation).is_ok());
    }

    // Verify history access with appropriate trust distance
    let history = db.get_atom_history(&field_uuid).expect("Failed to get history");
    assert_eq!(history.len(), 3);

    // Test reading latest version with different trust distances
    let trusted_query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["versioned_field".to_string()],
        pub_key: reader_key.clone(),
        trust_distance: 1, // Within trust distance
    };
    let trusted_results = db.query_schema(trusted_query.clone());
    let trusted_results: HashMap<String, _> = trusted_query.fields.iter().cloned()
        .zip(trusted_results.into_iter())
        .collect();
    
    let versioned_result = trusted_results.get("versioned_field").unwrap();
    assert!(versioned_result.is_ok());
    assert_eq!(versioned_result.as_ref().unwrap(), &json!("version 3"));

    let untrusted_query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["versioned_field".to_string()],
        pub_key: reader_key,
        trust_distance: 3, // Beyond trust distance
    };
    let untrusted_results = db.query_schema(untrusted_query.clone());
    let untrusted_results: HashMap<String, _> = untrusted_query.fields.iter().cloned()
        .zip(untrusted_results.into_iter())
        .collect();
    
    assert!(untrusted_results.get("versioned_field").unwrap().is_err());

    cleanup_test_db(&db_path);
}
