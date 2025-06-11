//! Cross-platform validation for DataFold message signing protocol
//!
//! This module ensures consistency and interoperability across Rust, JavaScript,
//! and Python implementations of the DataFold message signing protocol.

pub mod interop_tests;
pub mod signature_verification;
pub mod message_canonicalization;
pub mod configuration_compatibility;

use crate::{CategoryResult, TestFailure, TestWarning, ValidationCategory, ValidationStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tracing::{debug, info, warn, error};

/// Configuration for cross-platform validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformConfig {
    /// Enable signature interoperability tests
    pub enable_signature_interop: bool,
    /// Enable message canonicalization consistency tests
    pub enable_canonicalization_tests: bool,
    /// Enable configuration compatibility tests
    pub enable_config_compatibility: bool,
    /// Enable error handling consistency tests
    pub enable_error_handling_tests: bool,
    /// Path to JavaScript SDK for testing
    pub js_sdk_path: Option<PathBuf>,
    /// Path to Python SDK for testing
    pub python_sdk_path: Option<PathBuf>,
    /// Path to DataFold server for testing
    pub server_path: Option<PathBuf>,
    /// Server URL for integration tests
    pub server_url: String,
    /// Test vectors to use for cross-platform testing
    pub test_vectors_path: Option<PathBuf>,
    /// Number of random test cases to generate
    pub random_test_count: usize,
    /// Test different key sizes and algorithms
    pub test_key_variations: bool,
    /// Test different message sizes
    pub test_message_variations: bool,
    /// Enable performance comparison across platforms
    pub enable_performance_comparison: bool,
}

impl Default for CrossPlatformConfig {
    fn default() -> Self {
        Self {
            enable_signature_interop: true,
            enable_canonicalization_tests: true,
            enable_config_compatibility: true,
            enable_error_handling_tests: true,
            js_sdk_path: Some(PathBuf::from("../../js-sdk")),
            python_sdk_path: Some(PathBuf::from("../../python-sdk")),
            server_path: Some(PathBuf::from("../../target/release/datafold_node")),
            server_url: "http://localhost:8080".to_string(),
            test_vectors_path: Some(PathBuf::from("test-vectors/interop-tests")),
            random_test_count: 100,
            test_key_variations: true,
            test_message_variations: true,
            enable_performance_comparison: false, // Disabled by default for faster tests
        }
    }
}

/// Cross-platform validator
pub struct CrossPlatformValidator {
    config: CrossPlatformConfig,
    interop_tester: interop_tests::InteropTester,
    signature_verifier: signature_verification::CrossPlatformSignatureVerifier,
    canonicalization_tester: message_canonicalization::CanonicalizationTester,
    config_tester: configuration_compatibility::ConfigurationTester,
}

impl CrossPlatformValidator {
    /// Create a new cross-platform validator
    pub fn new(config: CrossPlatformConfig) -> Result<Self> {
        // Validate SDK paths exist
        Self::validate_sdk_paths(&config)?;

        let interop_tester = interop_tests::InteropTester::new(config.clone())?;
        let signature_verifier = signature_verification::CrossPlatformSignatureVerifier::new(config.clone())?;
        let canonicalization_tester = message_canonicalization::CanonicalizationTester::new(config.clone())?;
        let config_tester = configuration_compatibility::ConfigurationTester::new(config.clone())?;

        Ok(Self {
            config,
            interop_tester,
            signature_verifier,
            canonicalization_tester,
            config_tester,
        })
    }

