//! Test utilities for E2E integration testing
//!
//! This module provides common utilities, test fixtures, and helper functions
//! for end-to-end integration testing of the DataFold authentication system.

use datafold::crypto::ed25519::{generate_master_keypair, MasterKeyPair, PublicKey};
use datafold::datafold_node::config::NodeConfig;
use datafold::datafold_node::signature_auth::{SignatureAuthConfig, SecurityProfile};
use datafold::datafold_node::crypto_routes::PublicKeyRegistrationRequest;
use reqwest::{Client, Response};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tempfile::TempDir;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

/// Test environment configuration for E2E tests
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub server_url: String,
    pub test_timeout_secs: u64,
    pub concurrent_clients: usize,
    pub enable_attack_simulation: bool,
    pub temp_dir: PathBuf,
}

impl Default for E2ETestConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            test_timeout_secs: 60,
            concurrent_clients: 10,
            enable_attack_simulation: false,
            temp_dir: std::env::temp_dir(),
        }
    }
}

/// Test client credentials for authentication testing
#[derive(Debug, Clone)]
pub struct TestCredentials {
    pub client_id: String,
    pub key_id: String,
    pub user_id: String,
    pub keypair: MasterKeyPair,
    pub public_key_hex: String,
    pub registration_id: Option<String>,
}

impl TestCredentials {
    /// Generate new test credentials
    pub fn generate() -> anyhow::Result<Self> {
        let keypair = generate_master_keypair()?;
        let public_key_hex = hex::encode(keypair.public_key_bytes());
        let client_id = format!("test_client_{}", Uuid::new_v4());
        let key_id = format!("test_key_{}", Uuid::new_v4());
        let user_id = format!("test_user_{}", Uuid::new_v4());

        Ok(Self {
            client_id,
            key_id,
            user_id,
            keypair,
            public_key_hex,
            registration_id: None,
        })
    }

    /// Create public key registration request
    pub fn registration_request(&self) -> PublicKeyRegistrationRequest {
        PublicKeyRegistrationRequest {
            client_id: Some(self.client_id.clone()),
            user_id: Some(self.user_id.clone()),
            public_key: self.public_key_hex.clone(),
            key_name: Some(format!("E2E Test Key - {}", self.key_id)),
            metadata: Some(json!({
                "test_suite": "e2e_integration",
                "created_at": SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "test_type": "end_to_end"
            })),
        }
    }
}

/// Test server configuration builder
pub struct TestServerConfig {
    pub temp_dir: TempDir,
    pub config: NodeConfig,
}

impl TestServerConfig {
    /// Create a new test server configuration with signature authentication
    pub fn new_with_auth(security_profile: SecurityProfile) -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let mut config = match security_profile {
            SecurityProfile::Strict => NodeConfig::production_with_signature_auth(temp_dir.path().to_path_buf()),
            SecurityProfile::Standard => NodeConfig::with_optional_signature_auth(temp_dir.path().to_path_buf()),
            SecurityProfile::Lenient => NodeConfig::development_with_signature_auth(temp_dir.path().to_path_buf()),
        };

        // Customize for testing
        if let Some(ref mut auth_config) = config.signature_auth_config {
            auth_config.enabled = true;
            auth_config.log_replay_attempts = true;
            auth_config.security_logging.log_successful_auth = true;
        }

        Ok(Self { temp_dir, config })
    }

    /// Create configuration for cross-platform testing
    pub fn for_cross_platform_testing() -> anyhow::Result<Self> {
        let temp_dir = TempDir::new()?;
        let mut config = NodeConfig::with_optional_signature_auth(temp_dir.path().to_path_buf());

        if let Some(ref mut auth_config) = config.signature_auth_config {
            auth_config.enabled = true;
            auth_config.security_profile = SecurityProfile::Standard;
            auth_config.allowed_time_window_secs = 600; // Generous for cross-platform timing
            auth_config.clock_skew_tolerance_secs = 60;
            auth_config.enforce_rfc3339_timestamps = false; // Allow flexibility
            auth_config.require_uuid4_nonces = false;
        }

        Ok(Self { temp_dir, config })
    }
}

