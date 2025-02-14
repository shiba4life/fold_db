use fold_db::fees::payment_config::SchemaPaymentConfig;
use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::types::{Mutation, Query};
use fold_db::schema::Schema;
use fold_db::FoldDB;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

mod test_helpers;
use test_helpers::{cleanup_test_db, cleanup_tmp_dir, get_test_db_path};

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
fn test_schema_operations() {
    let (mut db, db_path) = setup_test_db();

    // Create test schema
    let mut fields = HashMap::new();
    let mut write_counts = HashMap::new();
    write_counts.insert("test_key".to_string(), 1);

    fields.insert(
        "name".to_string(),
        SchemaField {
            ref_atom_uuid: Some(Uuid::new_v4().to_string()),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(5),
                write_policy: TrustDistance::Distance(0),
                explicit_write_policy: Some(ExplicitCounts {
                    counts_by_pub_key: write_counts,
                }),
                explicit_read_policy: None,
            },
            payment_config: create_default_payment_config(),
        },
    );

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
    };

    // Test schema loading
    assert!(db.load_schema(schema.clone()).is_ok());
    assert!(db.allow_schema("test_schema").is_ok());

    // Test non-existent schema
    assert!(db.allow_schema("nonexistent").is_err());

    cleanup_test_db(&db_path);
}

#[test]
fn test_write_and_query() {
    let (mut db, db_path) = setup_test_db();

    // Setup schema
    let field_uuid = Uuid::new_v4().to_string();
    let mut fields = HashMap::new();
    let mut write_counts = HashMap::new();
    write_counts.insert("test_key".to_string(), 1);

    fields.insert(
        "test_field".to_string(),
        SchemaField {
            ref_atom_uuid: Some(field_uuid.clone()),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(5), // Allow reads with trust distance up to 5
                write_policy: TrustDistance::Distance(0),
                explicit_write_policy: Some(ExplicitCounts {
                    counts_by_pub_key: write_counts,
                }),
                explicit_read_policy: None,
            },
            payment_config: create_default_payment_config(),
        },
    );

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
    };

    db.load_schema(schema).expect("Failed to load schema");
    db.allow_schema("test_schema")
        .expect("Failed to allow schema");

    // Test write
    let mutation = Mutation {
        schema_name: "test_schema".to_string(),
        fields_and_values: {
            let mut map = HashMap::new();
            map.insert("test_field".to_string(), json!("test_value"));
            map
        },
        pub_key: "test_key".to_string(),
        trust_distance: 1,
    };

    assert!(db.write_schema(mutation).is_ok());

    // Test query
    let query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["test_field".to_string()],
        pub_key: "test_key".to_string(),
        trust_distance: 1,
    };

    let results = db.query_schema(query);
    assert_eq!(results.len(), 1);

    let result = &results[0];
    assert!(result.is_ok());
    assert_eq!(result.as_ref().unwrap(), &json!("test_value"));

    cleanup_test_db(&db_path);
}

#[test]
fn test_atom_history() {
    let (mut db, db_path) = setup_test_db();

    // Setup schema
    let field_uuid = Uuid::new_v4().to_string();
    let mut fields = HashMap::new();
    let mut write_counts = HashMap::new();
    write_counts.insert("test_key".to_string(), 1);

    fields.insert(
        "version_field".to_string(),
        SchemaField {
            ref_atom_uuid: Some(field_uuid.clone()),
            permission_policy: PermissionsPolicy {
                read_policy: TrustDistance::Distance(5),
                write_policy: TrustDistance::Distance(0),
                explicit_write_policy: Some(ExplicitCounts {
                    counts_by_pub_key: write_counts,
                }),
                explicit_read_policy: None,
            },
            payment_config: create_default_payment_config(),
        },
    );

    let schema = Schema {
        name: "test_schema".to_string(),
        fields,
        payment_config: SchemaPaymentConfig::default(),
    };

    db.load_schema(schema).expect("Failed to load schema");
    db.allow_schema("test_schema")
        .expect("Failed to allow schema");

    use std::thread;
    use std::time::Duration;

    // Write multiple versions with small delay between writes
    for i in 1..=3 {
        let mutation = Mutation {
            schema_name: "test_schema".to_string(),
            fields_and_values: {
                let mut map = HashMap::new();
                map.insert("version_field".to_string(), json!(i));
                map
            },
            pub_key: "test_key".to_string(),
            trust_distance: 1,
        };
        db.write_schema(mutation).expect("Failed to write");
        thread::sleep(Duration::from_millis(10)); // Small delay to ensure ordering
    }

    // Query latest
    let query = Query {
        schema_name: "test_schema".to_string(),
        fields: vec!["version_field".to_string()],
        pub_key: "test_key".to_string(),
        trust_distance: 1,
    };

    let results = db.query_schema(query);
    assert_eq!(results[0].as_ref().unwrap(), &json!(3));

    // Get history
    let history = db
        .get_atom_history(&field_uuid)
        .expect("Failed to get history");
    assert_eq!(history.len(), 3);

    // Check versions are in reverse chronological order
    for (i, atom) in history.iter().enumerate() {
        let version = 3 - i;
        assert_eq!(atom.content(), &json!(version));
    }

    cleanup_test_db(&db_path);
}