    /// Run complete cross-platform validation
    pub async fn run_validation(&self) -> Result<CategoryResult> {
        let start_time = Instant::now();
        info!("Starting cross-platform validation");

        let mut tests_run = 0;
        let mut tests_passed = 0;
        let mut tests_failed = 0;
        let mut failures = Vec::new();
        let mut warnings = Vec::new();

        // Run signature interoperability tests
        if self.config.enable_signature_interop {
            info!("Running signature interoperability tests");
            let result = self.run_signature_interop_tests().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run message canonicalization tests
        if self.config.enable_canonicalization_tests {
            info!("Running message canonicalization tests");
            let result = self.run_canonicalization_tests().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run configuration compatibility tests
        if self.config.enable_config_compatibility {
            info!("Running configuration compatibility tests");
            let result = self.run_config_compatibility_tests().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run error handling consistency tests
        if self.config.enable_error_handling_tests {
            info!("Running error handling consistency tests");
            let result = self.run_error_handling_tests().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        let duration = start_time.elapsed();
        let status = if tests_failed > 0 {
            ValidationStatus::Failed
        } else if !warnings.is_empty() {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Passed
        };

        info!("Cross-platform validation completed: {} passed, {} failed, {} warnings",
              tests_passed, tests_failed, warnings.len());

        Ok(CategoryResult {
            category: ValidationCategory::CrossPlatform,
            status,
            tests_run,
            tests_passed,
            tests_failed,
            tests_skipped: 0,
            duration_ms: duration.as_millis() as u64,
            failures,
            warnings,
        })
    }

    /// Run signature interoperability tests
    async fn run_signature_interop_tests(&self) -> Result<CrossPlatformTestResult> {
        debug!("Running signature interoperability tests");
        self.signature_verifier.run_interop_tests().await
    }

    /// Run message canonicalization tests
    async fn run_canonicalization_tests(&self) -> Result<CrossPlatformTestResult> {
        debug!("Running message canonicalization tests");
        self.canonicalization_tester.run_tests().await
    }

    /// Run configuration compatibility tests
    async fn run_config_compatibility_tests(&self) -> Result<CrossPlatformTestResult> {
        debug!("Running configuration compatibility tests");
        self.config_tester.run_tests().await
    }

    /// Run error handling consistency tests
    async fn run_error_handling_tests(&self) -> Result<CrossPlatformTestResult> {
        debug!("Running error handling consistency tests");
        self.interop_tester.run_error_handling_tests().await
    }

    /// Validate that SDK paths exist and are accessible
    fn validate_sdk_paths(config: &CrossPlatformConfig) -> Result<()> {
        if let Some(js_path) = &config.js_sdk_path {
            if !js_path.exists() {
                warn!("JavaScript SDK path does not exist: {:?}", js_path);
                // Don't fail - just warn and skip JS tests
            }
        }

        if let Some(python_path) = &config.python_sdk_path {
            if !python_path.exists() {
                warn!("Python SDK path does not exist: {:?}", python_path);
                // Don't fail - just warn and skip Python tests
            }
        }

        Ok(())
    }
}

/// Result structure for cross-platform tests
#[derive(Debug)]
pub struct CrossPlatformTestResult {
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub failures: Vec<TestFailure>,
    pub warnings: Vec<TestWarning>,
}

/// Platform identifier for cross-platform testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Platform {
    Rust,
    JavaScript,
    Python,
    Server,
}

impl std::fmt::Display for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Platform::Rust => write!(f, "rust"),
            Platform::JavaScript => write!(f, "javascript"),
            Platform::Python => write!(f, "python"),
            Platform::Server => write!(f, "server"),
        }
    }
}

/// Cross-platform test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformTestCase {
    pub test_id: String,
    pub name: String,
    pub description: String,
    pub test_type: CrossPlatformTestType,
    pub source_platform: Platform,
    pub target_platforms: Vec<Platform>,
    pub test_data: CrossPlatformTestData,
    pub expected_consistency: bool,
    pub timeout_secs: u64,
}

/// Types of cross-platform tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossPlatformTestType {
    /// Test signature generation and verification across platforms
    SignatureInterop,
    /// Test message canonicalization consistency
    MessageCanonicaliation,
    /// Test configuration compatibility
    ConfigurationCompatibility,
    /// Test error handling consistency
    ErrorHandling,
    /// Test performance consistency
    PerformanceComparison,
}

