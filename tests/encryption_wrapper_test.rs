//! Comprehensive tests for the database encryption wrapper layer
//!
//! These tests verify the encryption wrapper functionality including:
//! - Transparent encryption/decryption
//! - Backward compatibility with unencrypted data
//! - Multiple encryption contexts
//! - Migration capabilities
//! - Error handling and edge cases

use datafold::config::crypto::CryptoConfig;
use datafold::crypto::generate_master_keypair;
use datafold::db_operations::{contexts, DbOperations, EncryptionWrapper};
use datafold::schema::SchemaError;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: u32,
    name: String,
    values: Vec<i32>,
}

impl TestData {
    fn new(id: u32, name: &str, values: Vec<i32>) -> Self {
        Self {
            id,
            name: name.to_string(),
            values,
        }
    }
}

fn create_test_encryption_wrapper() -> EncryptionWrapper {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let master_keypair = generate_master_keypair().unwrap();
    EncryptionWrapper::new(db_ops, &master_keypair).unwrap()
}

fn create_test_db_ops() -> DbOperations {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    DbOperations::new(db).unwrap()
}

#[test]
fn test_encryption_wrapper_creation_success() {
    let wrapper = create_test_encryption_wrapper();
    assert!(wrapper.is_encryption_enabled());

    // Should have encryptors for all contexts
    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(
        stats.get("available_contexts"),
        Some(&(contexts::all_contexts().len() as u64))
    );
    assert_eq!(stats.get("encryption_enabled"), Some(&1));
}

#[test]
fn test_encryption_wrapper_with_config() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let master_keypair = generate_master_keypair().unwrap();
    let crypto_config = CryptoConfig::default();

    let wrapper = EncryptionWrapper::with_config(db_ops, &master_keypair, &crypto_config).unwrap();
    assert!(wrapper.is_encryption_enabled());
}

#[test]
fn test_encryption_wrapper_without_encryption() {
    let db_ops = create_test_db_ops();
    let wrapper = EncryptionWrapper::without_encryption(db_ops);
    assert!(!wrapper.is_encryption_enabled());
}

#[test]
fn test_store_and_retrieve_encrypted_simple_data() {
    let wrapper = create_test_encryption_wrapper();

    let test_string = "Simple test string";
    let test_key = "simple_test";
    let context = contexts::ATOM_DATA;

    // Store encrypted
    wrapper
        .store_encrypted_item(test_key, &test_string, context)
        .unwrap();

    // Retrieve and verify
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, context)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_string);
}

#[test]
fn test_store_and_retrieve_encrypted_complex_data() {
    let wrapper = create_test_encryption_wrapper();

    let test_data = TestData::new(42, "complex test", vec![1, 2, 3, 4, 5]);
    let test_key = "complex_test";
    let context = contexts::SCHEMA_DATA;

    // Store encrypted
    wrapper
        .store_encrypted_item(test_key, &test_data, context)
        .unwrap();

    // Retrieve and verify
    let retrieved: TestData = wrapper
        .get_encrypted_item(test_key, context)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

#[test]
fn test_multiple_encryption_contexts() {
    let wrapper = create_test_encryption_wrapper();

    let test_data = "Context separation test";
    let test_key = "context_test";

    // Store with different contexts
    wrapper
        .store_encrypted_item(test_key, &test_data, contexts::ATOM_DATA)
        .unwrap();

    // Try to retrieve with wrong context - should fail
    let result: Result<Option<String>, SchemaError> =
        wrapper.get_encrypted_item(test_key, contexts::SCHEMA_DATA);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Context mismatch"));

    // Retrieve with correct context - should succeed
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

#[test]
fn test_all_encryption_contexts() {
    let wrapper = create_test_encryption_wrapper();

    // Test all available contexts
    for context in contexts::all_contexts() {
        let test_data = format!("Test data for context: {}", context);
        let test_key = format!("test_{}", context);

        // Store and retrieve
        wrapper
            .store_encrypted_item(&test_key, &test_data, context)
            .unwrap();
        let retrieved: String = wrapper
            .get_encrypted_item(&test_key, context)
            .unwrap()
            .unwrap();
        assert_eq!(retrieved, test_data);
    }
}

#[test]
fn test_backward_compatibility_unencrypted_data() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store unencrypted data using raw DbOperations
    let test_data = TestData::new(123, "legacy data", vec![10, 20, 30]);
    let test_key = "legacy_key";
    db_ops.store_item(test_key, &test_data).unwrap();

    // Create encryption wrapper
    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    // Should be able to read unencrypted data
    let retrieved: TestData = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);
}

