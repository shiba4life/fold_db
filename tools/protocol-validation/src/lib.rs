//! DataFold Protocol Compliance Validation Library
//!
//! This library provides comprehensive validation tools for DataFold's RFC 9421 HTTP Message
//! Signatures implementation, ensuring protocol compliance, security, and interoperability
//! across all DataFold implementations (Rust, JavaScript, Python).

pub mod rfc9421;
pub mod cross_platform;
pub mod security;
pub mod performance;
pub mod test_vectors;
pub mod config;
pub mod report;
pub mod utils;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn, error};
use crate::reporting::types::UnifiedSummarySection;

/// Main validation result encompassing all test categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub test_suite_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub duration_ms: u64,
    pub overall_status: ValidationStatus,
    pub categories: HashMap<String, CategoryResult>,
    pub summary: ValidationSummary,
    pub environment_info: EnvironmentInfo,
}

/// Status of validation tests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// All tests passed
    Passed,
    /// Some tests failed but not critical
    Warning,
    /// Critical tests failed
    Failed,
    /// Tests could not be executed
    Error,
}

/// Result for a specific validation category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryResult {
    pub category: ValidationCategory,
    pub status: ValidationStatus,
    pub tests_run: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub tests_skipped: usize,
    pub duration_ms: u64,
    pub failures: Vec<TestFailure>,
    pub warnings: Vec<TestWarning>,
}

/// Validation categories supported by the validation suite
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ValidationCategory {
    /// RFC 9421 HTTP Message Signatures compliance
    RFC9421Compliance,
    /// Cross-platform interoperability testing
    CrossPlatform,
    /// Security validation and attack simulation
    Security,
    /// Performance and scalability testing
    Performance,
    /// Test vector validation
    TestVectors,
}

/// Individual test failure details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    pub test_name: String,
    pub error_type: String,
    pub error_message: String,
    pub expected: Option<String>,
    pub actual: Option<String>,
    pub context: HashMap<String, String>,
}

/// Individual test warning details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestWarning {
    pub test_name: String,
    pub warning_type: String,
    pub message: String,
    pub recommendation: Option<String>,
}

/// Summary statistics for the validation run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    pub total_tests: usize,
    pub total_passed: usize,
    pub total_failed: usize,
    pub total_skipped: usize,
    pub success_rate: f64,
    pub critical_failures: usize,
    pub warnings: usize,
}

/// Environment information for reproducibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentInfo {
    pub rust_version: String,
    pub node_version: Option<String>,
    pub python_version: Option<String>,
    pub platform: String,
    pub architecture: String,
    pub datafold_server_version: Option<String>,
    pub test_environment: String, // development, staging, production
}

/// Configuration for the validation suite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    pub enabled_categories: Vec<ValidationCategory>,
    pub rfc9421_config: rfc9421::RFC9421Config,
    pub security_config: security::SecurityConfig,
    pub performance_config: performance::PerformanceConfig,
    pub cross_platform_config: cross_platform::CrossPlatformConfig,
    pub output_config: OutputConfig,
}

/// Output and reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub generate_html_report: bool,
    pub generate_json_report: bool,
    pub generate_junit_xml: bool,
    pub output_directory: String,
    pub include_debug_info: bool,
    pub include_test_vectors: bool,
}

/// Main validation orchestrator
pub struct ValidationSuite {
    config: ValidationConfig,
    environment: EnvironmentInfo,
}

impl ValidationSuite {
    /// Create a new validation suite with the given configuration
    pub fn new(config: ValidationConfig) -> Result<Self> {
        let environment = Self::collect_environment_info()
            .context("Failed to collect environment information")?;
        
        info!("Initialized validation suite with {} categories enabled", 
              config.enabled_categories.len());
        
        Ok(Self { config, environment })
    }

    /// Run the complete validation suite
    pub async fn run_all_validations(&self) -> Result<ValidationResult> {
        let start_time = Instant::now();
        let test_suite_id = uuid::Uuid::new_v4().to_string();
        
        info!("Starting validation suite run: {}", test_suite_id);
        
        let mut categories = HashMap::new();
        let mut overall_status = ValidationStatus::Passed;

        // Run each enabled validation category
        for category in &self.config.enabled_categories {
            info!("Running validation category: {:?}", category);
            
            let category_result = match category {
                ValidationCategory::RFC9421Compliance => {
                    self.run_rfc9421_validation().await
                        .context("RFC 9421 validation failed")?
                }
                ValidationCategory::CrossPlatform => {
                    self.run_cross_platform_validation().await
                        .context("Cross-platform validation failed")?
                }
                ValidationCategory::Security => {
                    self.run_security_validation().await
                        .context("Security validation failed")?
                }
                ValidationCategory::Performance => {
                    self.run_performance_validation().await
                        .context("Performance validation failed")?
                }
                ValidationCategory::TestVectors => {
                    self.run_test_vector_validation().await
                        .context("Test vector validation failed")?
                }
            };

            // Update overall status based on category results
            match category_result.status {
                ValidationStatus::Failed => overall_status = ValidationStatus::Failed,
                ValidationStatus::Warning if overall_status == ValidationStatus::Passed => {
                    overall_status = ValidationStatus::Warning;
                }
                _ => {}
            }

            categories.insert(format!("{:?}", category), category_result);
        }

        let duration = start_time.elapsed();
        let summary = Self::calculate_summary(&categories);

        let result = ValidationResult {
            test_suite_id,
            timestamp: chrono::Utc::now(),
            duration_ms: duration.as_millis() as u64,
            overall_status,
            categories,
            summary,
            environment_info: self.environment.clone(),
        };

        info!("Validation suite completed in {}ms with status: {:?}", 
              result.duration_ms, result.overall_status);

        Ok(result)
    }