/// HTTP client for E2E testing with authentication support
pub struct E2EHttpClient {
    client: Client,
    base_url: String,
    credentials: Option<TestCredentials>,
}

impl E2EHttpClient {
    /// Create a new HTTP client for testing
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            credentials: None,
        }
    }

    /// Set authentication credentials
    pub fn with_credentials(mut self, credentials: TestCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Register public key with the server
    pub async fn register_public_key(&mut self, credentials: &TestCredentials) -> anyhow::Result<String> {
        let registration_request = credentials.registration_request();
        
        let response = self.client
            .post(&format!("{}/api/crypto/keys/register", self.base_url))
            .json(&registration_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow::anyhow!("Failed to register public key: {}", error_text));
        }

        let result: Value = response.json().await?;
        let registration_id = result["registration_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("No registration_id in response"))?
            .to_string();

        Ok(registration_id)
    }

    /// Make an authenticated request (placeholder for actual signing implementation)
    pub async fn authenticated_request(
        &self,
        method: &str,
        path: &str,
        body: Option<Value>,
    ) -> anyhow::Result<Response> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => self.client.get(&url),
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "DELETE" => self.client.delete(&url),
            _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
        };

        if let Some(body) = body {
            request_builder = request_builder.json(&body);
        }

        // TODO: Add actual signature generation here
        // For now, this is a placeholder that will be integrated with the SDK implementations
        
        let response = request_builder.send().await?;
        Ok(response)
    }

    /// Check server health
    pub async fn health_check(&self) -> anyhow::Result<bool> {
        let response = timeout(
            Duration::from_secs(5),
            self.client.get(&format!("{}/api/system/status", self.base_url)).send()
        ).await??;

        Ok(response.status().is_success())
    }

    /// Wait for server to become available
    pub async fn wait_for_server(&self, max_attempts: u32) -> anyhow::Result<()> {
        for attempt in 1..=max_attempts {
            if self.health_check().await.unwrap_or(false) {
                log::info!("Server is available after {} attempts", attempt);
                return Ok(());
            }
            
            if attempt < max_attempts {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        Err(anyhow::anyhow!("Server did not become available after {} attempts", max_attempts))
    }
}

/// Test scenario builder for complex E2E testing
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub clients: Vec<TestCredentials>,
    pub expected_outcomes: HashMap<String, bool>,
    pub performance_thresholds: HashMap<String, f64>,
}

impl TestScenario {
    /// Create a new test scenario
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            clients: Vec::new(),
            expected_outcomes: HashMap::new(),
            performance_thresholds: HashMap::new(),
        }
    }

    /// Add a test client to the scenario
    pub fn add_client(&mut self, credentials: TestCredentials) {
        self.clients.push(credentials);
    }

    /// Set expected outcome for a test
    pub fn expect(&mut self, test_name: &str, should_pass: bool) {
        self.expected_outcomes.insert(test_name.to_string(), should_pass);
    }

    /// Set performance threshold
    pub fn set_threshold(&mut self, metric: &str, threshold: f64) {
        self.performance_thresholds.insert(metric.to_string(), threshold);
    }

    /// Generate multiple test clients
    pub fn generate_clients(&mut self, count: usize) -> anyhow::Result<()> {
        for _ in 0..count {
            let credentials = TestCredentials::generate()?;
            self.clients.push(credentials);
        }
        Ok(())
    }
}