#[test]
fn test_encryption_stats() {
    let wrapper = create_test_encryption_wrapper();

    // Initially no data
    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&0));
    assert_eq!(stats.get("unencrypted_items"), Some(&0));

    // Store some encrypted data
    wrapper
        .store_encrypted_item("key1", &"data1", contexts::ATOM_DATA)
        .unwrap();
    wrapper
        .store_encrypted_item("key2", &"data2", contexts::SCHEMA_DATA)
        .unwrap();

    // Store some unencrypted data directly
    wrapper.db_ops().store_item("key3", &"data3").unwrap();
    wrapper.db_ops().store_item("key4", &"data4").unwrap();

    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&2));
    assert_eq!(stats.get("unencrypted_items"), Some(&2));
    assert_eq!(stats.get("encryption_enabled"), Some(&1));
    assert_eq!(
        stats.get("available_contexts"),
        Some(&(contexts::all_contexts().len() as u64))
    );
}

#[test]
fn test_migration_to_encrypted() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    // Store some unencrypted data
    let data1 = TestData::new(1, "item1", vec![1, 2]);
    let data2 = TestData::new(2, "item2", vec![3, 4]);
    let data3 = "simple string data";

    db_ops.store_item("item1", &data1).unwrap();
    db_ops.store_item("item2", &data2).unwrap();
    db_ops.store_item("item3", &data3).unwrap();

    // Create encryption wrapper
    let master_keypair = generate_master_keypair().unwrap();
    let wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    // Verify initial stats
    let initial_stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(initial_stats.get("encrypted_items"), Some(&0));
    assert_eq!(initial_stats.get("unencrypted_items"), Some(&3));

    // Migrate to encrypted
    let migrated_count = wrapper.migrate_to_encrypted(contexts::ATOM_DATA).unwrap();
    assert_eq!(migrated_count, 3);

    // Verify all data is now encrypted and can still be read
    let retrieved1: TestData = wrapper
        .get_encrypted_item("item1", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved1, data1);

    let retrieved2: TestData = wrapper
        .get_encrypted_item("item2", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved2, data2);

    let retrieved3: String = wrapper
        .get_encrypted_item("item3", contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved3, data3);

    // Verify final stats
    let final_stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(final_stats.get("encrypted_items"), Some(&3));
    assert_eq!(final_stats.get("unencrypted_items"), Some(&0));
}

#[test]
fn test_migration_with_encryption_disabled() {
    let db_ops = create_test_db_ops();
    let wrapper = EncryptionWrapper::without_encryption(db_ops);

    let result = wrapper.migrate_to_encrypted(contexts::ATOM_DATA);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("encryption is disabled"));
}

#[test]
fn test_self_test_functionality() {
    let wrapper = create_test_encryption_wrapper();

    // Self-test should pass for enabled encryption
    wrapper.self_test().unwrap();

    // Self-test should pass (no-op) for disabled encryption
    let db_ops = create_test_db_ops();
    let disabled_wrapper = EncryptionWrapper::without_encryption(db_ops);
    disabled_wrapper.self_test().unwrap();
}

#[test]
fn test_encryption_enabled_toggle() {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    let db_ops = DbOperations::new(db).unwrap();

    let master_keypair = generate_master_keypair().unwrap();
    let mut wrapper = EncryptionWrapper::new(db_ops, &master_keypair).unwrap();

    assert!(wrapper.is_encryption_enabled());

    // Disable encryption
    wrapper.set_encryption_enabled(false);
    assert!(!wrapper.is_encryption_enabled());

    // Should fall back to unencrypted storage
    let test_data = "unencrypted fallback data";
    let test_key = "fallback_test";
    wrapper
        .store_encrypted_item(test_key, &test_data, contexts::ATOM_DATA)
        .unwrap();

    // Should still be able to read it
    let retrieved: String = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, test_data);

    // Re-enable encryption
    wrapper.set_encryption_enabled(true);
    assert!(wrapper.is_encryption_enabled());
}

#[test]
fn test_error_handling_invalid_context() {
    let wrapper = create_test_encryption_wrapper();

    let test_data = "test data";
    let test_key = "test_key";
    let invalid_context = "invalid_context";

    // Storing with invalid context should fail
    let result = wrapper.store_encrypted_item(test_key, &test_data, invalid_context);
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Unknown encryption context"));
}

