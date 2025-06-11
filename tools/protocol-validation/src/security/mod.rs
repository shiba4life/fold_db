//! Security validation and attack simulation for DataFold message signing protocol
//!
//! This module provides comprehensive security testing including:
//! - Timestamp validation testing
//! - Nonce uniqueness and replay detection testing  
//! - Attack scenario simulation tools
//! - Security parameter validation

pub mod timestamp_validation;
pub mod nonce_validation;
pub mod attack_simulation;
pub mod security_parameters;
pub mod replay_detection;

use crate::{CategoryResult, TestFailure, TestWarning, ValidationCategory, ValidationStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn, error};

/// Configuration for security validation testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable timestamp validation tests
    pub enable_timestamp_tests: bool,
    /// Enable nonce validation tests
    pub enable_nonce_tests: bool,
    /// Enable attack simulation tests
    pub enable_attack_simulation: bool,
    /// Enable security parameter validation
    pub enable_parameter_validation: bool,
    /// Enable replay detection tests
    pub enable_replay_detection: bool,
    /// Time window configurations to test
    pub test_time_windows: Vec<u64>,
    /// Clock skew values to test
    pub test_clock_skews: Vec<u64>,
    /// Number of concurrent attack attempts to simulate
    pub concurrent_attack_count: usize,
    /// Attack simulation duration in seconds
    pub attack_duration_secs: u64,
    /// Enable DoS simulation (caution: resource intensive)
    pub enable_dos_simulation: bool,
    /// Maximum memory usage for testing (MB)
    pub max_memory_usage_mb: usize,
    /// Rate limiting test configurations
    pub rate_limit_configs: Vec<RateLimitTestConfig>,
}

/// Rate limiting test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitTestConfig {
    pub requests_per_second: usize,
    pub burst_size: usize,
    pub test_duration_secs: u64,
    pub expected_success_rate: f64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_timestamp_tests: true,
            enable_nonce_tests: true,
            enable_attack_simulation: true,
            enable_parameter_validation: true,
            enable_replay_detection: true,
            test_time_windows: vec![60, 300, 600], // 1min, 5min, 10min
            test_clock_skews: vec![0, 5, 30, 120], // 0s, 5s, 30s, 2min
            concurrent_attack_count: 100,
            attack_duration_secs: 30,
            enable_dos_simulation: false, // Disabled by default for safety
            max_memory_usage_mb: 512,
            rate_limit_configs: vec![
                RateLimitTestConfig {
                    requests_per_second: 10,
                    burst_size: 5,
                    test_duration_secs: 10,
                    expected_success_rate: 0.9,
                },
                RateLimitTestConfig {
                    requests_per_second: 100,
                    burst_size: 20,
                    test_duration_secs: 5,
                    expected_success_rate: 0.5,
                },
            ],
        }
    }
}

/// Security validator for DataFold message signing protocol
pub struct SecurityValidator {
    config: SecurityConfig,
    timestamp_validator: timestamp_validation::TimestampValidator,
    nonce_validator: nonce_validation::NonceValidator,
    attack_simulator: attack_simulation::AttackSimulator,
    parameter_validator: security_parameters::SecurityParameterValidator,
    replay_detector: replay_detection::ReplayDetector,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let timestamp_validator = timestamp_validation::TimestampValidator::new(config.clone())?;
        let nonce_validator = nonce_validation::NonceValidator::new(config.clone())?;
        let attack_simulator = attack_simulation::AttackSimulator::new(config.clone())?;
        let parameter_validator = security_parameters::SecurityParameterValidator::new(config.clone())?;
        let replay_detector = replay_detection::ReplayDetector::new(config.clone())?;

