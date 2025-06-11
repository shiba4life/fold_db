//! Test utilities for E2E testing framework

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration for E2E tests
#[derive(Debug, Clone)]
pub struct E2ETestConfig {
    pub server_url: String,
    pub test_timeout_secs: u64,
    pub concurrent_clients: usize,
    pub enable_attack_simulation: bool,
    pub temp_dir: PathBuf,
}

impl E2ETestConfig {
    pub fn quick() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            test_timeout_secs: 30,
            concurrent_clients: 5,
            enable_attack_simulation: false,
            temp_dir: std::env::temp_dir(),
        }
    }

    pub fn comprehensive() -> Self {
        Self {
            server_url: "http://localhost:8080".to_string(),
            test_timeout_secs: 120,
            concurrent_clients: 10,
            enable_attack_simulation: true,
            temp_dir: std::env::temp_dir(),
        }
    }
}

/// Test credentials for authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCredentials {
    pub username: String,
    pub private_key: String,
    pub public_key: String,
}

impl TestCredentials {
    pub fn new(username: String, private_key: String, public_key: String) -> Self {
        Self {
            username,
            private_key,
            public_key,
        }
    }

    pub fn default_test_credentials() -> Self {
        Self {
            username: "test_user".to_string(),
            private_key: "test_private_key".to_string(),
            public_key: "test_public_key".to_string(),
        }
    }
}

/// HTTP client for E2E testing
pub struct E2EHttpClient {
    client: reqwest::Client,
    base_url: String,
    credentials: Option<TestCredentials>,
}

impl E2EHttpClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            credentials: None,
        }
    }

    pub fn with_credentials(mut self, credentials: TestCredentials) -> Self {
        self.credentials = Some(credentials);
        self
    }

    pub async fn get(&self, path: &str) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        self.client.get(&url).send().await
    }

    pub async fn post(&self, path: &str, body: serde_json::Value) -> Result<reqwest::Response, reqwest::Error> {
        let url = format!("{}{}", self.base_url, path);
        self.client.post(&url).json(&body).send().await
    }
}

/// Test scenario definition
#[derive(Debug, Clone)]
pub struct TestScenario {
    pub name: String,
    pub description: String,
    pub steps: Vec<TestStep>,
    pub clients: Vec<TestCredentials>,
    pub expectations: HashMap<String, bool>,
}

#[derive(Debug, Clone)]
pub struct TestStep {
    pub name: String,
    pub action: TestAction,
    pub expected_result: ExpectedResult,
}

#[derive(Debug, Clone)]
pub enum TestAction {
    HttpGet(String),
    HttpPost(String, serde_json::Value),
    Sleep(u64),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum ExpectedResult {
    HttpStatus(u16),
    JsonResponse(serde_json::Value),
    Success,
    Failure,
}

impl TestScenario {
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            steps: Vec::new(),
            clients: Vec::new(),
            expectations: HashMap::new(),
        }
    }

    pub fn add_step(mut self, step: TestStep) -> Self {
        self.steps.push(step);
        self
    }

    pub fn generate_clients(&mut self, count: usize) -> Result<(), Box<dyn std::error::Error>> {
        for i in 0..count {
            let credentials = TestCredentials::new(
                format!("test_user_{}", i),
                format!("test_private_key_{}", i),
                format!("test_public_key_{}", i),
            );
            self.clients.push(credentials);
        }
        Ok(())
    }

    pub fn expect(&mut self, key: &str, value: bool) {
        self.expectations.insert(key.to_string(), value);
    }

    pub async fn execute(&self, client: &E2EHttpClient) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Executing scenario: {}", self.name);
        
        for step in &self.steps {
            log::debug!("Executing step: {}", step.name);
            
            match &step.action {
                TestAction::HttpGet(path) => {
                    let response = client.get(path).await?;
                    self.validate_response(&step.expected_result, response).await?;
                }
                TestAction::HttpPost(path, body) => {
                    let response = client.post(path, body.clone()).await?;
                    self.validate_response(&step.expected_result, response).await?;
                }
                TestAction::Sleep(duration) => {
                    tokio::time::sleep(tokio::time::Duration::from_secs(*duration)).await;
                }
                TestAction::Custom(action) => {
                    log::info!("Custom action: {}", action);
                }
            }
        }
        
        Ok(())
    }

    async fn validate_response(&self, expected: &ExpectedResult, response: reqwest::Response) -> Result<(), Box<dyn std::error::Error>> {
        match expected {
            ExpectedResult::HttpStatus(status) => {
                if response.status().as_u16() != *status {
                    return Err(format!("Expected status {}, got {}", status, response.status()).into());
                }
            }
            ExpectedResult::JsonResponse(expected_json) => {
                let response_json: serde_json::Value = response.json().await?;
                if response_json != *expected_json {
                    return Err("JSON response mismatch".to_string().into());
                }
            }
            ExpectedResult::Success => {
                if !response.status().is_success() {
                    return Err(format!("Expected success, got status {}", response.status()).into());
                }
            }
            ExpectedResult::Failure => {
                if response.status().is_success() {
                    return Err(format!("Expected failure, got success status {}", response.status()).into());
                }
            }
        }
        Ok(())
    }
}

/// Utility functions for E2E testing
pub mod utils {
    use super::*;

    pub fn create_basic_test_scenario() -> TestScenario {
        TestScenario::new(
            "Basic API Test",
            "Tests basic API functionality",
        )
        .add_step(TestStep {
            name: "Health check".to_string(),
            action: TestAction::HttpGet("/health".to_string()),
            expected_result: ExpectedResult::HttpStatus(200),
        })
    }

    pub fn create_authentication_test_scenario() -> TestScenario {
        TestScenario::new(
            "Authentication Test",
            "Tests authentication flow",
        )
        .add_step(TestStep {
            name: "Login".to_string(),
            action: TestAction::HttpPost(
                "/auth/login".to_string(),
                serde_json::json!({
                    "username": "test_user",
                    "password": "test_password"
                })
            ),
            expected_result: ExpectedResult::Success,
        })
    }

    pub async fn setup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Setting up test environment");
        // Add any environment setup logic here
        Ok(())
    }

    pub async fn cleanup_test_environment() -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Cleaning up test environment");
        // Add any cleanup logic here
        Ok(())
    }
}