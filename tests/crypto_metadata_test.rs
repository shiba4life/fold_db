//! Integration tests for crypto metadata storage and retrieval functionality

use datafold::{
    db_operations::{DbOperations, CryptoMetadata},
    crypto::{generate_master_keypair, derive_master_keypair, generate_salt, Argon2Params},
    MasterKeyConfig,
};
use tempfile::tempdir;

/// Helper function to create a test database
fn create_test_db() -> DbOperations {
    let temp_dir = tempdir().unwrap();
    let db = sled::open(temp_dir.path()).unwrap();
    DbOperations::new(db).unwrap()
}

#[test]
fn test_store_and_retrieve_random_key_metadata() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    
    // Create metadata for random key
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    
    // Store metadata
    assert!(db_ops.store_crypto_metadata(&metadata).is_ok());
    
    // Verify metadata exists
    assert!(db_ops.has_crypto_metadata().unwrap());
    
    // Retrieve and verify
    let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
    assert_eq!(retrieved.version, 1);
    assert_eq!(retrieved.signature_algorithm, "Ed25519");
    assert_eq!(retrieved.key_derivation_method, "Random");
    assert_eq!(retrieved.master_public_key.to_bytes(), keypair.public_key().to_bytes());
    assert!(retrieved.verify_integrity().unwrap());
}

#[test]
fn test_store_and_retrieve_passphrase_derived_metadata() {
    let db_ops = create_test_db();
    
    // Generate key from passphrase
    let passphrase = "test_passphrase_for_crypto_metadata";
    let salt = generate_salt();
    let params = Argon2Params::default();
    let keypair = derive_master_keypair(passphrase, &salt, &params).unwrap();
    
    // Create metadata for passphrase-derived key
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Argon2id".to_string(),
    ).unwrap();
    
    // Store metadata
    assert!(db_ops.store_crypto_metadata(&metadata).is_ok());
    
    // Retrieve and verify
    let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
    assert_eq!(retrieved.key_derivation_method, "Argon2id");
    assert_eq!(retrieved.master_public_key.to_bytes(), keypair.public_key().to_bytes());
}

#[test]
fn test_crypto_metadata_integrity_verification() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    
    // Verify integrity before storing
    assert!(metadata.verify_integrity().unwrap());
    
    // Store and retrieve
    db_ops.store_crypto_metadata(&metadata).unwrap();
    let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
    
    // Verify integrity after retrieval
    assert!(retrieved.verify_integrity().unwrap());
    
    // Verify checksum is not empty
    assert!(!retrieved.checksum.is_empty());
}

#[test]
fn test_master_public_key_quick_access() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    let public_key_bytes = keypair.public_key().to_bytes();
    
    // Initially no key
    assert!(db_ops.get_master_public_key().unwrap().is_none());
    
    // Store metadata
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    db_ops.store_crypto_metadata(&metadata).unwrap();
    
    // Quick access to public key
    let retrieved_key = db_ops.get_master_public_key().unwrap().unwrap();
    assert_eq!(retrieved_key.to_bytes(), public_key_bytes);
}

#[test]
fn test_crypto_metadata_additional_metadata() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    
    // Store initial metadata
    db_ops.store_crypto_metadata(&metadata).unwrap();
    
    // Update additional metadata
    db_ops.update_crypto_additional_metadata(
        "database_id".to_string(),
        "test_db_123".to_string(),
    ).unwrap();
    
    // Verify first update
    let updated_metadata = db_ops.get_crypto_metadata().unwrap().unwrap();
    assert_eq!(
        updated_metadata.additional_metadata.get("database_id"),
        Some(&"test_db_123".to_string())
    );
    assert!(updated_metadata.verify_integrity().unwrap());
    
    // Add second piece of metadata
    db_ops.update_crypto_additional_metadata(
        "encryption_level".to_string(),
        "AES-256".to_string(),
    ).unwrap();
    
    // Verify final state
    let final_metadata = db_ops.get_crypto_metadata().unwrap().unwrap();
    assert_eq!(final_metadata.additional_metadata.len(), 2);
    assert_eq!(
        final_metadata.additional_metadata.get("database_id"),
        Some(&"test_db_123".to_string())
    );
    assert_eq!(
        final_metadata.additional_metadata.get("encryption_level"),
        Some(&"AES-256".to_string())
    );
    
    assert!(final_metadata.verify_integrity().unwrap());
}