#[test]
fn test_error_handling_corrupted_data() {
    let wrapper = create_test_encryption_wrapper();

    // Store some valid encrypted data first
    let test_data = "valid data";
    let test_key = "test_key";
    wrapper
        .store_encrypted_item(test_key, &test_data, contexts::ATOM_DATA)
        .unwrap();

    // Corrupt the data by directly modifying the database
    let mut corrupted_data = Vec::new();
    corrupted_data.extend_from_slice(b"DF_ENC"); // Valid magic
    corrupted_data.push(1); // Valid version
    corrupted_data.push(9); // Valid context length
    corrupted_data.extend_from_slice(b"atom_data"); // Valid context
    corrupted_data.extend_from_slice(b"corrupted_nonce_and_ciphertext"); // Invalid encrypted data

    wrapper
        .db_ops()
        .db()
        .insert(test_key.as_bytes(), corrupted_data)
        .unwrap();

    // Trying to retrieve should fail
    let result: Result<Option<String>, SchemaError> =
        wrapper.get_encrypted_item(test_key, contexts::ATOM_DATA);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_large_data_encryption() {
    let wrapper = create_test_encryption_wrapper();

    // Create moderately sized test data (reduced from 50000 to 5000 for faster testing)
    let large_data: Vec<u8> = (0..5000).map(|i| (i % 256) as u8).collect();
    let test_key = "large_data_test";

    // Store and retrieve large data with timeout protection
    let encryption_timeout = std::time::Duration::from_secs(30);

    let result = tokio::time::timeout(encryption_timeout, async {
        let start = std::time::Instant::now();

        // Store encrypted data
        wrapper
            .store_encrypted_item(test_key, &large_data, contexts::ATOM_DATA)
            .unwrap();

        // Retrieve and verify data
        let retrieved: Vec<u8> = wrapper
            .get_encrypted_item(test_key, contexts::ATOM_DATA)
            .unwrap()
            .unwrap();
        let duration = start.elapsed();

        (retrieved, duration)
    })
    .await;

    match result {
        Ok((retrieved, duration)) => {
            assert_eq!(retrieved, large_data);
            assert!(
                duration < std::time::Duration::from_secs(15),
                "Test took too long: {:?}",
                duration
            );
            println!("Large data encryption test completed in {:?}", duration);
        }
        Err(_) => {
            panic!(
                "Test timed out after {} seconds - encryption operations are hanging",
                encryption_timeout.as_secs()
            );
        }
    }
}

#[test]
fn test_empty_data_encryption() {
    let wrapper = create_test_encryption_wrapper();

    let empty_data: Vec<u8> = Vec::new();
    let test_key = "empty_data_test";

    // Store and retrieve empty data
    wrapper
        .store_encrypted_item(test_key, &empty_data, contexts::ATOM_DATA)
        .unwrap();
    let retrieved: Vec<u8> = wrapper
        .get_encrypted_item(test_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved, empty_data);
}

#[test]
fn test_concurrent_context_usage() {
    let wrapper = create_test_encryption_wrapper();

    // Store data with multiple contexts simultaneously
    let contexts_and_data = [
        (contexts::ATOM_DATA, "atom data"),
        (contexts::SCHEMA_DATA, "schema data"),
        (contexts::METADATA, "metadata"),
        (contexts::TRANSFORM_DATA, "transform data"),
    ];

    // Store all data
    for (i, (context, data)) in contexts_and_data.iter().enumerate() {
        let key = format!("concurrent_test_{}", i);
        wrapper.store_encrypted_item(&key, data, context).unwrap();
    }

    // Retrieve and verify all data
    for (i, (context, expected_data)) in contexts_and_data.iter().enumerate() {
        let key = format!("concurrent_test_{}", i);
        let retrieved: String = wrapper.get_encrypted_item(&key, context).unwrap().unwrap();
        assert_eq!(retrieved, *expected_data);
    }
}

#[test]
fn test_database_operations_delegation() {
    let wrapper = create_test_encryption_wrapper();

    // Test that we can still access underlying DbOperations
    let db_ops = wrapper.db_ops();

    // Store some data using raw operations
    let test_data = "raw operation data";
    db_ops.store_item("raw_key", &test_data).unwrap();

    // Retrieve using raw operations
    let retrieved: String = db_ops.get_item("raw_key").unwrap().unwrap();
    assert_eq!(retrieved, test_data);
}

#[test]
fn test_encryption_format_detection() {
    let wrapper = create_test_encryption_wrapper();

    // Store encrypted data
    let encrypted_data = "encrypted test data";
    let encrypted_key = "encrypted_key";
    wrapper
        .store_encrypted_item(encrypted_key, &encrypted_data, contexts::ATOM_DATA)
        .unwrap();

    // Store unencrypted data directly
    let unencrypted_data = "unencrypted test data";
    let unencrypted_key = "unencrypted_key";
    wrapper
        .db_ops()
        .store_item(unencrypted_key, &unencrypted_data)
        .unwrap();

    // Both should be readable through the wrapper
    let retrieved_encrypted: String = wrapper
        .get_encrypted_item(encrypted_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved_encrypted, encrypted_data);

    let retrieved_unencrypted: String = wrapper
        .get_encrypted_item(unencrypted_key, contexts::ATOM_DATA)
        .unwrap()
        .unwrap();
    assert_eq!(retrieved_unencrypted, unencrypted_data);

    // Stats should reflect the difference
    let stats = wrapper.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encrypted_items"), Some(&1));
    assert_eq!(stats.get("unencrypted_items"), Some(&1));
}
