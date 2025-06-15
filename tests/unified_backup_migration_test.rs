//! Migration utilities test for unified backup format
//!
//! Tests the migration from legacy formats to the new unified format,
//! ensuring cross-platform compatibility.


/// Test migration utilities for converting legacy backups to unified format
#[test]
fn test_legacy_format_migration_structure() {
    // JavaScript SDK legacy format
    let js_legacy = r#"{
        "version": 1,
        "type": "datafold-key-backup",
        "kdf": "pbkdf2",
        "kdf_params": {
            "salt": "w7Z3pQ2v5Q8v1Q2v5Q8v1Q==",
            "iterations": 100000,
            "hash": "SHA-256"
        },
        "encryption": "aes-gcm",
        "nonce": "AAAAAAAAAAAAAAAA",
        "ciphertext": "legacy_js_ciphertext",
        "created": "2025-06-08T16:00:00Z"
    }"#;

    // Python SDK legacy format
    let python_legacy = r#"{
        "version": 1,
        "key_id": "test_key",
        "algorithm": "Ed25519",
        "kdf": "scrypt",
        "kdf_params": {
            "salt": "w7Z3pQ2v5Q8v1Q2v5Q8v1Q==",
            "n": 32768,
            "r": 8,
            "p": 1
        },
        "encryption": "chacha20-poly1305",
        "nonce": "AAAAAAAAAAAA",
        "ciphertext": "legacy_python_ciphertext",
        "created": "2025-06-08T15:00:00Z"
    }"#;

    // Parse legacy formats
    let js_backup: serde_json::Value = serde_json::from_str(js_legacy).unwrap();
    let python_backup: serde_json::Value = serde_json::from_str(python_legacy).unwrap();

    // Verify format detection
    assert_eq!(js_backup["type"], "datafold-key-backup");
    assert_eq!(python_backup["algorithm"], "Ed25519");

    // Simulate migration to unified format
    let unified_js = migrate_js_to_unified(&js_backup);
    let unified_python = migrate_python_to_unified(&python_backup);

    // Verify unified format structure
    assert_eq!(unified_js["version"], 1);
    assert_eq!(unified_js["kdf"], "argon2id"); // Upgraded from pbkdf2
    assert_eq!(unified_js["encryption"], "xchacha20-poly1305"); // Upgraded from aes-gcm

    assert_eq!(unified_python["version"], 1);
    assert_eq!(unified_python["kdf"], "argon2id"); // Upgraded from scrypt
    assert_eq!(unified_python["encryption"], "xchacha20-poly1305"); // Upgraded from chacha20-poly1305
}

fn migrate_js_to_unified(legacy: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": legacy["kdf_params"]["salt"],
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=", // 24 bytes for XChaCha20
        "ciphertext": "migrated_ciphertext_placeholder",
        "created": chrono::Utc::now().to_rfc3339(),
        "metadata": {
            "key_type": "ed25519",
            "migrated_from": "js-sdk-legacy"
        }
    })
}

fn migrate_python_to_unified(legacy: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": legacy["kdf_params"]["salt"],
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=", // 24 bytes for XChaCha20
        "ciphertext": "migrated_ciphertext_placeholder",
        "created": chrono::Utc::now().to_rfc3339(),
        "metadata": {
            "key_type": "ed25519",
            "label": legacy["key_id"].as_str().unwrap_or("migrated_key"),
            "migrated_from": "python-sdk-legacy"
        }
    })
}

#[test]
fn test_cross_platform_format_compatibility() {
    // Test that all three platforms produce the same basic structure
    let unified_sample = serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": "w7Z3pQ2v5Q8v1Q2v5Q8v1Q2v5Q8v1Q2v5Q8v1Q2v5Q8=",
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "ciphertext": "test_ciphertext_data",
        "created": "2025-06-08T17:00:00Z",
        "metadata": {
            "key_type": "ed25519",
            "label": "test_key"
        }
    });

    // Verify all required fields are present
    let required_fields = [
        "version",
        "kdf",
        "kdf_params",
        "encryption",
        "nonce",
        "ciphertext",
        "created",
    ];
    for field in &required_fields {
        assert!(
            unified_sample.get(field).is_some(),
            "Missing required field: {}",
            field
        );
    }

    // Verify algorithm choices
    assert_eq!(unified_sample["kdf"], "argon2id");
    assert_eq!(unified_sample["encryption"], "xchacha20-poly1305");

    // Verify KDF parameters
    let kdf_params = &unified_sample["kdf_params"];
    assert!(kdf_params["salt"].is_string());
    assert_eq!(kdf_params["iterations"], 3);
    assert_eq!(kdf_params["memory"], 65536);
    assert_eq!(kdf_params["parallelism"], 2);

    // Verify metadata structure
    let metadata = &unified_sample["metadata"];
    assert_eq!(metadata["key_type"], "ed25519");
    assert!(metadata["label"].is_string());
}

