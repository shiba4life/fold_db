//! Comprehensive tests for backward compatibility with unencrypted data
//!
//! This test suite validates all aspects of PBI 9 Task 9-6:
//! - Seamless backward compatibility with existing unencrypted data
//! - Migration utilities for detecting and migrating unencrypted data
//! - Mixed environment handling (encrypted + unencrypted data)
//! - Enhanced error handling for backward compatibility scenarios
//! - Multiple migration modes (gradual, full, read-only)

use datafold::crypto::generate_master_keypair;
use datafold::db_operations::encryption_wrapper::contexts;
use datafold::db_operations::{DbOperations, EncryptionWrapper, MigrationConfig, MigrationMode};
use datafold::schema::SchemaError;
use serde_json::json;
use tempfile::tempdir;

/// Create a test database with unencrypted data
fn create_legacy_database() -> (sled::Db, DbOperations) {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db.clone()).unwrap();

    // Store some unencrypted data directly
    db_ops
        .store_item(
            "legacy_user_1",
            &json!({
                "name": "Alice",
                "email": "alice@example.com",
                "age": 30
            }),
        )
        .unwrap();

    db_ops
        .store_item(
            "legacy_user_2",
            &json!({
                "name": "Bob",
                "email": "bob@example.com",
                "age": 25
            }),
        )
        .unwrap();

    db_ops
        .store_item(
            "legacy_config",
            &json!({
                "version": "1.0.0",
                "settings": {"theme": "dark", "language": "en"}
            }),
        )
        .unwrap();

    (db, db_ops)
}

/// Create an encryption wrapper for testing
fn create_encryption_wrapper_with_mode(
    db_ops: DbOperations,
    mode: MigrationMode,
) -> EncryptionWrapper {
    let master_keypair = generate_master_keypair().unwrap();
    EncryptionWrapper::with_migration_mode(db_ops, &master_keypair, mode).unwrap()
}

#[test]
fn test_read_only_compatibility_mode() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::ReadOnlyCompatibility);

    // Should be able to read existing unencrypted data
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");

    // Should not encrypt new data in read-only mode
    assert!(!wrapper.is_encryption_enabled());

    // Storing new data should remain unencrypted
    wrapper
        .store_encrypted_item("new_user", &json!({"name": "Charlie"}), contexts::ATOM_DATA)
        .unwrap();

    // Verify it was stored unencrypted by reading directly from db
    let raw_data = wrapper.db_ops().db().get("new_user").unwrap().unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&raw_data).unwrap();
    assert_eq!(parsed["name"], "Charlie");
}

#[test]
fn test_gradual_migration_mode() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Should be able to read existing unencrypted data
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");

    // Should encrypt new data in gradual mode
    assert!(wrapper.is_encryption_enabled());

    // Store new data - should be encrypted
    wrapper
        .store_encrypted_item("new_user", &json!({"name": "Charlie"}), contexts::ATOM_DATA)
        .unwrap();

    // Verify it was stored encrypted by checking magic bytes
    let raw_data = wrapper.db_ops().db().get("new_user").unwrap().unwrap();
    assert!(raw_data.starts_with(b"DF_ENC")); // Magic bytes for encrypted data

    // Should still be able to read the new encrypted data
    let new_user: serde_json::Value = wrapper
        .get_encrypted_item("new_user", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(new_user["name"], "Charlie");
}

#[test]
fn test_full_migration_mode() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Full);

    // Should be able to read existing unencrypted data before migration
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");

    // Perform full migration
    let config = MigrationConfig {
        mode: MigrationMode::Full,
        batch_size: 50,
        verify_integrity: true,
        backup_before_migration: false,
        target_context: contexts::ATOM_DATA.to_string(),
    };

    let migrated_count = wrapper.perform_batch_migration(&config).unwrap();
    assert_eq!(migrated_count, 3); // Should have migrated 3 legacy items

    // Verify all data is now encrypted
    let migration_status = wrapper.get_migration_status().unwrap();
    assert!(migration_status.is_fully_encrypted());
    assert_eq!(migration_status.unencrypted_items, 0);

    // Should still be able to read the migrated data
    let user1_after: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1_after["name"], "Alice");
}

