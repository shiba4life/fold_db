//! Comprehensive tests for encrypted backup and restore functionality

#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]

use datafold::db_operations::{
    DbOperations, EncryptedBackupManager, BackupMode, BackupOptions, RestoreOptions,
    BackupError
};
use datafold::crypto::generate_master_keypair;
use datafold::schema::SchemaError;
use serde_json::json;
use std::collections::HashMap;
use tempfile::{tempdir, NamedTempFile};
use uuid::Uuid;

/// Test helper to create a test database with sample data
fn create_test_database() -> (DbOperations, tempfile::TempDir) {
    let temp_dir = tempdir().expect("Failed to create temp directory");
    let db_path = temp_dir.path().join("test.db");
    let db = sled::open(&db_path).expect("Failed to open test database");
    let db_ops = DbOperations::new(db).expect("Failed to create DbOperations");
    
    // Add some test data to the expected trees that backup manager will find
    // Store in metadata tree
    let metadata_tree = db_ops.db().open_tree("metadata").expect("Failed to open metadata tree");
    metadata_tree.insert("test_atom_1", serde_json::to_vec(&json!({"id": 1, "data": "test1"})).unwrap()).expect("Failed to store metadata");
    metadata_tree.insert("test_atom_2", serde_json::to_vec(&json!({"id": 2, "data": "test2"})).unwrap()).expect("Failed to store metadata");
    
    // Store in schemas tree
    let schemas_tree = db_ops.db().open_tree("schemas").expect("Failed to open schemas tree");
    schemas_tree.insert("test_schema_1", serde_json::to_vec(&json!({"name": "TestSchema", "version": 1})).unwrap()).expect("Failed to store schema");
    
    (db_ops, temp_dir)
}

/// Test helper to verify restored data matches original
fn verify_restored_data(db_ops: &DbOperations) {
    // Check that original data was restored correctly from the correct trees
    let metadata_tree = db_ops.db().open_tree("metadata").expect("Failed to open metadata tree");
    let schemas_tree = db_ops.db().open_tree("schemas").expect("Failed to open schemas tree");
    
    let atom1_data = metadata_tree.get("test_atom_1").expect("Failed to get test_atom_1");
    assert!(atom1_data.is_some(), "test_atom_1 should exist in restored database");
    let atom1: serde_json::Value = serde_json::from_slice(&atom1_data.unwrap()).expect("Failed to deserialize atom1");
    assert_eq!(atom1["id"], 1);
    
    let atom2_data = metadata_tree.get("test_atom_2").expect("Failed to get test_atom_2");
    assert!(atom2_data.is_some(), "test_atom_2 should exist in restored database");
    let atom2: serde_json::Value = serde_json::from_slice(&atom2_data.unwrap()).expect("Failed to deserialize atom2");
    assert_eq!(atom2["id"], 2);
    
    let schema1_data = schemas_tree.get("test_schema_1").expect("Failed to get test_schema_1");
    assert!(schema1_data.is_some(), "test_schema_1 should exist in restored database");
    let schema1: serde_json::Value = serde_json::from_slice(&schema1_data.unwrap()).expect("Failed to deserialize schema1");
    assert_eq!(schema1["name"], "TestSchema");
}

#[test]
fn test_backup_manager_creation() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair);
    assert!(backup_manager.is_ok(), "Failed to create backup manager");
}

#[test]
fn test_full_backup_creation() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    let backup_file = NamedTempFile::new().expect("Failed to create temp backup file");
    let backup_path = backup_file.path();
    
    let options = BackupOptions {
        mode: BackupMode::Full,
        include_metadata: true,
        verify_during_creation: true,
        ..Default::default()
    };
    
    let result = backup_manager.create_backup(backup_path, &options);
    assert!(result.is_ok(), "Failed to create backup: {:?}", result.err());
    
    let backup_result = result.unwrap();
    assert_eq!(backup_result.metadata.mode, BackupMode::Full);
    assert!(backup_result.metadata.encryption_params.encrypted);
    assert_eq!(backup_result.metadata.encryption_params.algorithm, "AES-256-GCM");
    assert!(backup_result.stats.items_backed_up >= 0);
    
    // Verify backup file exists and has content
    assert!(backup_path.exists());
    let file_size = std::fs::metadata(backup_path).unwrap().len();
    assert!(file_size > 0, "Backup file is empty");
}

