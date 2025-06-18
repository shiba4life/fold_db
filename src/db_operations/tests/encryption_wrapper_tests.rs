//! Comprehensive tests for the database encryption wrapper
//!
//! This module contains all test functions for the encryption wrapper functionality,
//! organized by functional area to ensure comprehensive coverage of encryption,
//! migration, backward compatibility, and security features.

use crate::crypto::generate_master_keypair;
use crate::db_operations::{
    contexts, migration::{MigrationConfig, MigrationMode, MigrationStatus}, 
    EncryptionWrapper, DbOperations
};
use crate::schema::SchemaError;
use tempfile::tempdir;

/// Create a test encryption wrapper for testing purposes
fn create_test_encryption_wrapper() -> EncryptionWrapper {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let master_keypair = generate_master_keypair().unwrap();
    EncryptionWrapper::new(db_ops, &master_keypair).unwrap()
}

/// Test encryption wrapper creation and initialization
#[test]
fn test_encryption_wrapper_creation() {
    let wrapper = create_test_encryption_wrapper();
    assert!(wrapper.is_encryption_enabled());
    assert_eq!(wrapper.encryptors.len(), contexts::all_contexts().len());
}

/// Test basic store and retrieve operations with encryption
#[test]
fn test_store_and_retrieve_encrypted_item() {
    let wrapper = create_test_encryption_wrapper();

    let test_data = "test data for encryption";
    let test_key = "test_key";
    let context = contexts::ATOM_DATA;

    // Store encrypted
    wrapper
        .store_encrypted_item(test_key, &test_data, context)
        .unwrap();

    // Retrieve and verify
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, context)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

/// Test backward compatibility with existing unencrypted data
#[test]
fn test_backward_compatibility() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store unencrypted data using raw DbOperations
    let test_data = "unencrypted test data";
    let test_key = "legacy_key";
    db_ops.store_item(test_key, &test_data).unwrap();

    // Create encryption wrapper and try to read the unencrypted data
    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    // Should be able to read unencrypted data
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

/// Test different encryption contexts and their isolation
#[test]
fn test_different_encryption_contexts() {
    let wrapper = create_test_encryption_wrapper();

    let test_data = "context test data";
    let test_key = "context_test_key";

    // Store with one context
    wrapper
        .store_encrypted_item(test_key, &test_data, contexts::ATOM_DATA)
        .unwrap();

    // Try to retrieve with different context (should fail)
    let result: Result<Option<String>, _> =
        wrapper.get_encrypted_item(test_key, contexts::SCHEMA_DATA);
    assert!(result.is_err());

    // Retrieve with correct context (should work)
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

/// Test tree-based operations with encryption
#[test]
fn test_tree_operations() {
    let wrapper = create_test_encryption_wrapper();
    let tree = &wrapper.db_ops().metadata_tree;

    let test_data = "tree test data";
    let test_key = "tree_test_key";
    let context = contexts::METADATA;

    // Store in tree
    wrapper
        .store_encrypted_in_tree(tree, test_key, &test_data, context)
        .unwrap();

    // Retrieve from tree
    let retrieved: String = wrapper
        .get_encrypted_from_tree(tree, test_key, context)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);

    // Check existence
    assert!(wrapper.exists_in_tree(tree, test_key).unwrap());

    // Delete
    assert!(wrapper.delete_from_tree(tree, test_key).unwrap());
    assert!(!wrapper.exists_in_tree(tree, test_key).unwrap());
}

/// Test encryption statistics and monitoring
#[test]
fn test_encryption_stats() {
    let wrapper = create_test_encryption_wrapper();

    // Store some encrypted data
    wrapper
        .store_encrypted_item("key1", &"data1", contexts::ATOM_DATA)
        .unwrap();
    wrapper
        .store_encrypted_item("key2", &"data2", contexts::SCHEMA_DATA)
        .unwrap();

    // Store some unencrypted data directly
    wrapper.db_ops().store_item("key3", &"data3").unwrap();

    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&2));
    assert_eq!(stats.get("unencrypted_items"), Some(&1));
    assert_eq!(stats.get("encryption_enabled"), Some(&1));
}