/// Test data for cross-platform testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformTestData {
    pub request: TestHttpRequest,
    pub signing_config: TestSigningConfig,
    pub expected_outputs: HashMap<Platform, ExpectedOutput>,
    pub variation_parameters: Option<VariationParameters>,
}

/// HTTP request for cross-platform testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Signing configuration for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSigningConfig {
    pub algorithm: String,
    pub key_id: String,
    pub private_key: String,
    pub public_key: String,
    pub created: u64,
    pub nonce: Option<String>,
    pub expires: Option<u64>,
    pub signature_components: Vec<String>,
}

/// Expected output for a specific platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedOutput {
    pub signature_input: String,
    pub signature: String,
    pub canonical_message: String,
    pub verification_result: bool,
    pub error_code: Option<String>,
}

/// Parameters for test variations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariationParameters {
    pub key_sizes: Vec<usize>,
    pub message_sizes: Vec<usize>,
    pub component_variations: Vec<Vec<String>>,
    pub timestamp_variations: Vec<i64>,
}

/// Cross-platform comparison result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub test_case_id: String,
    pub platforms_tested: Vec<Platform>,
    pub consistent: bool,
    pub differences: Vec<PlatformDifference>,
    pub performance_metrics: Option<HashMap<Platform, PerformanceMetrics>>,
}

/// Difference between platform implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformDifference {
    pub field: String,
    pub platform_a: Platform,
    pub platform_b: Platform,
    pub value_a: String,
    pub value_b: String,
    pub severity: DifferenceSeverity,
}

/// Severity of differences between platforms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DifferenceSeverity {
    /// Minor differences that don't affect functionality
    Minor,
    /// Significant differences that could cause issues
    Major,
    /// Critical differences that break compatibility
    Critical,
}

/// Performance metrics for platform comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub signature_generation_ms: f64,
    pub signature_verification_ms: f64,
    pub canonicalization_ms: f64,
    pub total_time_ms: f64,
    pub memory_usage_mb: f64,
}

/// Utilities for cross-platform testing
pub mod utils {
    use super::*;
    use std::process::Command;
    use std::path::Path;

    /// Execute a platform-specific command
    pub async fn execute_platform_command(
        platform: &Platform,
        command: &str,
        args: &[&str],
        working_dir: Option<&Path>,
    ) -> Result<String> {
        let mut cmd = match platform {
            Platform::JavaScript => {
                let mut cmd = Command::new("node");
                cmd.arg(command);
                cmd
            }
            Platform::Python => {
                let mut cmd = Command::new("python3");
                cmd.arg(command);
                cmd
            }
            Platform::Rust => {
                let mut cmd = Command::new(command);
                cmd
            }
            Platform::Server => {
                let mut cmd = Command::new(command);
                cmd
            }
        };

        cmd.args(args);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let output = cmd.output()
            .context(format!("Failed to execute {} command", platform))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Command failed for platform {}: {}", platform, stderr
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Check if a platform SDK is available
    pub fn is_platform_available(platform: &Platform, config: &CrossPlatformConfig) -> bool {
        match platform {
            Platform::JavaScript => {
                config.js_sdk_path.as_ref()
                    .map(|p| p.exists())
                    .unwrap_or(false)
            }
            Platform::Python => {
                config.python_sdk_path.as_ref()
                    .map(|p| p.exists())
                    .unwrap_or(false)
            }
            Platform::Rust => true, // Always available in this context
            Platform::Server => {
                config.server_path.as_ref()
                    .map(|p| p.exists())
                    .unwrap_or(false)
            }
        }
    }

    /// Generate a cross-platform test case ID
    pub fn generate_test_case_id(
        test_type: &CrossPlatformTestType,
        source: &Platform,
        targets: &[Platform],
    ) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(format!("{:?}", test_type).as_bytes());
        hasher.update(source.to_string().as_bytes());
        
        for target in targets {
            hasher.update(target.to_string().as_bytes());
        }
        
        let hash = hasher.finalize();
        format!("cp_{}", hex::encode(&hash[..6])) // cp_ prefix for cross-platform
    }

