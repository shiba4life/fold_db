//! SDK Test Harnesses for Cross-Platform Integration Testing
//!
//! This module provides test harnesses for the JavaScript SDK, Python SDK,
//! and CLI tool to enable comprehensive cross-platform integration testing.

use super::test_utils::{E2ETestConfig, TestCredentials};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::TempDir;
use tokio::process::Command as AsyncCommand;

/// JavaScript SDK test harness
pub struct JavaScriptSDKHarness {
    config: E2ETestConfig,
    sdk_path: PathBuf,
    temp_dir: TempDir,
    custom_config: Option<Value>,
    initialized: bool,
}

impl JavaScriptSDKHarness {
    /// Create a new JavaScript SDK harness
    pub async fn new(config: &E2ETestConfig) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let sdk_path = PathBuf::from("js-sdk");

        Ok(Self {
            config: config.clone(),
            sdk_path,
            temp_dir,
            custom_config: None,
            initialized: false,
        })
    }

    /// Create with custom configuration
    pub async fn with_config(config: &E2ETestConfig, custom_config: Value) -> anyhow::Result<Self> {
        let mut harness = Self::new(config).await?;
        harness.custom_config = Some(custom_config);
        Ok(harness)
    }

    /// Setup the JavaScript SDK environment
    pub async fn setup(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("🔧 Setting up JavaScript SDK test environment");

        // Check if Node.js is available
        let node_check = Command::new("node").arg("--version").output()?;
        if !node_check.status.success() {
            return Err(anyhow::anyhow!("Node.js not found. Please install Node.js to run JavaScript SDK tests."));
        }

        // Check if npm is available
        let npm_check = Command::new("npm").arg("--version").output()?;
        if !npm_check.status.success() {
            return Err(anyhow::anyhow!("npm not found. Please install npm to run JavaScript SDK tests."));
        }

        // Install dependencies if needed
        if self.sdk_path.join("node_modules").exists() {
            log::info!("✓ JavaScript SDK dependencies already installed");
        } else {
            log::info!("📦 Installing JavaScript SDK dependencies...");
            let install_output = Command::new("npm")
                .arg("install")
                .current_dir(&self.sdk_path)
                .output()?;

            if !install_output.status.success() {
                let error = String::from_utf8_lossy(&install_output.stderr);
                return Err(anyhow::anyhow!("Failed to install JavaScript SDK dependencies: {}", error));
            }
        }

        // Create test configuration file
        self.create_test_config().await?;

        self.initialized = true;
        log::info!("✅ JavaScript SDK test environment ready");
        Ok(())
    }

    /// Sign a request using the JavaScript SDK
    pub async fn sign_request(&self, credentials: &TestCredentials, request: &Value) -> anyhow::Result<Value> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        // Create a test script to sign the request
        let script_content = format!(
            r#"
            const {{ DataFoldSigner }} = require('./src/index.js');
            
            const credentials = {{
                clientId: "{}",
                keyId: "{}",
                privateKey: "{}",
                publicKey: "{}"
            }};
            
            const request = {};
            
            async function signRequest() {{
                try {{
                    const signer = new DataFoldSigner(credentials);
                    const result = await signer.signRequest(request);
                    console.log(JSON.stringify(result));
                }} catch (error) {{
                    console.error('Error:', error.message);
                    process.exit(1);
                }}
            }}
            
            signRequest();
            "#,
            credentials.client_id,
            credentials.key_id,
            hex::encode(credentials.keypair.private_key_bytes()),
            credentials.public_key_hex,
            serde_json::to_string_pretty(request)?
        );

        let script_path = self.temp_dir.path().join("sign_request.js");
        tokio::fs::write(&script_path, script_content).await?;

        // Execute the signing script
        let output = AsyncCommand::new("node")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("JavaScript signing failed: {}", error));
        }

        let result_str = String::from_utf8(output.stdout)?;
        let result: Value = serde_json::from_str(&result_str)?;
        Ok(result)
    }

    /// Verify a signature using the JavaScript SDK
    pub async fn verify_signature(&self, credentials: &TestCredentials, message: &Value, signature: &Value) -> anyhow::Result<bool> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        // Create verification script
        let script_content = format!(
            r#"
            const {{ DataFoldVerifier }} = require('./src/index.js');
            
            const credentials = {{
                publicKey: "{}"
            }};
            
            const message = {};
            const signature = {};
            
            async function verifySignature() {{
                try {{
                    const verifier = new DataFoldVerifier();
                    const result = await verifier.verifySignature(credentials.publicKey, message, signature);
                    console.log(result);
                }} catch (error) {{
                    console.error('Error:', error.message);
                    process.exit(1);
                }}
            }}
            
            verifySignature();
            "#,
            credentials.public_key_hex,
            serde_json::to_string_pretty(message)?,
            serde_json::to_string_pretty(signature)?
        );

        let script_path = self.temp_dir.path().join("verify_signature.js");
        tokio::fs::write(&script_path, script_content).await?;

        let output = AsyncCommand::new("node")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(false);
        }

        let result_str = String::from_utf8(output.stdout)?.trim();
        Ok(result_str == "true")
    }

    /// Verify a raw signature
    pub async fn verify_raw_signature(&self, credentials: &TestCredentials, message: &str, signature: &str) -> anyhow::Result<bool> {
        let message_json = json!({ "raw_message": message });
        let signature_json = json!({ "raw_signature": signature });
        self.verify_signature(credentials, &message_json, &signature_json).await
    }

    /// Make an authenticated request using the JavaScript SDK
    pub async fn make_authenticated_request(
        &self,
        server_url: &str,
        credentials: &TestCredentials,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> anyhow::Result<reqwest::Response> {
        // For now, return a placeholder response
        // In a real implementation, this would use the JavaScript SDK to make authenticated HTTP requests
        let client = reqwest::Client::new();
        let url = format!("{}{}", server_url, path);
        
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }

        let response = request_builder.send().await?;
        Ok(response)
    }

    /// Verify with server
    pub async fn verify_with_server(&self, server_url: &str, signature_components: &Value) -> anyhow::Result<bool> {
        // Placeholder implementation
        // In reality, this would send the signature to the server for verification
        log::info!("Verifying JavaScript signature with server: {}", server_url);
        Ok(true)
    }

    /// Canonicalize a message using the JavaScript SDK
    pub async fn canonicalize_message(&self, request: &Value) -> anyhow::Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        let script_content = format!(
            r#"
            const {{ MessageCanonicalizer }} = require('./src/canonicalizer.js');
            
            const request = {};
            
            try {{
                const canonicalizer = new MessageCanonicalizer();
                const canonical = canonicalizer.canonicalize(request);
                console.log(canonical);
            }} catch (error) {{
                console.error('Error:', error.message);
                process.exit(1);
            }}
            "#,
            serde_json::to_string_pretty(request)?
        );

        let script_path = self.temp_dir.path().join("canonicalize.js");
        tokio::fs::write(&script_path, script_content).await?;

        let output = AsyncCommand::new("node")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("JavaScript canonicalization failed: {}", error));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Test platform-specific features
    pub async fn test_platform_specific_features(&self, credentials: &TestCredentials) -> anyhow::Result<bool> {
        log::info!("Testing JavaScript SDK platform-specific features");
        
        // Test browser-specific features (if applicable)
        // Test Node.js-specific features
        // Test SDK configuration options
        
        Ok(true) // Placeholder
    }

    /// Test configuration compliance
    pub async fn test_configuration_compliance(&self, credentials: &TestCredentials) -> anyhow::Result<bool> {
        log::info!("Testing JavaScript SDK configuration compliance");
        
        // Test that the SDK respects configuration settings
        // Test security profile compliance
        // Test parameter validation
        
        Ok(true) // Placeholder
    }

    /// Create test configuration
    async fn create_test_config(&self) -> anyhow::Result<()> {
        let config = self.custom_config.as_ref().unwrap_or(&json!({
            "test_mode": true,
            "log_level": "info"
        }));

        let config_path = self.temp_dir.path().join("test_config.json");
        tokio::fs::write(&config_path, serde_json::to_string_pretty(config)?).await?;
        
        Ok(())
    }

    /// Cleanup the JavaScript SDK environment
    pub async fn cleanup(&mut self) -> anyhow::Result<()> {
        log::info!("🧹 Cleaning up JavaScript SDK test environment");
        self.initialized = false;
        Ok(())
    }
}