/// Test migration from unencrypted to encrypted format
#[test]
fn test_migration_to_encrypted() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store some unencrypted data
    db_ops.store_item("item1", &"unencrypted data 1").unwrap();
    db_ops.store_item("item2", &"unencrypted data 2").unwrap();

    // Create encryption wrapper
    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    // Migrate to encrypted
    let migrated_count = wrapper.migrate_to_encrypted(contexts::ATOM_DATA).unwrap();
    assert_eq!(migrated_count, 2);

    // Verify data can still be read (now encrypted)
    let retrieved1: String = wrapper
        .get_encrypted_item("item1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved1, "unencrypted data 1");

    let retrieved2: String = wrapper
        .get_encrypted_item("item2", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved2, "unencrypted data 2");

    // Verify stats show all encrypted now
    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&2));
    assert_eq!(stats.get("unencrypted_items"), Some(&0));
}

/// Test self-test functionality
#[test]
fn test_self_test() {
    let wrapper = create_test_encryption_wrapper();
    wrapper.self_test().unwrap();
}

/// Test encryption disabled mode
#[test]
fn test_encryption_disabled_mode() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let wrapper = EncryptionWrapper::without_encryption(db_ops);
    assert!(!wrapper.is_encryption_enabled());

    // Should fall back to unencrypted storage
    let test_data = "unencrypted data";
    let test_key = "test_key";
    wrapper
        .store_encrypted_item(test_key, &test_data, contexts::ATOM_DATA)
        .unwrap();

    // Should be able to read it back
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

/// Test migration status reporting
#[test]
fn test_migration_status() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store mixed data
    db_ops.store_item("unencrypted", &"data1").unwrap();
    
    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();
    wrapper.store_encrypted_item("encrypted", &"data2", contexts::ATOM_DATA).unwrap();

    let status = wrapper.get_migration_status().unwrap();
    assert_eq!(status.total_items, 2);
    assert_eq!(status.encrypted_items, 1);
    assert_eq!(status.unencrypted_items, 1);
    assert!(status.is_mixed_environment());
    assert!(!status.is_fully_encrypted());
    assert_eq!(status.encryption_percentage(), 50.0);
}

/// Test migration configuration and batch processing
#[test]
fn test_batch_migration() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store multiple unencrypted items
    for i in 0..5 {
        db_ops.store_item(&format!("item{}", i), &format!("data{}", i)).unwrap();
    }

    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    let config = MigrationConfig {
        mode: MigrationMode::Full,
        batch_size: 2,
        verify_integrity: true,
        backup_before_migration: false,
        target_context: contexts::ATOM_DATA.to_string(),
    };

    let migrated_count = wrapper.perform_batch_migration(&config).unwrap();
    assert_eq!(migrated_count, 5);

    // Verify all data is now encrypted
    let status = wrapper.get_migration_status().unwrap();
    assert!(status.is_fully_encrypted());
}

/// Test data format consistency validation
#[test]
fn test_data_format_consistency() {
    let wrapper = create_test_encryption_wrapper();

    // Store mixed encrypted data with different contexts
    wrapper.store_encrypted_item("atom1", &"data1", contexts::ATOM_DATA).unwrap();
    wrapper.store_encrypted_item("schema1", &"data2", contexts::SCHEMA_DATA).unwrap();
    wrapper.store_encrypted_item("meta1", &"data3", contexts::METADATA).unwrap();

    let validation_stats = wrapper.validate_data_format_consistency().unwrap();
    assert_eq!(validation_stats.get("encrypted_valid"), Some(&3));
    assert_eq!(validation_stats.get("unencrypted_valid"), Some(&0));
    assert_eq!(validation_stats.get("invalid_format"), Some(&0));
    assert_eq!(validation_stats.get("total_contexts"), Some(&3));
}