#[test]
fn test_crypto_metadata_statistics() {
    let db_ops = create_test_db();
    
    // No crypto metadata initially
    let stats = db_ops.get_crypto_metadata_stats().unwrap();
    assert_eq!(stats.get("crypto_enabled"), Some(&"false".to_string()));
    
    // Add crypto metadata
    let keypair = generate_master_keypair().unwrap();
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Argon2id-Interactive".to_string(),
    ).unwrap();
    db_ops.store_crypto_metadata(&metadata).unwrap();
    
    // Check stats with crypto metadata
    let stats = db_ops.get_crypto_metadata_stats().unwrap();
    assert_eq!(stats.get("crypto_enabled"), Some(&"true".to_string()));
    assert_eq!(stats.get("version"), Some(&"1".to_string()));
    assert_eq!(stats.get("signature_algorithm"), Some(&"Ed25519".to_string()));
    assert_eq!(stats.get("key_derivation_method"), Some(&"Argon2id-Interactive".to_string()));
    assert_eq!(stats.get("integrity_verified"), Some(&"true".to_string()));
    assert_eq!(stats.get("additional_entries"), Some(&"0".to_string()));
    
    // Add additional metadata and check stats again
    db_ops.update_crypto_additional_metadata(
        "test_key".to_string(),
        "test_value".to_string(),
    ).unwrap();
    
    let stats = db_ops.get_crypto_metadata_stats().unwrap();
    assert_eq!(stats.get("additional_entries"), Some(&"1".to_string()));
}

#[test]
fn test_crypto_version_access() {
    let db_ops = create_test_db();
    
    // No version initially
    assert!(db_ops.get_crypto_version().unwrap().is_none());
    
    // Store metadata
    let keypair = generate_master_keypair().unwrap();
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    db_ops.store_crypto_metadata(&metadata).unwrap();
    
    // Check version
    let version = db_ops.get_crypto_version().unwrap().unwrap();
    assert_eq!(version, 1);
}

#[test]
fn test_crypto_metadata_persistence_across_db_reopen() {
    let temp_dir = tempdir().unwrap();
    let keypair = generate_master_keypair().unwrap();
    let public_key_bytes = keypair.public_key().to_bytes();
    
    // Store metadata in first database instance
    {
        let db = sled::open(temp_dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        let metadata = CryptoMetadata::new(
            keypair.public_key().clone(),
            "Random".to_string(),
        ).unwrap();
        db_ops.store_crypto_metadata(&metadata).unwrap();
    }
    
    // Reopen database and verify metadata persists
    {
        let db = sled::open(temp_dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        
        assert!(db_ops.has_crypto_metadata().unwrap());
        let retrieved_key = db_ops.get_master_public_key().unwrap().unwrap();
        assert_eq!(retrieved_key.to_bytes(), public_key_bytes);
        
        let metadata = db_ops.get_crypto_metadata().unwrap().unwrap();
        assert!(metadata.verify_integrity().unwrap());
        assert_eq!(metadata.key_derivation_method, "Random");
    }
}

#[test]
fn test_crypto_metadata_serialization_compatibility() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    
    // Create and store initial metadata
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Argon2id-Enhanced".to_string(),
    ).unwrap();
    
    db_ops.store_crypto_metadata(&metadata).unwrap();
    
    // Add one piece of additional metadata through database operations
    db_ops.update_crypto_additional_metadata("created_by".to_string(), "DataFold v0.1.0".to_string()).unwrap();
    
    // Retrieve and verify
    let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
    
    // Verify additional metadata
    assert_eq!(retrieved.additional_metadata.len(), 1);
    assert_eq!(retrieved.additional_metadata.get("created_by"), Some(&"DataFold v0.1.0".to_string()));
    
    // Check integrity - this should work since update_crypto_additional_metadata recalculates checksum
    assert!(retrieved.verify_integrity().unwrap());
}