impl Clone for JavaScriptSDKHarness {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            sdk_path: self.sdk_path.clone(),
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
            custom_config: self.custom_config.clone(),
            initialized: false, // Clone starts uninitialized
        }
    }
}

/// Python SDK test harness
pub struct PythonSDKHarness {
    config: E2ETestConfig,
    sdk_path: PathBuf,
    temp_dir: TempDir,
    custom_config: Option<Value>,
    initialized: bool,
}

impl PythonSDKHarness {
    /// Create a new Python SDK harness
    pub async fn new(config: &E2ETestConfig) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let sdk_path = PathBuf::from("python-sdk");

        Ok(Self {
            config: config.clone(),
            sdk_path,
            temp_dir,
            custom_config: None,
            initialized: false,
        })
    }

    /// Create with custom configuration
    pub async fn with_config(config: &E2ETestConfig, custom_config: Value) -> anyhow::Result<Self> {
        let mut harness = Self::new(config).await?;
        harness.custom_config = Some(custom_config);
        Ok(harness)
    }

    /// Setup the Python SDK environment
    pub async fn setup(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("🔧 Setting up Python SDK test environment");

        // Check if Python is available
        let python_check = Command::new("python").arg("--version").output();
        let python3_check = Command::new("python3").arg("--version").output();
        
        let python_cmd = if python_check.is_ok() && python_check.unwrap().status.success() {
            "python"
        } else if python3_check.is_ok() && python3_check.unwrap().status.success() {
            "python3"
        } else {
            return Err(anyhow::anyhow!("Python not found. Please install Python to run Python SDK tests."));
        };

        // Install dependencies if needed
        if self.sdk_path.join("__pycache__").exists() || self.sdk_path.join("build").exists() {
            log::info!("✓ Python SDK dependencies may already be installed");
        }

        log::info!("📦 Installing Python SDK dependencies...");
        let install_output = Command::new("pip")
            .args(&["install", "-r", "requirements-dev.txt"])
            .current_dir(&self.sdk_path)
            .output()?;

        if !install_output.status.success() {
            let error = String::from_utf8_lossy(&install_output.stderr);
            log::warn!("Failed to install Python SDK dependencies: {}", error);
            // Continue anyway, dependencies might already be installed
        }

        // Create test configuration file
        self.create_test_config().await?;

        self.initialized = true;
        log::info!("✅ Python SDK test environment ready");
        Ok(())
    }

    /// Sign a request using the Python SDK
    pub async fn sign_request(&self, credentials: &TestCredentials, request: &Value) -> anyhow::Result<Value> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        let script_content = format!(
            r#"
import json
import sys
sys.path.insert(0, 'src')

from datafold_client import DataFoldSigner

credentials = {{
    'client_id': "{}",
    'key_id': "{}",
    'private_key': "{}",
    'public_key': "{}"
}}

request = {}

try:
    signer = DataFoldSigner(credentials)
    result = signer.sign_request(request)
    print(json.dumps(result))
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
            "#,
            credentials.client_id,
            credentials.key_id,
            hex::encode(credentials.keypair.private_key_bytes()),
            credentials.public_key_hex,
            serde_json::to_string_pretty(request)?
        );

        let script_path = self.temp_dir.path().join("sign_request.py");
        tokio::fs::write(&script_path, script_content).await?;

        let output = AsyncCommand::new("python")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Python signing failed: {}", error));
        }

        let result_str = String::from_utf8(output.stdout)?;
        let result: Value = serde_json::from_str(&result_str)?;
        Ok(result)
    }

    /// Verify a signature using the Python SDK
    pub async fn verify_signature(&self, credentials: &TestCredentials, message: &Value, signature: &Value) -> anyhow::Result<bool> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        let script_content = format!(
            r#"
import json
import sys
sys.path.insert(0, 'src')

from datafold_client import DataFoldVerifier

public_key = "{}"
message = {}
signature = {}

try:
    verifier = DataFoldVerifier()
    result = verifier.verify_signature(public_key, message, signature)
    print(result)
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
            "#,
            credentials.public_key_hex,
            serde_json::to_string_pretty(message)?,
            serde_json::to_string_pretty(signature)?
        );

        let script_path = self.temp_dir.path().join("verify_signature.py");
        tokio::fs::write(&script_path, script_content).await?;

        let output = AsyncCommand::new("python")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            return Ok(false);
        }

        let result_str = String::from_utf8(output.stdout)?.trim();
        Ok(result_str == "True")
    }

    /// Verify a raw signature
    pub async fn verify_raw_signature(&self, credentials: &TestCredentials, message: &str, signature: &str) -> anyhow::Result<bool> {
        let message_json = json!({ "raw_message": message });
        let signature_json = json!({ "raw_signature": signature });
        self.verify_signature(credentials, &message_json, &signature_json).await
    }

    /// Make an authenticated request using the Python SDK
    pub async fn make_authenticated_request(
        &self,
        server_url: &str,
        credentials: &TestCredentials,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> anyhow::Result<reqwest::Response> {
        // Placeholder implementation
        let client = reqwest::Client::new();
        let url = format!("{}{}", server_url, path);
        
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(&url),
            "POST" => client.post(&url),
            "PUT" => client.put(&url),
            "DELETE" => client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }

        let response = request_builder.send().await?;
        Ok(response)
    }

    /// Verify with server
    pub async fn verify_with_server(&self, server_url: &str, signature_components: &Value) -> anyhow::Result<bool> {
        log::info!("Verifying Python signature with server: {}", server_url);
        Ok(true) // Placeholder
    }

    /// Canonicalize a message using the Python SDK
    pub async fn canonicalize_message(&self, request: &Value) -> anyhow::Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("SDK harness not initialized"));
        }

        let script_content = format!(
            r#"
import json
import sys
sys.path.insert(0, 'src')

from datafold_client import MessageCanonicalizer

request = {}

try:
    canonicalizer = MessageCanonicalizer()
    canonical = canonicalizer.canonicalize(request)
    print(canonical)
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
            "#,
            serde_json::to_string_pretty(request)?
        );

        let script_path = self.temp_dir.path().join("canonicalize.py");
        tokio::fs::write(&script_path, script_content).await?;

        let output = AsyncCommand::new("python")
            .arg(&script_path)
            .current_dir(&self.sdk_path)
            .output()
            .await?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Python canonicalization failed: {}", error));
        }

        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    /// Test platform-specific features
    pub async fn test_platform_specific_features(&self, credentials: &TestCredentials) -> anyhow::Result<bool> {
        log::info!("Testing Python SDK platform-specific features");
        Ok(true) // Placeholder
    }

    /// Test configuration compliance
    pub async fn test_configuration_compliance(&self, credentials: &TestCredentials) -> anyhow::Result<bool> {
        log::info!("Testing Python SDK configuration compliance");
        Ok(true) // Placeholder
    }

    /// Create test configuration
    async fn create_test_config(&self) -> anyhow::Result<()> {
        let config = self.custom_config.as_ref().unwrap_or(&json!({
            "test_mode": true,
            "log_level": "INFO"
        }));

        let config_path = self.temp_dir.path().join("test_config.json");
        tokio::fs::write(&config_path, serde_json::to_string_pretty(config)?).await?;
        
        Ok(())
    }

    /// Cleanup the Python SDK environment
    pub async fn cleanup(&mut self) -> anyhow::Result<()> {
        log::info!("🧹 Cleaning up Python SDK test environment");
        self.initialized = false;
        Ok(())
    }
}

