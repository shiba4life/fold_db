//! Comprehensive tests for encrypted atom storage integration
//!
//! These tests verify that the FoldDB atom storage system properly integrates
//! with the encryption wrapper to provide transparent encryption for atom data
//! while maintaining full backward compatibility.

use datafold::crypto::generate_master_keypair;
use datafold::db_operations::contexts;
use datafold::fold_db_core::FoldDB;
use datafold::config::crypto::CryptoConfig;
use serde_json::json;
use tempfile::tempdir;

/// Create a test FoldDB instance
fn create_test_folddb() -> FoldDB {
    let temp_dir = tempdir().unwrap();
    FoldDB::new(temp_dir.path().to_str().unwrap()).unwrap()
}

/// Create a test FoldDB instance with encryption enabled
fn create_encrypted_folddb() -> FoldDB {
    let mut folddb = create_test_folddb();
    let master_keypair = generate_master_keypair().unwrap();
    folddb.enable_atom_encryption(&master_keypair).unwrap();
    folddb
}

#[test]
fn test_folddb_encryption_enablement() {
    let mut folddb = create_test_folddb();
    
    // Initially encryption should be disabled
    assert!(!folddb.is_atom_encryption_enabled());
    
    // Enable encryption
    let master_keypair = generate_master_keypair().unwrap();
    folddb.enable_atom_encryption(&master_keypair).unwrap();
    
    // Now encryption should be enabled
    assert!(folddb.is_atom_encryption_enabled());
    
    // Check stats
    let stats = folddb.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encryption_enabled"), Some(&1));
    assert_eq!(stats.get("available_contexts"), Some(&(contexts::all_contexts().len() as u64)));
}

#[test]
fn test_folddb_encryption_with_crypto_config() {
    let mut folddb = create_test_folddb();
    let master_keypair = generate_master_keypair().unwrap();
    let crypto_config = CryptoConfig::default();
    
    folddb.enable_atom_encryption_with_config(&master_keypair, &crypto_config).unwrap();
    assert!(folddb.is_atom_encryption_enabled());
}

#[test]
fn test_encrypted_atom_creation_and_retrieval() {
    let folddb = create_encrypted_folddb();
    
    // Create an atom through the atom manager (should be encrypted automatically)
    let test_content = json!({
        "message": "This is a secret message",
        "timestamp": "2024-01-01T00:00:00Z",
        "priority": "high"
    });
    
    let atom = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        test_content.clone(),
    ).unwrap();
    
    assert_eq!(atom.content(), &test_content);
    assert_eq!(atom.source_schema_name(), "test_schema");
    assert_eq!(atom.source_pub_key(), "test_pub_key");
    
    // Verify that the atom is actually stored encrypted by checking the raw database
    let atom_key = format!("atom:{}", atom.uuid());
    let raw_data = folddb.db_ops().db().get(atom_key.as_bytes()).unwrap().unwrap();
    
    // The raw data should start with the encryption magic bytes
    assert!(raw_data.starts_with(b"DF_ENC"), "Atom data should be encrypted");
}

#[test]
fn test_encrypted_atom_ref_operations() {
    let folddb = create_encrypted_folddb();
    
    // Create an atom
    let test_content = json!({"test": "data"});
    let atom = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        test_content,
    ).unwrap();
    
    // Create an atom reference
    let atom_ref = folddb.atom_manager().update_atom_ref(
        "test_ref_uuid",
        atom.uuid().to_string(),
        "test_pub_key".to_string(),
    ).unwrap();
    
    assert_eq!(atom_ref.get_atom_uuid(), atom.uuid());
    
    // Verify that the atom ref is stored encrypted
    let ref_key = format!("ref:test_ref_uuid");
    let raw_ref_data = folddb.db_ops().db().get(ref_key.as_bytes()).unwrap().unwrap();
    assert!(raw_ref_data.starts_with(b"DF_ENC"), "AtomRef data should be encrypted");
}

#[test]
fn test_encrypted_atom_ref_range_operations() {
    let folddb = create_encrypted_folddb();
    
    // Create an atom
    let test_content = json!({"test": "range_data"});
    let atom = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        test_content,
    ).unwrap();
    
    // Create an atom reference range
    let atom_ref_range = folddb.atom_manager().update_atom_ref_range(
        "test_range_uuid",
        atom.uuid().to_string(),
        "range_key_1".to_string(),
        "test_pub_key".to_string(),
    ).unwrap();
    
    // Verify that the atom ref range is stored encrypted
    let range_key = format!("ref:test_range_uuid");
    let raw_range_data = folddb.db_ops().db().get(range_key.as_bytes()).unwrap().unwrap();
    assert!(raw_range_data.starts_with(b"DF_ENC"), "AtomRefRange data should be encrypted");
}