        Ok(Self {
            config,
            timestamp_validator,
            nonce_validator,
            attack_simulator,
            parameter_validator,
            replay_detector,
        })
    }

    /// Run complete security validation suite
    pub async fn run_validation(&self) -> Result<CategoryResult> {
        let start_time = Instant::now();
        info!("Starting security validation");

        let mut tests_run = 0;
        let mut tests_passed = 0;
        let mut tests_failed = 0;
        let mut failures = Vec::new();
        let mut warnings = Vec::new();

        // Run timestamp validation tests
        if self.config.enable_timestamp_tests {
            info!("Running timestamp validation tests");
            let result = self.run_timestamp_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run nonce validation tests
        if self.config.enable_nonce_tests {
            info!("Running nonce validation tests");
            let result = self.run_nonce_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run security parameter validation
        if self.config.enable_parameter_validation {
            info!("Running security parameter validation");
            let result = self.run_parameter_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run replay detection tests
        if self.config.enable_replay_detection {
            info!("Running replay detection tests");
            let result = self.run_replay_detection().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run attack simulation tests
        if self.config.enable_attack_simulation {
            info!("Running attack simulation tests");
            let result = self.run_attack_simulation().await?;
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

        info!("Security validation completed: {} passed, {} failed, {} warnings",
              tests_passed, tests_failed, warnings.len());

        Ok(CategoryResult {
            category: ValidationCategory::Security,
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

    /// Run timestamp validation tests
    async fn run_timestamp_validation(&self) -> Result<SecurityTestResult> {
        debug!("Running timestamp validation tests");
        self.timestamp_validator.run_tests().await
    }

    /// Run nonce validation tests
    async fn run_nonce_validation(&self) -> Result<SecurityTestResult> {
        debug!("Running nonce validation tests");
        self.nonce_validator.run_tests().await
    }

    /// Run security parameter validation
    async fn run_parameter_validation(&self) -> Result<SecurityTestResult> {
        debug!("Running security parameter validation");
        self.parameter_validator.run_tests().await
    }

    /// Run replay detection tests
    async fn run_replay_detection(&self) -> Result<SecurityTestResult> {
        debug!("Running replay detection tests");
        self.replay_detector.run_tests().await
    }

    /// Run attack simulation tests
    async fn run_attack_simulation(&self) -> Result<SecurityTestResult> {
        debug!("Running attack simulation tests");
        self.attack_simulator.run_tests().await
    }
}

/// Internal result structure for security validation tests
#[derive(Debug)]
pub struct SecurityTestResult {
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub failures: Vec<TestFailure>,
    pub warnings: Vec<TestWarning>,
}

/// Attack types for security testing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AttackType {
    /// Replay attack using previously captured signatures
    ReplayAttack,
    /// Timestamp manipulation attempts
    TimestampManipulation,
    /// Signature forgery attempts
    SignatureForgery,
    /// Nonce collision/prediction attempts
    NonceCollision,
    /// Rate limiting bypass attempts
    RateLimitBypass,
    /// Denial of Service attacks
    DenialOfService,
    /// Message tampering attempts
    MessageTampering,
    /// Clock skew exploitation
    ClockSkewExploitation,
}

/// Security test case definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestCase {
    pub test_id: String,
    pub name: String,
    pub description: String,
    pub attack_type: AttackType,
    pub test_data: SecurityTestData,
    pub expected_outcome: SecurityTestOutcome,
    pub severity: SecurityTestSeverity,
    pub timeout_secs: u64,
}

/// Test data for security tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityTestData {
    pub requests: Vec<TestRequest>,
    pub timing_constraints: Option<TimingConstraints>,
    pub concurrency_level: Option<usize>,
    pub attack_parameters: HashMap<String, String>,
}

/// Expected outcome for security tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityTestOutcome {
    /// Attack should be blocked/detected
    ShouldBlock,
    /// Attack should be rate limited
    ShouldRateLimit,
    /// Attack should trigger alerts
    ShouldAlert,
    /// Attack should be logged but allowed (for monitoring)
    ShouldLog,
}

/// Security test severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum SecurityTestSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Timing constraints for security tests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingConstraints {
    pub max_duration_ms: u64,
    pub min_delay_between_requests_ms: u64,
    pub max_delay_between_requests_ms: u64,
}

/// Test request structure for security testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRequest {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
    pub timestamp_offset_secs: Option<i64>, // Offset from current time
    pub nonce: Option<String>,
    pub signature_manipulation: Option<SignatureManipulation>,
}

/// Signature manipulation types for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignatureManipulation {
    /// Corrupt signature bytes
    CorruptSignature,
    /// Use wrong key for signing
    WrongKey,
    /// Modify signed components after signing
    ModifyComponents,
    /// Use expired signature
    ExpiredSignature,
    /// Use future signature
    FutureSignature,
    /// Duplicate nonce
    DuplicateNonce,
}