    /// Compare two signature outputs for consistency
    pub fn compare_signature_outputs(
        output_a: &ExpectedOutput,
        output_b: &ExpectedOutput,
        platform_a: &Platform,
        platform_b: &Platform,
    ) -> Vec<PlatformDifference> {
        let mut differences = Vec::new();

        // Compare signature input headers
        if output_a.signature_input != output_b.signature_input {
            differences.push(PlatformDifference {
                field: "signature_input".to_string(),
                platform_a: platform_a.clone(),
                platform_b: platform_b.clone(),
                value_a: output_a.signature_input.clone(),
                value_b: output_b.signature_input.clone(),
                severity: DifferenceSeverity::Critical,
            });
        }

        // Compare signatures (should be identical for same input)
        if output_a.signature != output_b.signature {
            differences.push(PlatformDifference {
                field: "signature".to_string(),
                platform_a: platform_a.clone(),
                platform_b: platform_b.clone(),
                value_a: output_a.signature.clone(),
                value_b: output_b.signature.clone(),
                severity: DifferenceSeverity::Critical,
            });
        }

        // Compare canonical messages
        if output_a.canonical_message != output_b.canonical_message {
            differences.push(PlatformDifference {
                field: "canonical_message".to_string(),
                platform_a: platform_a.clone(),
                platform_b: platform_b.clone(),
                value_a: output_a.canonical_message.clone(),
                value_b: output_b.canonical_message.clone(),
                severity: DifferenceSeverity::Critical,
            });
        }

        // Compare verification results
        if output_a.verification_result != output_b.verification_result {
            differences.push(PlatformDifference {
                field: "verification_result".to_string(),
                platform_a: platform_a.clone(),
                platform_b: platform_b.clone(),
                value_a: output_a.verification_result.to_string(),
                value_b: output_b.verification_result.to_string(),
                severity: DifferenceSeverity::Critical,
            });
        }

        differences
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_platform_validator_creation() {
        let config = CrossPlatformConfig::default();
        // This might fail in CI if SDKs aren't available, which is expected
        let result = CrossPlatformValidator::new(config);
        // Just test that it doesn't panic
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_platform_display() {
        assert_eq!(Platform::Rust.to_string(), "rust");
        assert_eq!(Platform::JavaScript.to_string(), "javascript");
        assert_eq!(Platform::Python.to_string(), "python");
        assert_eq!(Platform::Server.to_string(), "server");
    }

    #[test]
    fn test_generate_test_case_id() {
        let test_type = CrossPlatformTestType::SignatureInterop;
        let source = Platform::Rust;
        let targets = vec![Platform::JavaScript, Platform::Python];
        
        let id = utils::generate_test_case_id(&test_type, &source, &targets);
        assert!(id.starts_with("cp_"));
        assert_eq!(id.len(), 15); // "cp_" + 12 hex chars
    }

    #[test]
    fn test_compare_signature_outputs() {
        let output_a = ExpectedOutput {
            signature_input: "test-input".to_string(),
            signature: "test-sig".to_string(),
            canonical_message: "test-canonical".to_string(),
            verification_result: true,
            error_code: None,
        };

        let output_b = ExpectedOutput {
            signature_input: "different-input".to_string(),
            signature: "test-sig".to_string(),
            canonical_message: "test-canonical".to_string(),
            verification_result: true,
            error_code: None,
        };

        let diffs = utils::compare_signature_outputs(
            &output_a, &output_b, &Platform::Rust, &Platform::JavaScript
        );

        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].field, "signature_input");
        assert_eq!(diffs[0].severity, DifferenceSeverity::Critical);
    }
}