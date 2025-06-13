//! Integration tests for CLI server integration functionality

use reqwest;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;
use tokio;

/// Test CLI server integration end-to-end workflow
#[tokio::test]
async fn test_cli_server_integration_workflow() {
    // Skip if no test server is running
    if !is_test_server_available().await {
        println!("Skipping test - no test server available at localhost:8080");
        return;
    }

    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let storage_dir = temp_dir.path().join("keys");

    // Test key generation
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "generate-key",
            "--format",
            "hex",
            "--public-only",
        ])
        .output()
        .expect("Failed to execute generate-key command");

    assert!(output.status.success(), "Key generation failed");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.len() > 60, "Public key should be 64 hex characters");

    // Test key storage
    let store_output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "store-key",
            "--key-id",
            "test_integration_key",
            "--storage-dir",
            storage_dir.to_str().unwrap(),
            "--force",
        ])
        .output();

    // Note: This would require interactive input in real usage
    // For automated testing, we'd need a way to provide the passphrase
    println!("Store key test would require interactive passphrase input");
}

/// Test CLI help for server integration commands
#[test]
fn test_cli_server_commands_help() {
    // Test register-key help
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "register-key",
            "--help",
        ])
        .output()
        .expect("Failed to execute register-key help");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Register public key with DataFold server"));
    assert!(stdout.contains("--server-url"));
    assert!(stdout.contains("--key-id"));

    // Test check-registration help
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "check-registration",
            "--help",
        ])
        .output()
        .expect("Failed to execute check-registration help");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Check public key registration status"));
    assert!(stdout.contains("--client-id"));

    // Test sign-and-verify help
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "sign-and-verify",
            "--help",
        ])
        .output()
        .expect("Failed to execute sign-and-verify help");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Sign a message and verify with server"));
    assert!(stdout.contains("--message"));

    // Test test-server-integration help
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--bin",
            "datafold_cli",
            "--",
            "test-server-integration",
            "--help",
        ])
        .output()
        .expect("Failed to execute test-server-integration help");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Test end-to-end workflow"));
    assert!(stdout.contains("--test-message"));
}

/// Test CLI command validation
#[test]
fn test_cli_server_command_validation() {
    // Test register-key without required parameters
    let output = Command::new("cargo")
        .args(&["run", "--bin", "datafold_cli", "--", "register-key"])
        .output()
        .expect("Failed to execute register-key command");

    // Should fail due to missing required key-id
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("key-id") || stderr.contains("required"));

    // Test check-registration without required parameters
    let output = Command::new("cargo")
        .args(&["run", "--bin", "datafold_cli", "--", "check-registration"])
        .output()
        .expect("Failed to execute check-registration command");

    // Should fail due to missing required client-id
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("client-id") || stderr.contains("required"));
}

/// Check if test server is available for integration testing
async fn is_test_server_available() -> bool {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
        .unwrap();

    match client
        .get("http://localhost:8080/api/crypto/status")
        .send()
        .await
    {
        Ok(response) => response.status().is_success(),
        Err(_) => false,
    }
}

/// Test HTTP client helper functions (unit test for internal functions)
mod http_client_tests {
    use super::*;

    #[tokio::test]
    async fn test_http_client_timeout() {
        // Test that HTTP client respects timeout settings
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1))
            .build()
            .unwrap();

        // This should timeout quickly
        let result = client.get("http://httpbin.org/delay/5").send().await;
        assert!(result.is_err());
    }

    #[test]
    fn test_api_response_structures() {
        // Test API response structures can be properly serialized/deserialized
        use serde_json;

        let api_response = json!({
            "success": true,
            "data": {
                "registration_id": "test-123",
                "client_id": "client-456",
                "public_key": "abcd1234",
                "status": "active",
                "registered_at": "2024-01-01T00:00:00Z"
            }
        });

        // Should be able to parse this as a typical server response
        let parsed: serde_json::Value = serde_json::from_value(api_response).unwrap();
        assert!(parsed["success"].as_bool().unwrap());
        assert_eq!(parsed["data"]["status"].as_str().unwrap(), "active");
    }
}