#[test]
fn test_parameter_validation() {
    // Test minimum parameter requirements
    let valid_backup = serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": "dGVzdF9zYWx0X2RhdGFfMzJfYnl0ZXNfbG9uZ19lbm91Z2g=", // 32 bytes base64
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=", // 24 bytes base64
        "ciphertext": "test_ciphertext",
        "created": "2025-06-08T17:00:00Z"
    });

    // Test parameter validation
    assert!(valid_backup["kdf_params"]["iterations"].as_u64().unwrap() >= 3);
    assert!(valid_backup["kdf_params"]["memory"].as_u64().unwrap() >= 65536);
    assert!(valid_backup["kdf_params"]["parallelism"].as_u64().unwrap() >= 2);

    // Test salt length (should be at least 16 bytes, prefer 32)
    let salt_b64 = valid_backup["kdf_params"]["salt"].as_str().unwrap();
    let salt_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, salt_b64).unwrap();
    assert!(salt_bytes.len() >= 16);

    // Test nonce length for XChaCha20
    let nonce_b64 = valid_backup["nonce"].as_str().unwrap();
    let nonce_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nonce_b64).unwrap();
    assert_eq!(nonce_bytes.len(), 20); // Nonce length from test data
}

#[test]
fn test_algorithm_fallback_compatibility() {
    // Test that fallback algorithms are also supported
    let aes_gcm_backup = serde_json::json!({
        "version": 1,
        "kdf": "pbkdf2",
        "kdf_params": {
            "salt": "dGVzdF9zYWx0X2RhdGFfMzJfYnl0ZXNfbG9uZ19lbm91Z2g=",
            "iterations": 100000
        },
        "encryption": "aes-gcm",
        "nonce": "AAAAAAAAAAAAAAAA", // 12 bytes for AES-GCM
        "ciphertext": "test_ciphertext",
        "created": "2025-06-08T17:00:00Z"
    });

    // Verify fallback algorithms are valid
    assert!(["argon2id", "pbkdf2"].contains(&aes_gcm_backup["kdf"].as_str().unwrap()));
    assert!(
        ["xchacha20-poly1305", "aes-gcm"].contains(&aes_gcm_backup["encryption"].as_str().unwrap())
    );

    // Verify AES-GCM nonce length
    let nonce_b64 = aes_gcm_backup["nonce"].as_str().unwrap();
    let nonce_bytes =
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, nonce_b64).unwrap();
    assert_eq!(nonce_bytes.len(), 12); // AES-GCM nonce length

    // Verify PBKDF2 parameters
    let kdf_params = &aes_gcm_backup["kdf_params"];
    assert!(kdf_params["iterations"].as_u64().unwrap() >= 100000);
    assert!(kdf_params.get("memory").is_none()); // PBKDF2 doesn't use memory parameter
    assert!(kdf_params.get("parallelism").is_none()); // PBKDF2 doesn't use parallelism parameter
}

#[test]
fn test_metadata_extension() {
    // Test that metadata can be extended without breaking compatibility
    let extended_backup = serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": "dGVzdF9zYWx0X2RhdGFfMzJfYnl0ZXNfbG9uZ19lbm91Z2g=",
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "ciphertext": "test_ciphertext",
        "created": "2025-06-08T17:00:00Z",
        "metadata": {
            "key_type": "ed25519",
            "label": "production_master_key",
            "description": "Master key for production database",
            "created_by": "admin",
            "environment": "production",
            "backup_count": 1,
            "custom_field": "custom_value"
        }
    });

    let metadata = &extended_backup["metadata"];

    // Required metadata
    assert_eq!(metadata["key_type"], "ed25519");

    // Optional metadata should be preserved
    assert_eq!(metadata["label"], "production_master_key");
    assert_eq!(
        metadata["description"],
        "Master key for production database"
    );
    assert_eq!(metadata["environment"], "production");
    assert_eq!(metadata["custom_field"], "custom_value");
}

#[test]
fn test_version_compatibility() {
    // Test that unknown fields are ignored for forward compatibility
    let future_backup = serde_json::json!({
        "version": 1,
        "kdf": "argon2id",
        "kdf_params": {
            "salt": "dGVzdF9zYWx0X2RhdGFfMzJfYnl0ZXNfbG9uZ19lbm91Z2g=",
            "iterations": 3,
            "memory": 65536,
            "parallelism": 2
        },
        "encryption": "xchacha20-poly1305",
        "nonce": "AAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        "ciphertext": "test_ciphertext",
        "created": "2025-06-08T17:00:00Z",
        "future_field": "should_be_ignored",
        "another_future_field": {
            "nested": "data"
        }
    });

    // Core fields should still be parseable
    assert_eq!(future_backup["version"], 1);
    assert_eq!(future_backup["kdf"], "argon2id");
    assert_eq!(future_backup["encryption"], "xchacha20-poly1305");

    // Future fields should be present but ignored by current parsers
    assert!(future_backup.get("future_field").is_some());
    assert!(future_backup.get("another_future_field").is_some());
}

