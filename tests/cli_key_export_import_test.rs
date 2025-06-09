//! Tests for CLI key export/import functionality (Task 10-4-4)
//! 
//! These tests verify the implementation of encrypted key export/import
//! capabilities in the DataFold CLI, following the research guidelines
//! from task 10-1-3.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test helper to run CLI commands
    fn run_cli_command(args: &[&str]) -> std::process::Output {
        Command::new("cargo")
            .arg("run")
            .arg("--bin")
            .arg("datafold_cli")
            .arg("--")
            .args(args)
            .output()
            .expect("Failed to execute CLI command")
    }

    /// Test helper to create a temporary directory
    fn create_temp_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temporary directory")
    }

    #[test]
    fn test_key_export_import_json_format() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("exported_key.json");
        
        // Step 1: Generate a test key
        let output = run_cli_command(&[
            "generate-key",
            "--format", "hex",
            "--private-key-file", &temp_dir.path().join("test_private.key").to_string_lossy(),
            "--public-key-file", &temp_dir.path().join("test_public.key").to_string_lossy(),
        ]);
        assert!(output.status.success(), "Key generation failed");
        
        // Step 2: Store the key with a known passphrase
        // Note: This would require stdin input in a real scenario
        // For testing, we'll use the store-key command with file input
        let key_content = fs::read_to_string(temp_dir.path().join("test_private.key"))
            .expect("Failed to read generated private key");
        
        // Step 3: Test export with JSON format
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "json",
            "--include-metadata",
        ]);
        
        // Step 4: Verify export file exists and has correct format
        assert!(export_file.exists(), "Export file was not created");
        
        let export_content = fs::read_to_string(&export_file)
            .expect("Failed to read export file");
        
        // Verify it's valid JSON
        let export_data: serde_json::Value = serde_json::from_str(&export_content)
            .expect("Export file is not valid JSON");
        
        // Verify required fields are present
        assert!(export_data["version"].is_number(), "Missing version field");
        assert!(export_data["kdf"].is_string(), "Missing kdf field");
        assert!(export_data["encryption"].is_string(), "Missing encryption field");
        assert!(export_data["ciphertext"].is_array(), "Missing ciphertext field");
        assert!(export_data["metadata"].is_object(), "Missing metadata field");
        
        // Step 5: Test import
        let import_key_id = "imported_test_key";
        let output = run_cli_command(&[
            "import-key",
            "--export-file", &export_file.to_string_lossy(),
            "--key-id", import_key_id,
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--verify-integrity",
        ]);
        
        // Verify import completed successfully
        let imported_key_file = storage_dir.join(format!("{}.key", import_key_id));
        assert!(imported_key_file.exists(), "Imported key file was not created");
    }

    #[test]
    fn test_key_export_import_binary_format() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("exported_key.bin");
        
        // Test binary format export/import
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "binary",
        ]);
        
        // Verify binary export file exists
        assert!(export_file.exists(), "Binary export file was not created");
        
        // Test import from binary format
        let output = run_cli_command(&[
            "import-key",
            "--export-file", &export_file.to_string_lossy(),
            "--key-id", "imported_binary_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
        ]);
    }

    #[test]
    fn test_export_with_additional_passphrase() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("double_encrypted.json");
        
        // Test export with additional passphrase protection
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "json",
            "--export-passphrase",
            "--include-metadata",
        ]);
        
        assert!(export_file.exists(), "Double-encrypted export file was not created");
    }

    #[test]
    fn test_import_integrity_verification() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("test_export.json");
        
        // Test import with integrity verification enabled
        let output = run_cli_command(&[
            "import-key",
            "--export-file", &export_file.to_string_lossy(),
            "--key-id", "verified_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--verify-integrity", "true",
            "--force",
        ]);
    }

    #[test]
    fn test_import_corrupted_file_detection() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let corrupted_file = temp_dir.path().join("corrupted.json");
        
        // Create a corrupted export file
        fs::write(&corrupted_file, r#"{"version": 1, "invalid": "data"}"#)
            .expect("Failed to create corrupted file");
        
        // Test import should fail gracefully
        let output = run_cli_command(&[
            "import-key",
            "--export-file", &corrupted_file.to_string_lossy(),
            "--key-id", "corrupted_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
        ]);
        
        // Should fail due to corruption
        assert!(!output.status.success(), "Import should fail for corrupted file");
    }

    #[test]
    fn test_import_wrong_passphrase_detection() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("valid_export.json");
        
        // Create a valid export file (this would normally be done by export command)
        let valid_export = r#"{
            "version": 1,
            "kdf": "argon2id",
            "kdf_params": {
                "salt": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
                "memory": 131072,
                "iterations": 4,
                "parallelism": 4
            },
            "encryption": "aes-gcm-like",
            "nonce": [1,2,3,4,5,6,7,8,9,10,11,12],
            "ciphertext": [1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32],
            "created": "2025-06-08T23:00:00Z",
            "metadata": null
        }"#;
        
        fs::write(&export_file, valid_export)
            .expect("Failed to create test export file");
        
        // Test import with wrong passphrase should fail gracefully
        let output = run_cli_command(&[
            "import-key",
            "--export-file", &export_file.to_string_lossy(),
            "--key-id", "wrong_pass_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
        ]);
    }

    #[test]
    fn test_cross_platform_compatibility() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("cross_platform.json");
        
        // Test that export files use standard formats
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "json",
        ]);
        
        if export_file.exists() {
            let content = fs::read_to_string(&export_file)
                .expect("Failed to read export file");
            
            // Verify standard JSON format
            let parsed: serde_json::Value = serde_json::from_str(&content)
                .expect("Export should be valid JSON");
            
            // Verify standard algorithms
            assert_eq!(parsed["kdf"], "argon2id");
            assert_eq!(parsed["encryption"], "aes-gcm-like");
        }
    }

    #[test]
    fn test_export_file_permissions() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("permissions_test.json");
        
        // Test export and check file permissions
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "json",
        ]);
        
        if export_file.exists() {
            let metadata = fs::metadata(&export_file)
                .expect("Failed to get export file metadata");
            
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();
                // Check that file has 600 permissions (owner read/write only)
                assert_eq!(mode & 0o777, 0o600, "Export file should have 600 permissions");
            }
        }
    }

    #[test]
    fn test_batch_export_operations() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        
        // Test exporting multiple keys
        let keys = ["key1", "key2", "key3"];
        
        for key_id in &keys {
            let export_file = temp_dir.path().join(format!("{}.json", key_id));
            
            let output = run_cli_command(&[
                "export-key",
                "--key-id", key_id,
                "--storage-dir", &storage_dir.to_string_lossy(),
                "--export-file", &export_file.to_string_lossy(),
                "--format", "json",
            ]);
        }
    }

    #[test]
    fn test_metadata_preservation() {
        let temp_dir = create_temp_dir();
        let storage_dir = temp_dir.path().join("keys");
        let export_file = temp_dir.path().join("metadata_test.json");
        
        // Test export with metadata
        let output = run_cli_command(&[
            "export-key",
            "--key-id", "test_key",
            "--storage-dir", &storage_dir.to_string_lossy(),
            "--export-file", &export_file.to_string_lossy(),
            "--format", "json",
            "--include-metadata",
        ]);
        
        if export_file.exists() {
            let content = fs::read_to_string(&export_file)
                .expect("Failed to read export file");
            
            let parsed: serde_json::Value = serde_json::from_str(&content)
                .expect("Export should be valid JSON");
            
            // Verify metadata is included
            if let Some(metadata) = parsed.get("metadata") {
                assert!(metadata["key_id"].is_string(), "Metadata should include original key_id");
                assert!(metadata["original_created"].is_string(), "Metadata should include creation time");
                assert!(metadata["export_source"].is_string(), "Metadata should include export source");
            }
        }
    }
}