#[test]
fn test_crypto_metadata_error_handling() {
    let db_ops = create_test_db();
    
    // Try to update metadata when none exists
    let result = db_ops.update_crypto_additional_metadata(
        "key".to_string(),
        "value".to_string(),
    );
    assert!(result.is_err());
    
    // Try to get metadata when none exists
    assert!(db_ops.get_crypto_metadata().unwrap().is_none());
    assert!(db_ops.get_master_public_key().unwrap().is_none());
    assert!(!db_ops.has_crypto_metadata().unwrap());
}

#[test]
fn test_crypto_metadata_delete_functionality() {
    let db_ops = create_test_db();
    let keypair = generate_master_keypair().unwrap();
    let metadata = CryptoMetadata::new(
        keypair.public_key().clone(),
        "Random".to_string(),
    ).unwrap();
    
    // Store metadata
    db_ops.store_crypto_metadata(&metadata).unwrap();
    assert!(db_ops.has_crypto_metadata().unwrap());
    
    // Delete metadata
    assert!(db_ops.delete_crypto_metadata().unwrap());
    
    // Verify deletion
    assert!(!db_ops.has_crypto_metadata().unwrap());
    assert!(db_ops.get_crypto_metadata().unwrap().is_none());
    assert!(db_ops.get_master_public_key().unwrap().is_none());
    
    // Try to delete again (should return false)
    assert!(!db_ops.delete_crypto_metadata().unwrap());
}

#[test]
fn test_integration_with_crypto_config() {
    let db_ops = create_test_db();
    
    // Test with different crypto configurations
    let test_cases = [
        ("Random", MasterKeyConfig::Random),
        ("Passphrase-Interactive", MasterKeyConfig::Passphrase { 
            passphrase: "test_passphrase".to_string() 
        }),
    ];
    
    for (method_name, master_key_config) in test_cases {
        // Generate key based on config
        let keypair = match &master_key_config {
            MasterKeyConfig::Random => generate_master_keypair().unwrap(),
                         MasterKeyConfig::Passphrase { passphrase } => {
                 let salt = generate_salt();
                 let params = Argon2Params::interactive();
                 derive_master_keypair(passphrase, &salt, &params).unwrap()
             }
            _ => panic!("Unsupported master key config for test"),
        };
        
        // Create and store metadata
        let metadata = CryptoMetadata::new(
            keypair.public_key().clone(),
            method_name.to_string(),
        ).unwrap();
        
        // Clear any existing metadata
        let _ = db_ops.delete_crypto_metadata();
        
        // Store new metadata
        db_ops.store_crypto_metadata(&metadata).unwrap();
        
        // Verify storage and retrieval
        let retrieved = db_ops.get_crypto_metadata().unwrap().unwrap();
        assert_eq!(retrieved.key_derivation_method, method_name);
        assert_eq!(retrieved.master_public_key.to_bytes(), keypair.public_key().to_bytes());
        assert!(retrieved.verify_integrity().unwrap());
    }
}

#[test]
fn test_backward_compatibility_check() {
    let db_ops = create_test_db();
    
    // Simulate database without crypto metadata (backward compatibility)
    assert!(!db_ops.has_crypto_metadata().unwrap());
    
    // Operations should work gracefully with no crypto metadata
    let stats = db_ops.get_crypto_metadata_stats().unwrap();
    assert_eq!(stats.get("crypto_enabled"), Some(&"false".to_string()));
    
    assert!(db_ops.get_crypto_metadata().unwrap().is_none());
    assert!(db_ops.get_master_public_key().unwrap().is_none());
    assert!(db_ops.get_crypto_version().unwrap().is_none());
} 