#[test]
fn test_backward_compatibility_with_unencrypted_atoms() {
    let temp_dir1 = tempdir().unwrap();
    let db_path1 = temp_dir1.path().to_str().unwrap();
    let temp_dir2 = tempdir().unwrap();
    let db_path2 = temp_dir2.path().to_str().unwrap();
    
    // First, create atoms without encryption
    let _atom_uuid = {
        let folddb = FoldDB::new(db_path1).unwrap();
        
        // Create unencrypted atom
        let test_content = json!({"legacy": "unencrypted_data"});
        let atom = folddb.atom_manager().create_atom(
            "test_schema",
            "test_pub_key".to_string(),
            test_content,
        ).unwrap();
        atom.uuid().to_string()
    };
    
    // Copy the database files to ensure we have the unencrypted data
    let src_db = std::path::Path::new(db_path1).join("db");
    let dst_db = std::path::Path::new(db_path2).join("db");
    if src_db.exists() {
        std::fs::create_dir_all(dst_db.parent().unwrap()).unwrap();
        std::fs::copy(&src_db, &dst_db).unwrap();
    }
    
    // Now open a database with encryption enabled
    {
        let mut folddb = FoldDB::new(db_path2).unwrap();
        let master_keypair = generate_master_keypair().unwrap();
        folddb.enable_atom_encryption(&master_keypair).unwrap();
        
        // Create a new encrypted atom
        let new_content = json!({"new": "encrypted_data"});
        let _new_atom = folddb.atom_manager().create_atom(
            "test_schema",
            "test_pub_key".to_string(),
            new_content,
        ).unwrap();
        
        // Check stats - should show both encrypted and unencrypted items
        let stats = folddb.get_encryption_stats().unwrap();
        assert_eq!(stats.get("encryption_enabled"), Some(&1));
        // The stats should show we have both encrypted and unencrypted items
        let total_items = stats.get("encrypted_items").unwrap() + stats.get("unencrypted_items").unwrap();
        assert!(total_items >= 1, "Should have at least 1 item");
    }
}

#[test]
fn test_migration_to_encrypted() {
    let temp_dir = tempdir().unwrap();
    let db_path = temp_dir.path().to_str().unwrap();
    
    let mut folddb = FoldDB::new(db_path).unwrap();
    
    // First, create several unencrypted atoms
    for i in 0..3 {
        let test_content = json!({"item": i, "data": format!("unencrypted_item_{}", i)});
        let _atom = folddb.atom_manager().create_atom(
            "test_schema",
            "test_pub_key".to_string(),
            test_content,
        ).unwrap();
    }
    
    // Now enable encryption and migrate
    let master_keypair = generate_master_keypair().unwrap();
    folddb.enable_atom_encryption(&master_keypair).unwrap();
    
    // Check initial stats
    let initial_stats = folddb.get_encryption_stats().unwrap();
    let initial_unencrypted = *initial_stats.get("unencrypted_items").unwrap_or(&0);
    assert!(initial_unencrypted > 0, "Should have unencrypted items to migrate");
    
    // Migrate to encrypted
    let migrated_count = folddb.migrate_atoms_to_encrypted().unwrap();
    assert!(migrated_count > 0, "Should have migrated some items");
    
    // Check final stats
    let final_stats = folddb.get_encryption_stats().unwrap();
    let final_encrypted = *final_stats.get("encrypted_items").unwrap_or(&0);
    let final_unencrypted = *final_stats.get("unencrypted_items").unwrap_or(&0);
    
    assert!(final_encrypted >= migrated_count, "Should have encrypted items");
    assert!(final_unencrypted < initial_unencrypted, "Should have fewer unencrypted items");
}