    /// Run RFC 9421 compliance validation
    async fn run_rfc9421_validation(&self) -> Result<CategoryResult> {
        let validator = rfc9421::RFC9421Validator::new(self.config.rfc9421_config.clone())?;
        validator.run_validation().await
    }

    /// Run cross-platform interoperability validation
    async fn run_cross_platform_validation(&self) -> Result<CategoryResult> {
        let validator = cross_platform::CrossPlatformValidator::new(
            self.config.cross_platform_config.clone()
        )?;
        validator.run_validation().await
    }

    /// Run security validation and attack simulation
    async fn run_security_validation(&self) -> Result<CategoryResult> {
        let validator = security::SecurityValidator::new(self.config.security_config.clone())?;
        validator.run_validation().await
    }

    /// Run performance validation and benchmarking
    async fn run_performance_validation(&self) -> Result<CategoryResult> {
        let validator = performance::PerformanceValidator::new(
            self.config.performance_config.clone()
        )?;
        validator.run_validation().await
    }

    /// Run test vector validation
    async fn run_test_vector_validation(&self) -> Result<CategoryResult> {
        let validator = test_vectors::TestVectorValidator::new()?;
        validator.run_validation().await
    }

    /// Collect environment information for reproducibility
    fn collect_environment_info() -> Result<EnvironmentInfo> {
        let rust_version = std::env::var("RUSTC_VERSION")
            .unwrap_or_else(|_| "unknown".to_string());
        
        let node_version = std::process::Command::new("node")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string());

        let python_version = std::process::Command::new("python3")
            .arg("--version")
            .output()
            .ok()
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .map(|s| s.trim().to_string());

        let platform = std::env::consts::OS.to_string();
        let architecture = std::env::consts::ARCH.to_string();

        let test_environment = std::env::var("TEST_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        Ok(EnvironmentInfo {
            rust_version,
            node_version,
            python_version,
            platform,
            architecture,
            datafold_server_version: None, // TODO: Query server for version
            test_environment,
        })
    }

    /// Calculate summary statistics from category results
    fn calculate_summary(categories: &HashMap<String, CategoryResult>) -> ValidationSummary {
        let mut total_tests = 0;
        let mut total_passed = 0;
        let mut total_failed = 0;
        let mut total_skipped = 0;
        let mut critical_failures = 0;
        let mut warnings = 0;

        for result in categories.values() {
            total_tests += result.tests_run;
            total_passed += result.tests_passed;
            total_failed += result.tests_failed;
            total_skipped += result.tests_skipped;
            warnings += result.warnings.len();

            // Count critical failures (security and RFC compliance failures)
            match result.category {
                ValidationCategory::RFC9421Compliance | ValidationCategory::Security => {
                    critical_failures += result.tests_failed;
                }
                _ => {}
            }
        }

        let success_rate = if total_tests > 0 {
            (total_passed as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };

        ValidationSummary {
            total_tests,
            total_passed,
            total_failed,
            total_skipped,
            success_rate,
            critical_failures,
            warnings,
        }
    }

    /// Generate reports based on validation results
    pub async fn generate_reports(&self, result: &ValidationResult) -> Result<()> {
        let reporter = report::ReportGenerator::new(self.config.output_config.clone());
        reporter.generate_all_reports(result).await
            .context("Failed to generate validation reports")
    }
}

/// Default configuration for the validation suite
impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            enabled_categories: vec![
                ValidationCategory::RFC9421Compliance,
                ValidationCategory::CrossPlatform,
                ValidationCategory::Security,
                ValidationCategory::Performance,
                ValidationCategory::TestVectors,
            ],
            rfc9421_config: rfc9421::RFC9421Config::default(),
            security_config: security::SecurityConfig::default(),
            performance_config: performance::PerformanceConfig::default(),
            cross_platform_config: cross_platform::CrossPlatformConfig::default(),
            output_config: OutputConfig {
                generate_html_report: true,
                generate_json_report: true,
                generate_junit_xml: true,
                output_directory: "reports".to_string(),
                include_debug_info: false,
                include_test_vectors: true,
            },
        }
    }
}

// Implement UnifiedSummarySection for ValidationSummary
impl UnifiedSummarySection for ValidationSummary {
    fn section_name(&self) -> &'static str { "validation_summary" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_suite_creation() {
        let config = ValidationConfig::default();
        let suite = ValidationSuite::new(config).unwrap();
        assert!(!suite.environment.rust_version.is_empty());
        assert!(!suite.environment.platform.is_empty());
    }

    #[test]
    fn test_validation_status_ordering() {
        assert!(ValidationStatus::Passed != ValidationStatus::Failed);
        assert!(ValidationStatus::Warning != ValidationStatus::Error);
    }

    #[test]
    fn test_validation_summary_calculation() {
        let mut categories = HashMap::new();
        
        categories.insert("test1".to_string(), CategoryResult {
            category: ValidationCategory::RFC9421Compliance,
            status: ValidationStatus::Passed,
            tests_run: 10,
            tests_passed: 8,
            tests_failed: 2,
            tests_skipped: 0,
            duration_ms: 1000,
            failures: vec![],
            warnings: vec![],
        });

        let summary = ValidationSuite::calculate_summary(&categories);
        assert_eq!(summary.total_tests, 10);
        assert_eq!(summary.total_passed, 8);
        assert_eq!(summary.total_failed, 2);
        assert_eq!(summary.success_rate, 80.0);
    }
}