/// Test unencrypted data detection
#[test]
fn test_detect_unencrypted_data() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store some unencrypted data
    db_ops.store_item("unencrypted1", &"data1").unwrap();
    db_ops.store_item("unencrypted2", &"data2").unwrap();

    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    // Store some encrypted data
    wrapper.store_encrypted_item("encrypted1", &"data3", contexts::ATOM_DATA).unwrap();

    let unencrypted_keys = wrapper.detect_unencrypted_data().unwrap();
    assert_eq!(unencrypted_keys.len(), 2);
    assert!(unencrypted_keys.contains(&"unencrypted1".to_string()));
    assert!(unencrypted_keys.contains(&"unencrypted2".to_string()));
}

/// Test migration mode behavior
#[test]
fn test_migration_modes() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let master_keypair = generate_master_keypair().unwrap();

    // Test ReadOnlyCompatibility mode
    let wrapper = EncryptionWrapper::with_migration_mode(
        db_ops,
        &master_keypair,
        MigrationMode::ReadOnlyCompatibility,
    ).unwrap();
    
    assert!(!wrapper.is_encryption_enabled());
    assert_eq!(wrapper.migration_mode(), MigrationMode::ReadOnlyCompatibility);

    // Test Gradual mode
    let temp_dir2 = tempdir().unwrap();
    let db2 = sled::open(temp_dir2.path()).unwrap();
    let db_ops2 = DbOperations::new(db2).unwrap();
    
    let wrapper2 = EncryptionWrapper::with_migration_mode(
        db_ops2,
        &master_keypair,
        MigrationMode::Gradual,
    ).unwrap();
    
    assert!(wrapper2.is_encryption_enabled());
    assert_eq!(wrapper2.migration_mode(), MigrationMode::Gradual);
}

/// Test error handling in various scenarios
#[test]
fn test_error_handling() {
    let wrapper = create_test_encryption_wrapper();

    // Test invalid context
    let result = wrapper.store_encrypted_item("key", &"data", "invalid_context");
    assert!(result.is_err());

    // Test context mismatch on retrieval
    wrapper.store_encrypted_item("key", &"data", contexts::ATOM_DATA).unwrap();
    let result: Result<Option<String>, _> = wrapper.get_encrypted_item("key", contexts::SCHEMA_DATA);
    assert!(result.is_err());
}

/// Test encryption with custom configuration
#[test]
fn test_migration_config_validation() {
    use crate::db_operations::migration::MigrationUtils;

    let mut config = MigrationConfig::default();
    assert!(MigrationUtils::validate_config(&config).is_ok());

    // Test invalid batch size
    config.batch_size = 0;
    assert!(MigrationUtils::validate_config(&config).is_err());

    // Test empty target context
    config.batch_size = 100;
    config.target_context = String::new();
    assert!(MigrationUtils::validate_config(&config).is_err());
}

/// Test performance with larger datasets
#[test]
fn test_performance_with_large_dataset() {
    let wrapper = create_test_encryption_wrapper();
    
    // Store many items
    let item_count = 100;
    for i in 0..item_count {
        let key = format!("perf_test_key_{}", i);
        let data = format!("performance test data item {}", i);
        wrapper.store_encrypted_item(&key, &data, contexts::ATOM_DATA).unwrap();
    }

    // Verify all items can be retrieved
    for i in 0..item_count {
        let key = format!("perf_test_key_{}", i);
        let expected_data = format!("performance test data item {}", i);
        let retrieved: String = wrapper
            .get_encrypted_item(&key, contexts::ATOM_DATA)
            .unwrap()
            .unwrap();
        assert_eq!(retrieved, expected_data);
    }

    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&(item_count as u64)));
    assert_eq!(stats.get("unencrypted_items"), Some(&0));
}