/// Security metrics collected during testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMetrics {
    pub total_requests: usize,
    pub blocked_requests: usize,
    pub rate_limited_requests: usize,
    pub successful_attacks: usize,
    pub failed_attacks: usize,
    pub average_response_time_ms: f64,
    pub peak_memory_usage_mb: usize,
    pub alerts_triggered: usize,
    pub false_positives: usize,
    pub false_negatives: usize,
}

/// Security validation utilities
pub mod utils {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use rand::Rng;

    /// Generate a timestamp with specified offset
    pub fn generate_timestamp_with_offset(offset_secs: i64) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        (now + offset_secs).max(0) as u64
    }

    /// Generate a malformed nonce for testing
    pub fn generate_malformed_nonce() -> String {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => "invalid-nonce".to_string(), // Non-UUID format
            1 => "".to_string(), // Empty nonce
            2 => "x".repeat(1000), // Extremely long nonce
            _ => "12345678-1234-1234-1234-12345678901X".to_string(), // Invalid UUID
        }
    }

    /// Generate a batch of duplicate nonces for collision testing
    pub fn generate_duplicate_nonces(count: usize) -> Vec<String> {
        let base_nonce = uuid::Uuid::new_v4().to_string();
        vec![base_nonce; count]
    }

    /// Corrupt a signature for forgery testing
    pub fn corrupt_signature(signature: &str) -> String {
        let mut bytes = signature.as_bytes().to_vec();
        if !bytes.is_empty() {
            // Flip some bits in the signature
            let mut rng = rand::thread_rng();
            for _ in 0..3 {
                let index = rng.gen_range(0..bytes.len());
                bytes[index] ^= 0xFF;
            }
        }
        String::from_utf8_lossy(&bytes).to_string()
    }

    /// Calculate attack success rate
    pub fn calculate_success_rate(successful: usize, total: usize) -> f64 {
        if total == 0 {
            0.0
        } else {
            (successful as f64 / total as f64) * 100.0
        }
    }

    /// Validate security test configuration
    pub fn validate_security_config(config: &SecurityConfig) -> Result<()> {
        if config.concurrent_attack_count > 10000 {
            return Err(anyhow::anyhow!("Concurrent attack count too high for safety"));
        }

        if config.attack_duration_secs > 300 {
            return Err(anyhow::anyhow!("Attack duration too long for testing"));
        }

        if config.max_memory_usage_mb > 2048 {
            return Err(anyhow::anyhow!("Memory usage limit too high"));
        }

        Ok(())
    }
}

/// Built-in security test cases
pub mod builtin_tests {
    use super::*;

    /// Generate timestamp manipulation test cases
    pub fn timestamp_manipulation_tests() -> Vec<SecurityTestCase> {
        vec![
            SecurityTestCase {
                test_id: "ts_001".to_string(),
                name: "Expired Timestamp Attack".to_string(),
                description: "Test rejection of expired timestamps".to_string(),
                attack_type: AttackType::TimestampManipulation,
                test_data: SecurityTestData {
                    requests: vec![TestRequest {
                        method: "POST".to_string(),
                        uri: "/api/test".to_string(),
                        headers: HashMap::new(),
                        body: Some("{}".to_string()),
                        timestamp_offset_secs: Some(-3600), // 1 hour ago
                        nonce: None,
                        signature_manipulation: None,
                    }],
                    timing_constraints: None,
                    concurrency_level: Some(1),
                    attack_parameters: HashMap::new(),
                },
                expected_outcome: SecurityTestOutcome::ShouldBlock,
                severity: SecurityTestSeverity::High,
                timeout_secs: 30,
            },
            SecurityTestCase {
                test_id: "ts_002".to_string(),
                name: "Future Timestamp Attack".to_string(),
                description: "Test rejection of future timestamps".to_string(),
                attack_type: AttackType::TimestampManipulation,
                test_data: SecurityTestData {
                    requests: vec![TestRequest {
                        method: "POST".to_string(),
                        uri: "/api/test".to_string(),
                        headers: HashMap::new(),
                        body: Some("{}".to_string()),
                        timestamp_offset_secs: Some(3600), // 1 hour future
                        nonce: None,
                        signature_manipulation: None,
                    }],
                    timing_constraints: None,
                    concurrency_level: Some(1),
                    attack_parameters: HashMap::new(),
                },
                expected_outcome: SecurityTestOutcome::ShouldBlock,
                severity: SecurityTestSeverity::High,
                timeout_secs: 30,
            },
        ]
    }