/// Test result collection and analysis
#[derive(Debug, Default)]
pub struct E2ETestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub performance_metrics: HashMap<String, f64>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl E2ETestResults {
    /// Add a test result
    pub fn add_result(&mut self, test_name: &str, passed: bool, error: Option<String>) {
        self.total_tests += 1;
        
        if passed {
            self.passed_tests += 1;
        } else {
            self.failed_tests += 1;
            if let Some(error) = error {
                self.errors.push(format!("{}: {}", test_name, error));
            }
        }
    }

    /// Add performance metric
    pub fn add_metric(&mut self, metric_name: &str, value: f64) {
        self.performance_metrics.insert(metric_name.to_string(), value);
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_tests == 0 {
            0.0
        } else {
            self.passed_tests as f64 / self.total_tests as f64
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed_tests == 0 && self.total_tests > 0
    }
}

/// Utility functions for E2E testing
pub mod utils {
    use super::*;

    /// Create a test timestamp (current time)
    pub fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Create a test nonce
    pub fn generate_nonce() -> String {
        Uuid::new_v4().to_string().replace('-', "")
    }

    /// Create test HTTP headers
    pub fn test_headers() -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());
        headers.insert("user-agent".to_string(), "DataFold-E2E-Test/1.0".to_string());
        headers.insert("accept".to_string(), "application/json".to_string());
        headers
    }

    /// Wait for condition with timeout
    pub async fn wait_for_condition<F, Fut>(
        condition: F,
        timeout_secs: u64,
        check_interval_ms: u64,
    ) -> anyhow::Result<()>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = bool>,
    {
        let start = std::time::Instant::now();
        let timeout_duration = Duration::from_secs(timeout_secs);

        while start.elapsed() < timeout_duration {
            if condition().await {
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(check_interval_ms)).await;
        }

        Err(anyhow::anyhow!("Condition not met within {} seconds", timeout_secs))
    }

    /// Measure execution time of an async operation
    pub async fn measure_time<F, Fut, T>(operation: F) -> (T, Duration)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let start = std::time::Instant::now();
        let result = operation().await;
        let duration = start.elapsed();
        (result, duration)
    }

    /// Generate test data of specified size
    pub fn generate_test_data(size_bytes: usize) -> Vec<u8> {
        (0..size_bytes).map(|i| (i % 256) as u8).collect()
    }

    /// Create a test JSON payload
    pub fn test_json_payload(size_hint: usize) -> Value {
        let data = "x".repeat(size_hint.saturating_sub(100)); // Account for JSON overhead
        json!({
            "test_data": data,
            "timestamp": current_timestamp(),
            "nonce": generate_nonce(),
            "metadata": {
                "test_suite": "e2e_integration",
                "data_size": size_hint
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_generation() {
        let credentials = TestCredentials::generate().unwrap();
        
        assert!(!credentials.client_id.is_empty());
        assert!(!credentials.key_id.is_empty());
        assert!(!credentials.user_id.is_empty());
        assert!(!credentials.public_key_hex.is_empty());
        assert_eq!(credentials.public_key_hex.len(), 64); // 32 bytes * 2 for hex
    }

    #[test]
    fn test_scenario_builder() {
        let mut scenario = TestScenario::new("test", "Test scenario");
        scenario.generate_clients(3).unwrap();
        scenario.expect("authentication", true);
        scenario.set_threshold("response_time_ms", 100.0);
        
        assert_eq!(scenario.clients.len(), 3);
        assert_eq!(scenario.expected_outcomes.get("authentication"), Some(&true));
        assert_eq!(scenario.performance_thresholds.get("response_time_ms"), Some(&100.0));
    }

    #[test]
    fn test_results_collection() {
        let mut results = E2ETestResults::default();
        
        results.add_result("test1", true, None);
        results.add_result("test2", false, Some("Failed".to_string()));
        results.add_metric("latency", 50.0);
        
        assert_eq!(results.total_tests, 2);
        assert_eq!(results.passed_tests, 1);
        assert_eq!(results.failed_tests, 1);
        assert_eq!(results.success_rate(), 0.5);
        assert!(!results.all_passed());
    }
}