#[test]
fn test_mixed_environment_detection() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Add some encrypted data
    wrapper
        .store_encrypted_item(
            "encrypted_user",
            &json!({"name": "Dave"}),
            contexts::ATOM_DATA,
        )
        .unwrap();

    // Check migration status
    let status = wrapper.get_migration_status().unwrap();
    assert!(status.is_mixed_environment());
    assert_eq!(status.encrypted_items, 1);
    assert_eq!(status.unencrypted_items, 3);
    assert_eq!(status.total_items, 4);

    // Test encryption percentage calculation
    assert_eq!(status.encryption_percentage(), 25.0);
}

#[test]
fn test_unencrypted_data_detection() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Detect unencrypted data
    let unencrypted_keys = wrapper.detect_unencrypted_data().unwrap();
    assert_eq!(unencrypted_keys.len(), 3);
    assert!(unencrypted_keys.contains(&"legacy_user_1".to_string()));
    assert!(unencrypted_keys.contains(&"legacy_user_2".to_string()));
    assert!(unencrypted_keys.contains(&"legacy_config".to_string()));

    // Add encrypted data
    wrapper
        .store_encrypted_item(
            "encrypted_item",
            &json!({"test": "data"}),
            contexts::ATOM_DATA,
        )
        .unwrap();

    // Should still detect the same unencrypted items
    let unencrypted_keys_after = wrapper.detect_unencrypted_data().unwrap();
    assert_eq!(unencrypted_keys_after.len(), 3);
}

#[test]
fn test_data_format_validation() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Add some encrypted data
    wrapper
        .store_encrypted_item(
            "encrypted_user",
            &json!({"name": "Eve"}),
            contexts::ATOM_DATA,
        )
        .unwrap();

    // Add some invalid data directly to the database
    wrapper
        .db_ops()
        .db()
        .insert("invalid_data", b"not json at all")
        .unwrap();

    // Validate data format consistency
    let validation_stats = wrapper.validate_data_format_consistency().unwrap();

    assert_eq!(validation_stats.get("encrypted_valid"), Some(&1));
    assert_eq!(validation_stats.get("unencrypted_valid"), Some(&3));
    assert_eq!(validation_stats.get("invalid_format"), Some(&1));
    assert_eq!(validation_stats.get("total_contexts"), Some(&1));
    assert_eq!(
        validation_stats.get(&format!("context_{}", contexts::ATOM_DATA)),
        Some(&1)
    );
}

#[test]
fn test_enhanced_error_handling_read_only_mode() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::ReadOnlyCompatibility);

    // Add some corrupted data
    wrapper
        .db_ops()
        .db()
        .insert("corrupted_data", b"invalid json")
        .unwrap();

    // In read-only mode, should return None for corrupted data instead of error
    let result: Option<serde_json::Value> = wrapper
        .get_encrypted_item("corrupted_data", contexts::ATOM_DATA)
        .unwrap();
    assert!(result.is_none());

    // Should still be able to read valid data
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");
}

#[test]
fn test_enhanced_error_handling_strict_mode() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Add some corrupted data
    wrapper
        .db_ops()
        .db()
        .insert("corrupted_data", b"invalid json")
        .unwrap();

    // In non-read-only mode, should return error for corrupted data
    let result: Result<Option<serde_json::Value>, SchemaError> =
        wrapper.get_encrypted_item("corrupted_data", contexts::ATOM_DATA);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Failed to deserialize data"));
}

#[test]
fn test_migration_mode_switching() {
    let (_, db_ops) = create_legacy_database();
    let mut wrapper =
        create_encryption_wrapper_with_mode(db_ops, MigrationMode::ReadOnlyCompatibility);

    // Initially in read-only mode
    assert_eq!(
        wrapper.migration_mode(),
        MigrationMode::ReadOnlyCompatibility
    );
    assert!(!wrapper.is_encryption_enabled());

    // Switch to gradual mode
    wrapper.set_migration_mode(MigrationMode::Gradual);
    assert_eq!(wrapper.migration_mode(), MigrationMode::Gradual);
    assert!(wrapper.is_encryption_enabled());

    // Switch to full mode
    wrapper.set_migration_mode(MigrationMode::Full);
    assert_eq!(wrapper.migration_mode(), MigrationMode::Full);
    assert!(wrapper.is_encryption_enabled());
}