    /// Generate replay attack test cases
    pub fn replay_attack_tests() -> Vec<SecurityTestCase> {
        vec![
            SecurityTestCase {
                test_id: "replay_001".to_string(),
                name: "Immediate Replay Attack".to_string(),
                description: "Test detection of immediate request replay".to_string(),
                attack_type: AttackType::ReplayAttack,
                test_data: SecurityTestData {
                    requests: vec![
                        // Original request
                        TestRequest {
                            method: "POST".to_string(),
                            uri: "/api/test".to_string(),
                            headers: HashMap::new(),
                            body: Some("{}".to_string()),
                            timestamp_offset_secs: None,
                            nonce: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
                            signature_manipulation: None,
                        },
                        // Replayed request (same nonce)
                        TestRequest {
                            method: "POST".to_string(),
                            uri: "/api/test".to_string(),
                            headers: HashMap::new(),
                            body: Some("{}".to_string()),
                            timestamp_offset_secs: None,
                            nonce: Some("550e8400-e29b-41d4-a716-446655440000".to_string()),
                            signature_manipulation: None,
                        },
                    ],
                    timing_constraints: Some(TimingConstraints {
                        max_duration_ms: 5000,
                        min_delay_between_requests_ms: 100,
                        max_delay_between_requests_ms: 1000,
                    }),
                    concurrency_level: Some(1),
                    attack_parameters: HashMap::new(),
                },
                expected_outcome: SecurityTestOutcome::ShouldBlock,
                severity: SecurityTestSeverity::Critical,
                timeout_secs: 30,
            },
        ]
    }

    /// Generate signature forgery test cases
    pub fn signature_forgery_tests() -> Vec<SecurityTestCase> {
        vec![
            SecurityTestCase {
                test_id: "forge_001".to_string(),
                name: "Corrupted Signature Attack".to_string(),
                description: "Test rejection of corrupted signatures".to_string(),
                attack_type: AttackType::SignatureForgery,
                test_data: SecurityTestData {
                    requests: vec![TestRequest {
                        method: "POST".to_string(),
                        uri: "/api/test".to_string(),
                        headers: HashMap::new(),
                        body: Some("{}".to_string()),
                        timestamp_offset_secs: None,
                        nonce: None,
                        signature_manipulation: Some(SignatureManipulation::CorruptSignature),
                    }],
                    timing_constraints: None,
                    concurrency_level: Some(1),
                    attack_parameters: HashMap::new(),
                },
                expected_outcome: SecurityTestOutcome::ShouldBlock,
                severity: SecurityTestSeverity::Critical,
                timeout_secs: 30,
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_validator_creation() {
        let config = SecurityConfig::default();
        let validator = SecurityValidator::new(config).unwrap();
        assert!(validator.config.enable_timestamp_tests);
    }

    #[test]
    fn test_generate_timestamp_with_offset() {
        let timestamp = utils::generate_timestamp_with_offset(-3600);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        assert!(timestamp < now);
        
        let future_timestamp = utils::generate_timestamp_with_offset(3600);
        assert!(future_timestamp > now);
    }

    #[test]
    fn test_generate_malformed_nonce() {
        let nonce = utils::generate_malformed_nonce();
        assert!(!nonce.is_empty());
        // Should not be a valid UUID
        assert!(uuid::Uuid::parse_str(&nonce).is_err() || nonce.len() != 36);
    }

    #[test]
    fn test_security_config_validation() {
        let mut config = SecurityConfig::default();
        assert!(utils::validate_security_config(&config).is_ok());
        
        config.concurrent_attack_count = 20000;
        assert!(utils::validate_security_config(&config).is_err());
    }
}