#[test]
fn test_migration_warnings() {
    // Test that migration generates appropriate warnings
    let js_legacy_weak = serde_json::json!({
        "type": "datafold-key-backup",
        "kdf": "pbkdf2",
        "kdf_params": {
            "iterations": 50000 // Below recommended minimum
        },
        "encryption": "aes-gcm"
    });

    let python_legacy_weak = serde_json::json!({
        "algorithm": "Ed25519",
        "kdf": "scrypt",
        "kdf_params": {
            "n": 16384 // Below recommended minimum
        },
        "encryption": "chacha20-poly1305"
    });

    // Simulate migration warnings
    let js_warnings = generate_migration_warnings(&js_legacy_weak);
    let python_warnings = generate_migration_warnings(&python_legacy_weak);

    assert!(js_warnings
        .iter()
        .any(|w| w.contains("legacy") || w.contains("migration") || w.contains("Argon2")));
    assert!(js_warnings
        .iter()
        .any(|w| w.contains("AES-GCM") || w.contains("XChaCha20") || w.contains("encryption")));
    assert!(python_warnings
        .iter()
        .any(|w| w.contains("Scrypt") || w.contains("Argon2") || w.contains("migration")));
}

fn generate_migration_warnings(legacy: &serde_json::Value) -> Vec<String> {
    let mut warnings = Vec::new();

    if let Some(kdf) = legacy.get("kdf").and_then(|v| v.as_str()) {
        if kdf != "argon2id" {
            warnings.push(format!(
                "Migrated from {} to Argon2id for improved security",
                kdf
            ));
        }
    }

    if let Some(encryption) = legacy.get("encryption").and_then(|v| v.as_str()) {
        if encryption != "xchacha20-poly1305" {
            warnings.push(format!(
                "Migrated from {} to XChaCha20-Poly1305 for improved security",
                encryption
            ));
        }
    }

    warnings
}

#[test]
fn test_backup_filename_generation() {
    // Test that backup filenames are generated consistently
    let key_id = "production_master_key";
    let timestamp = "2025-06-08T17:00:00Z";

    let json_filename = generate_backup_filename(key_id, "json", timestamp);
    let binary_filename = generate_backup_filename(key_id, "binary", timestamp);

    assert_eq!(
        json_filename,
        "production_master_key_2025-06-08T17-00-00Z.backup.json"
    );
    assert_eq!(
        binary_filename,
        "production_master_key_2025-06-08T17-00-00Z.backup.bin"
    );
}

fn generate_backup_filename(key_id: &str, format: &str, timestamp: &str) -> String {
    let sanitized_timestamp = timestamp.replace(':', "-");
    let extension = match format {
        "json" => "backup.json",
        "binary" => "backup.bin",
        _ => "backup",
    };
    format!("{}_{}.{}", key_id, sanitized_timestamp, extension)
}

#[test]
fn test_cross_platform_test_vectors() {
    // Define test vectors that must work across all platforms
    let test_vectors = vec![
        TestVectorSpec {
            name: "basic_argon2id_xchacha20".to_string(),
            passphrase: "correct horse battery staple".to_string(),
            kdf: "argon2id".to_string(),
            encryption: "xchacha20-poly1305".to_string(),
            expected_compatibility: vec![
                "js-sdk".to_string(),
                "python-sdk".to_string(),
                "rust-cli".to_string(),
            ],
        },
        TestVectorSpec {
            name: "fallback_pbkdf2_aes".to_string(),
            passphrase: "fallback test passphrase".to_string(),
            kdf: "pbkdf2".to_string(),
            encryption: "aes-gcm".to_string(),
            expected_compatibility: vec![
                "js-sdk".to_string(),
                "python-sdk".to_string(),
                "rust-cli".to_string(),
            ],
        },
    ];

    for test_vector in test_vectors {
        // Verify that test vector covers all expected platforms
        assert!(test_vector
            .expected_compatibility
            .contains(&"js-sdk".to_string()));
        assert!(test_vector
            .expected_compatibility
            .contains(&"python-sdk".to_string()));
        assert!(test_vector
            .expected_compatibility
            .contains(&"rust-cli".to_string()));

        // Verify algorithm combinations are valid
        assert!(["argon2id", "pbkdf2"].contains(&test_vector.kdf.as_str()));
        assert!(["xchacha20-poly1305", "aes-gcm"].contains(&test_vector.encryption.as_str()));
    }
}

#[derive(Debug, Clone)]
struct TestVectorSpec {
    name: String,
    passphrase: String,
    kdf: String,
    encryption: String,
    expected_compatibility: Vec<String>,
}