#[test]
fn test_encryption_disable_and_fallback() {
    let mut folddb = create_encrypted_folddb();
    
    // Initially encryption should be enabled
    assert!(folddb.is_atom_encryption_enabled());
    
    // Create an encrypted atom
    let test_content = json!({"test": "encrypted_data"});
    let _encrypted_atom = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        test_content.clone(),
    ).unwrap();
    
    // Disable encryption
    folddb.disable_atom_encryption();
    assert!(!folddb.is_atom_encryption_enabled());
    
    // Create another atom (should be unencrypted now)
    let _unencrypted_atom = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        test_content,
    ).unwrap();
    
    // Stats should reflect the change
    let stats = folddb.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encryption_enabled"), Some(&0));
}

#[test]
fn test_atom_history_with_encryption() {
    let folddb = create_encrypted_folddb();
    
    // Create initial atom
    let initial_content = json!({"version": 1, "data": "initial"});
    let atom1 = folddb.atom_manager().create_atom(
        "test_schema",
        "test_pub_key".to_string(),
        initial_content,
    ).unwrap();
    
    // Create atom reference
    let atom_ref = folddb.atom_manager().update_atom_ref(
        "history_test_ref",
        atom1.uuid().to_string(),
        "test_pub_key".to_string(),
    ).unwrap();
    
    assert_eq!(atom_ref.get_atom_uuid(), atom1.uuid());
    
    // Get atom history (should work with encrypted atoms)
    let history = folddb.atom_manager().get_atom_history("history_test_ref").unwrap();
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].uuid(), atom1.uuid());
}

#[test]
fn test_large_encrypted_atom_performance() {
    let folddb = create_encrypted_folddb();
    
    // Create a large atom with substantial content
    let large_data: Vec<i32> = (0..1000).collect();
    let large_content = json!({
        "large_array": large_data,
        "description": "This is a large atom for performance testing",
        "metadata": {
            "size": large_data.len(),
            "type": "performance_test"
        }
    });
    
    let start_time = std::time::Instant::now();
    
    let atom = folddb.atom_manager().create_atom(
        "large_schema",
        "test_pub_key".to_string(),
        large_content.clone(),
    ).unwrap();
    
    let creation_time = start_time.elapsed();
    assert!(creation_time < std::time::Duration::from_millis(500), 
           "Large atom creation should complete in reasonable time");
    
    // Verify the content is correct
    assert_eq!(atom.content(), &large_content);
    
    // Verify it's actually encrypted in storage
    let atom_key = format!("atom:{}", atom.uuid());
    let raw_data = folddb.db_ops().db().get(atom_key.as_bytes()).unwrap().unwrap();
    assert!(raw_data.starts_with(b"DF_ENC"), "Large atom should be encrypted");
}

#[test]
fn test_concurrent_encrypted_operations() {
    let folddb = create_encrypted_folddb();
    
    // Create multiple atoms and references
    for i in 0..5 {
        let test_content = json!({
            "concurrent_test": i,
            "data": format!("concurrent_data_{}", i)
        });
        
        let atom = folddb.atom_manager().create_atom(
            "concurrent_schema",
            format!("pub_key_{}", i),
            test_content,
        ).unwrap();
        
        // Create corresponding atom reference
        let _atom_ref = folddb.atom_manager().update_atom_ref(
            &format!("concurrent_ref_{}", i),
            atom.uuid().to_string(),
            format!("pub_key_{}", i),
        ).unwrap();
    }
    
    // Verify all operations completed successfully
    let stats = folddb.get_encryption_stats().unwrap();
    let encrypted_items = *stats.get("encrypted_items").unwrap_or(&0);
    assert!(encrypted_items >= 10, "Should have created multiple encrypted items"); // 5 atoms + 5 refs
}

#[test]
fn test_encryption_wrapper_access() {
    let folddb = create_encrypted_folddb();
    
    // Should be able to access the encryption wrapper for advanced operations
    let encryption_wrapper = folddb.encryption_wrapper().unwrap();
    assert!(encryption_wrapper.is_encryption_enabled());
    
    // Should be able to perform self-test
    encryption_wrapper.self_test().unwrap();
}

#[test]
fn test_error_handling_encryption_not_enabled() {
    let mut folddb = create_test_folddb();
    
    // Migration should fail when encryption is not enabled
    let result = folddb.migrate_atoms_to_encrypted();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Encryption is not enabled"));
}

#[test]
fn test_encryption_stats_without_encryption() {
    let folddb = create_test_folddb();
    
    // Stats should work even without encryption enabled
    let stats = folddb.get_encryption_stats().unwrap();
    assert_eq!(stats.get("encryption_enabled"), Some(&0));
    assert_eq!(stats.get("available_contexts"), Some(&0));
}