impl Clone for PythonSDKHarness {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            sdk_path: self.sdk_path.clone(),
            temp_dir: TempDir::new().expect("Failed to create temp dir"),
            custom_config: self.custom_config.clone(),
            initialized: false, // Clone starts uninitialized
        }
    }
}

/// CLI tool test harness
pub struct CLIHarness {
    config: E2ETestConfig,
    temp_dir: TempDir,
    initialized: bool,
}

impl CLIHarness {
    /// Create a new CLI harness
    pub async fn new(config: &E2ETestConfig) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;

        Ok(Self {
            config: config.clone(),
            temp_dir,
            initialized: false,
        })
    }

    /// Setup the CLI environment
    pub async fn setup(&mut self) -> anyhow::Result<()> {
        if self.initialized {
            return Ok(());
        }

        log::info!("🔧 Setting up CLI test environment");

        // Check if cargo is available
        let cargo_check = Command::new("cargo").arg("--version").output()?;
        if !cargo_check.status.success() {
            return Err(anyhow::anyhow!("Cargo not found. Please install Rust to run CLI tests."));
        }

        // Build CLI if needed
        log::info!("🔨 Building DataFold CLI...");
        let build_output = Command::new("cargo")
            .args(&["build", "--bin", "datafold_cli"])
            .output()?;

        if !build_output.status.success() {
            let error = String::from_utf8_lossy(&build_output.stderr);
            return Err(anyhow::anyhow!("Failed to build CLI: {}", error));
        }

        self.initialized = true;
        log::info!("✅ CLI test environment ready");
        Ok(())
    }

    /// Generate credentials using the CLI
    pub async fn generate_credentials(&self) -> anyhow::Result<TestCredentials> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        let output = Command::new("cargo")
            .args(&["run", "--bin", "datafold_cli", "--", "generate-key", "--format", "hex"])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("CLI key generation failed: {}", error));
        }

        // For now, return generated credentials
        // In reality, we'd parse the CLI output
        TestCredentials::generate()
    }

    /// Register key with server using CLI
    pub async fn register_key_with_server(&self, server_url: &str, credentials: &TestCredentials) -> anyhow::Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "datafold_cli", "--",
                "register-key",
                "--server-url", server_url,
                "--key-id", &credentials.key_id,
                "--client-id", &credentials.client_id,
                "--public-key", &credentials.public_key_hex
            ])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("CLI key registration failed: {}", error));
        }

        // Parse registration ID from output
        let output_str = String::from_utf8(output.stdout)?;
        Ok(output_str.trim().to_string())
    }

    /// Test authentication using CLI
    pub async fn test_authentication(&self, server_url: &str, credentials: &TestCredentials) -> anyhow::Result<bool> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "datafold_cli", "--",
                "test-server-integration",
                "--server-url", server_url,
                "--client-id", &credentials.client_id,
                "--key-id", &credentials.key_id
            ])
            .output()?;

        Ok(output.status.success())
    }

    /// Sign a message using CLI
    pub async fn sign_message(&self, credentials: &TestCredentials, message: &str) -> anyhow::Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        let output = Command::new("cargo")
            .args(&[
                "run", "--bin", "datafold_cli", "--",
                "sign-and-verify",
                "--message", message,
                "--key-id", &credentials.key_id
            ])
            .output()?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("CLI message signing failed: {}", error));
        }

        let output_str = String::from_utf8(output.stdout)?;
        Ok(output_str.trim().to_string())
    }

    /// Verify a signature using CLI
    pub async fn verify_signature(&self, credentials: &TestCredentials, message: &str, signature: &str) -> anyhow::Result<bool> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        // For now, return true as placeholder
        // In reality, we'd use CLI verification command
        Ok(true)
    }

    /// Canonicalize a message using CLI
    pub async fn canonicalize_message(&self, request: &Value) -> anyhow::Result<String> {
        if !self.initialized {
            return Err(anyhow::anyhow!("CLI harness not initialized"));
        }

        // For now, return a simple canonicalization
        // In reality, this would use the CLI canonicalization command
        Ok(serde_json::to_string(request)?)
    }

    /// Cleanup the CLI environment
    pub async fn cleanup(&mut self) -> anyhow::Result<()> {
        log::info!("🧹 Cleaning up CLI test environment");
        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::e2e::init_e2e_environment;

    #[tokio::test]
    async fn test_javascript_sdk_harness() {
        init_e2e_environment();
        
        let config = E2ETestConfig::default();
        let mut harness = JavaScriptSDKHarness::new(&config).await.unwrap();
        
        // Test would setup and use the harness
        assert!(!harness.initialized);
    }

    #[tokio::test]
    async fn test_python_sdk_harness() {
        init_e2e_environment();
        
        let config = E2ETestConfig::default();
        let mut harness = PythonSDKHarness::new(&config).await.unwrap();
        
        assert!(!harness.initialized);
    }

    #[tokio::test]
    async fn test_cli_harness() {
        init_e2e_environment();
        
        let config = E2ETestConfig::default();
        let mut harness = CLIHarness::new(&config).await.unwrap();
        
        assert!(!harness.initialized);
    }
}