#[test]
fn test_batch_migration_with_verification() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Full);

    // Add some additional data to test batching
    for i in 0..10 {
        wrapper
            .db_ops()
            .store_item(&format!("batch_item_{}", i), &json!({"id": i}))
            .unwrap();
    }

    let config = MigrationConfig {
        mode: MigrationMode::Full,
        batch_size: 5, // Small batch size to test batching
        verify_integrity: true,
        backup_before_migration: false,
        target_context: contexts::ATOM_DATA.to_string(),
    };

    let migrated_count = wrapper.perform_batch_migration(&config).unwrap();
    assert_eq!(migrated_count, 13); // 3 legacy + 10 batch items

    // Verify all data is encrypted and accessible
    let status = wrapper.get_migration_status().unwrap();
    assert!(status.is_fully_encrypted());

    // Spot check some migrated data
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");

    let batch_item: serde_json::Value = wrapper
        .get_encrypted_item("batch_item_5", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(batch_item["id"], 5);
}

#[test]
fn test_migration_with_different_contexts() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Full);

    // Migrate with schema context
    let config = MigrationConfig {
        mode: MigrationMode::Full,
        batch_size: 100,
        verify_integrity: true,
        backup_before_migration: false,
        target_context: contexts::SCHEMA_DATA.to_string(),
    };

    let migrated_count = wrapper.perform_batch_migration(&config).unwrap();
    assert_eq!(migrated_count, 3);

    // Should be able to read with the correct context
    let user1: serde_json::Value = wrapper
        .get_encrypted_item("legacy_user_1", contexts::SCHEMA_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(user1["name"], "Alice");

    // Should fail with wrong context
    let result: Result<Option<serde_json::Value>, SchemaError> =
        wrapper.get_encrypted_item("legacy_user_1", contexts::ATOM_DATA);
    assert!(result.is_err());
}

#[test]
fn test_tree_operations_backward_compatibility() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store unencrypted data in a tree
    db_ops
        .store_in_tree(
            db_ops.metadata_tree(),
            "legacy_metadata",
            &json!({"version": "1.0"}),
        )
        .unwrap();

    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Should be able to read unencrypted data from tree
    let metadata: serde_json::Value = wrapper
        .get_encrypted_from_tree(
            wrapper.db_ops().metadata_tree(),
            "legacy_metadata",
            contexts::METADATA,
        )
        .unwrap()
        .unwrap();
    assert_eq!(metadata["version"], "1.0");

    // Store new encrypted data in tree
    wrapper
        .store_encrypted_in_tree(
            wrapper.db_ops().metadata_tree(),
            "new_metadata",
            &json!({"version": "2.0"}),
            contexts::METADATA,
        )
        .unwrap();

    // Should be able to read encrypted data from tree
    let new_metadata: serde_json::Value = wrapper
        .get_encrypted_from_tree(
            wrapper.db_ops().metadata_tree(),
            "new_metadata",
            contexts::METADATA,
        )
        .unwrap()
        .unwrap();
    assert_eq!(new_metadata["version"], "2.0");
}

#[test]
fn test_comprehensive_encryption_stats() {
    let (_, db_ops) = create_legacy_database();
    let wrapper = create_encryption_wrapper_with_mode(db_ops, MigrationMode::Gradual);

    // Add some encrypted data
    wrapper
        .store_encrypted_item("encrypted_1", &json!({"test": 1}), contexts::ATOM_DATA)
        .unwrap();
    wrapper
        .store_encrypted_item("encrypted_2", &json!({"test": 2}), contexts::SCHEMA_DATA)
        .unwrap();

    let stats = wrapper.get_encryption_stats().unwrap();

    assert_eq!(stats.get("encrypted_items"), Some(&2));
    assert_eq!(stats.get("unencrypted_items"), Some(&3));
    assert_eq!(stats.get("total_items"), Some(&5));
    assert_eq!(stats.get("encryption_enabled"), Some(&1));
    assert_eq!(stats.get("is_mixed_environment"), Some(&1));
    assert_eq!(stats.get("is_fully_encrypted"), Some(&0));
    assert_eq!(
        stats.get("migration_mode"),
        Some(&(MigrationMode::Gradual as u64))
    );
}

#[test]
fn test_full_migration_mode_requires_encryption() {
    let (_, db_ops) = create_legacy_database();

    // Create wrapper without encryption capabilities
    let wrapper = EncryptionWrapper::without_encryption(db_ops);

    let config = MigrationConfig {
        mode: MigrationMode::Full,
        batch_size: 100,
        verify_integrity: true,
        backup_before_migration: false,
        target_context: contexts::ATOM_DATA.to_string(),
    };

    // Should fail because encryption is disabled
    let result = wrapper.perform_batch_migration(&config);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("read-only compatibility mode"));
}