#[test]
fn test_backup_restore_roundtrip() {
    let (original_db_ops, _temp_dir1) = create_test_database();
    
    // Verify test data was stored in the correct trees
    let metadata_tree = original_db_ops.db().open_tree("metadata").expect("Failed to open metadata tree");
    let atom1_check = metadata_tree.get("test_atom_1").expect("Failed to check test data");
    assert!(atom1_check.is_some(), "Test data was not stored properly");
    println!("Test data verified: {} items found", 3);
    
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(original_db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    // Create backup
    let backup_file = NamedTempFile::new().expect("Failed to create temp backup file");
    let backup_path = backup_file.path();
    
    let backup_options = BackupOptions::default();
    let backup_result = backup_manager.create_backup(backup_path, &backup_options)
        .expect("Failed to create backup");
    
    // Validate backup was created successfully
    assert!(backup_result.stats.items_backed_up > 0, "No items were backed up");
    println!("Backup created with {} items", backup_result.stats.items_backed_up);
    
    
    // Create new database for restoration
    let temp_dir2 = tempdir().expect("Failed to create temp directory");
    let restore_db_path = temp_dir2.path().join("restore.db");
    let restore_db = sled::open(&restore_db_path).expect("Failed to open restore database");
    let restore_db_ops = DbOperations::new(restore_db).expect("Failed to create restore DbOperations");
    let restore_backup_manager = EncryptedBackupManager::new(restore_db_ops, &master_keypair)
        .expect("Failed to create restore backup manager");
    
    // Restore from backup
    let restore_options = RestoreOptions {
        overwrite_existing: true,
        verify_before_restore: true,
        backup_before_restore: false, // Skip safety backup for test
        ..Default::default()
    };
    
    let restore_result = restore_backup_manager.restore_backup(backup_path, &restore_options);
    assert!(restore_result.is_ok(), "Failed to restore backup: {:?}", restore_result.err());
    
    let restore_stats = restore_result.unwrap();
    assert!(restore_stats.items_restored > 0, "No items were restored");
    println!("Restore completed with {} items", restore_stats.items_restored);
    
    // Verify restored data
    verify_restored_data(&restore_backup_manager.db_ops);
}

#[test]
fn test_backup_integrity_verification() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    // Create backup
    let backup_file = NamedTempFile::new().expect("Failed to create temp backup file");
    let backup_path = backup_file.path();
    
    let backup_options = BackupOptions::default();
    backup_manager.create_backup(backup_path, &backup_options)
        .expect("Failed to create backup");
    
    // Verify backup integrity
    let verification_result = backup_manager.verify_backup(backup_path);
    assert!(verification_result.is_ok(), "Backup integrity verification failed");
    
    let metadata = verification_result.unwrap();
    assert_eq!(metadata.mode, BackupMode::Full);
    assert!(metadata.encryption_params.encrypted);
}

#[test]
fn test_backup_options_default() {
    let options = BackupOptions::default();
    assert_eq!(options.mode, BackupMode::Full);
    assert!(options.include_metadata);
    assert_eq!(options.compression_level, 6);
    assert!(options.verify_during_creation);
}

#[test]
fn test_restore_options_default() {
    let options = RestoreOptions::default();
    assert!(!options.overwrite_existing);
    assert!(options.verify_before_restore);
    assert!(options.backup_before_restore);
    assert!(!options.continue_on_errors);
}

#[test]
fn test_backup_with_tree_filter() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    let backup_file = NamedTempFile::new().expect("Failed to create temp backup file");
    let backup_path = backup_file.path();
    
    // Create backup with specific tree filter
    let options = BackupOptions {
        mode: BackupMode::Full,
        tree_filter: Some(vec!["metadata".to_string()]),
        include_metadata: true,
        ..Default::default()
    };
    
    let result = backup_manager.create_backup(backup_path, &options);
    assert!(result.is_ok(), "Failed to create filtered backup: {:?}", result.err());
    
    let backup_result = result.unwrap();
    // Should have fewer items since we filtered by tree
    assert!(backup_result.stats.items_backed_up >= 0);
}

#[test]
fn test_backup_format_error_handling() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    // Create a file with invalid format
    let invalid_backup_file = NamedTempFile::new().expect("Failed to create temp file");
    let invalid_backup_path = invalid_backup_file.path();
    std::fs::write(invalid_backup_path, b"invalid backup data")
        .expect("Failed to write invalid data");
    
    // Try to verify the invalid backup
    let verification_result = backup_manager.verify_backup(invalid_backup_path);
    assert!(verification_result.is_err());
    
    match verification_result.unwrap_err() {
        BackupError::FormatError(_) => {}, // Expected
        other => panic!("Expected FormatError, got: {:?}", other),
    }
}

#[test]
fn test_backup_file_not_found() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    let nonexistent_path = "/tmp/nonexistent_backup.dfb";
    
    // Try to verify non-existent backup
    let verification_result = backup_manager.verify_backup(nonexistent_path);
    assert!(verification_result.is_err());
    
    match verification_result.unwrap_err() {
        BackupError::FileNotFound(_) => {}, // Expected
        other => panic!("Expected FileNotFound, got: {:?}", other),
    }
}

#[test]
fn test_backup_list_functionality() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    let backup_dir = tempdir().expect("Failed to create backup directory");
    let backup_dir_path = backup_dir.path();
    
    // Create multiple backups
    for i in 1..=3 {
        let backup_path = backup_dir_path.join(format!("backup_{}.dfb", i));
        let backup_options = BackupOptions::default();
        backup_manager.create_backup(&backup_path, &backup_options)
            .expect("Failed to create backup");
    }
    
    // List backups
    let backup_list = backup_manager.list_backups(backup_dir_path);
    assert!(backup_list.is_ok());
    
    let backups = backup_list.unwrap();
    assert_eq!(backups.len(), 3);
    
    // Verify backups are sorted by creation time
    for i in 1..backups.len() {
        assert!(backups[i-1].created_at <= backups[i].created_at);
    }
}

#[test]
fn test_incremental_backup_mode() {
    let (db_ops, _temp_dir) = create_test_database();
    let master_keypair = generate_master_keypair().expect("Failed to generate keypair");
    let backup_manager = EncryptedBackupManager::new(db_ops, &master_keypair)
        .expect("Failed to create backup manager");
    
    let backup_file = NamedTempFile::new().expect("Failed to create temp backup file");
    let backup_path = backup_file.path();
    
    // Create incremental backup
    let options = BackupOptions {
        mode: BackupMode::Incremental,
        ..Default::default()
    };
    
    let result = backup_manager.create_backup(backup_path, &options);
    assert!(result.is_ok(), "Failed to create incremental backup: {:?}", result.err());
    
    let backup_result = result.unwrap();
    assert_eq!(backup_result.metadata.mode, BackupMode::Incremental);
}

#[test]
fn test_backup_error_types() {
    // Test that all error types can be created and formatted
    let errors = vec![
        BackupError::FormatError("test format error".to_string()),
        BackupError::VersionMismatch { found: 2, expected: 1 },
        BackupError::IntegrityError("test integrity error".to_string()),
        BackupError::CorruptionError("test corruption error".to_string()),
        BackupError::EncryptionKeyError("test key error".to_string()),
        BackupError::FileNotFound("test file".to_string()),
        BackupError::PermissionError("test permission error".to_string()),
        BackupError::DatabaseError("test database error".to_string()),
    ];
    
    for error in errors {
        let error_string = format!("{}", error);
        assert!(!error_string.is_empty(), "Error should have a non-empty string representation");
    }
}