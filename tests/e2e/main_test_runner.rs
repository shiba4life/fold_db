//! Main End-to-End Integration Test Runner
//!
//! This module provides the main test runner that orchestrates all E2E test suites
//! for DataFold's message signing authentication system.

use super::test_utils::{E2ETestConfig, E2ETestResults};
use super::workflow_tests::WorkflowTests;
use super::cross_platform_tests::CrossPlatformTests;
use super::real_world_scenarios::RealWorldScenarios;
use super::security_integration_tests::SecurityIntegrationTests;
use serde_json::json;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::timeout;

/// Main E2E test runner that orchestrates all test suites
pub struct E2ETestRunner {
    config: E2ETestConfig,
    results: E2ETestResults,
    start_time: Option<SystemTime>,
}

impl E2ETestRunner {
    /// Create a new E2E test runner
    pub fn new(config: E2ETestConfig) -> Self {
        Self {
            config,
            results: E2ETestResults::default(),
            start_time: None,
        }
    }

    /// Run all E2E test suites
    pub async fn run_all_tests(&mut self) -> anyhow::Result<E2ETestResults> {
        self.start_time = Some(SystemTime::now());
        
        log::info!("🚀 Starting DataFold E2E Integration Test Suite");
        log::info!("Configuration:");
        log::info!("  Server URL: {}", self.config.server_url);
        log::info!("  Timeout: {}s", self.config.test_timeout_secs);
        log::info!("  Concurrent clients: {}", self.config.concurrent_clients);
        log::info!("  Attack simulation: {}", self.config.enable_attack_simulation);

        // Run all test suites with error handling
        let suite_results = vec![
            ("Workflow Tests", self.run_workflow_tests().await),
            ("Cross-Platform Tests", self.run_cross_platform_tests().await),
            ("Real-World Scenarios", self.run_real_world_scenarios().await),
            ("Security Integration Tests", self.run_security_integration_tests().await),
        ];

        // Aggregate results
        let mut total_suites = 0;
        let mut passed_suites = 0;
        let mut total_tests = 0;
        let mut passed_tests = 0;

        for (suite_name, result) in suite_results {
            total_suites += 1;
            
            match result {
                Ok(suite_results) => {
                    passed_suites += 1;
                    total_tests += suite_results.total_tests;
                    passed_tests += suite_results.passed_tests;
                    
                    // Merge performance metrics
                    for (key, value) in suite_results.performance_metrics {
                        self.results.performance_metrics.insert(
                            format!("{}_{}", suite_name.to_lowercase().replace(' ', "_"), key),
                            value
                        );
                    }
                    
                    // Merge errors and warnings
                    self.results.errors.extend(suite_results.errors);
                    self.results.warnings.extend(suite_results.warnings);
                    
                    log::info!("✅ {} completed: {}/{} tests passed", 
                              suite_name, suite_results.passed_tests, suite_results.total_tests);
                }
                Err(e) => {
                    log::error!("❌ {} failed: {}", suite_name, e);
                    self.results.errors.push(format!("{}: {}", suite_name, e));
                }
            }
        }

        // Calculate final results
        self.results.total_tests = total_tests;
        self.results.passed_tests = passed_tests;
        let success_rate = if total_tests > 0 { passed_tests as f64 / total_tests as f64 } else { 0.0 };

        // Calculate total duration
        let total_duration = self.start_time
            .map(|start| SystemTime::now().duration_since(start).unwrap())
            .unwrap_or(Duration::from_secs(0));

        // Final summary
        log::info!("📊 E2E Test Suite Summary:");
        log::info!("  Test Suites: {}/{} passed", passed_suites, total_suites);
        log::info!("  Total Tests: {}/{} passed ({:.1}%)", passed_tests, total_tests, success_rate * 100.0);
        log::info!("  Duration: {:?}", total_duration);
        
        if self.results.errors.is_empty() {
            log::info!("🎉 All E2E tests completed successfully!");
        } else {
            log::warn!("⚠️  Some tests had issues:");
            for error in &self.results.errors {
                log::warn!("    {}", error);
            }
        }

        // Add final metrics
        self.results.add_metric("total_duration_seconds", total_duration.as_secs() as f64);
        self.results.add_metric("overall_success_rate", success_rate);
        self.results.add_metric("test_suites_passed", passed_suites as f64);
        self.results.add_metric("test_suites_total", total_suites as f64);

        Ok(self.results.clone())
    }

    /// Run workflow tests with timeout
    async fn run_workflow_tests(&self) -> anyhow::Result<E2ETestResults> {
        timeout(
            Duration::from_secs(self.config.test_timeout_secs * 6), // Allow more time for full suite
            async {
                let mut workflow_tests = WorkflowTests::new(self.config.clone());
                workflow_tests.run_all_tests().await
            }
        ).await
        .map_err(|_| anyhow::anyhow!("Workflow tests timed out"))?
    }

    /// Run cross-platform tests with timeout
    async fn run_cross_platform_tests(&self) -> anyhow::Result<E2ETestResults> {
        timeout(
            Duration::from_secs(self.config.test_timeout_secs * 8), // Cross-platform may take longer
            async {
                let mut cross_platform_tests = CrossPlatformTests::new(self.config.clone());
                cross_platform_tests.run_all_tests().await
            }
        ).await
        .map_err(|_| anyhow::anyhow!("Cross-platform tests timed out"))?
    }

    /// Run real-world scenario tests with timeout
    async fn run_real_world_scenarios(&self) -> anyhow::Result<E2ETestResults> {
        timeout(
            Duration::from_secs(self.config.test_timeout_secs * 10), // Real-world scenarios may be extensive
            async {
                let mut real_world_tests = RealWorldScenarios::new(self.config.clone());
                real_world_tests.run_all_tests().await
            }
        ).await
        .map_err(|_| anyhow::anyhow!("Real-world scenario tests timed out"))?
    }

    /// Run security integration tests with timeout
    async fn run_security_integration_tests(&self) -> anyhow::Result<E2ETestResults> {
        if !self.config.enable_attack_simulation {
            log::info!("🔒 Skipping attack simulation tests (disabled in configuration)");
        }

        timeout(
            Duration::from_secs(self.config.test_timeout_secs * 6),
            async {
                let mut security_tests = SecurityIntegrationTests::new(self.config.clone());
                security_tests.run_all_tests().await
            }
        ).await
        .map_err(|_| anyhow::anyhow!("Security integration tests timed out"))?
    }

    /// Generate comprehensive test report
    pub fn generate_test_report(&self) -> anyhow::Result<String> {
        let total_duration = self.start_time
            .map(|start| SystemTime::now().duration_since(start).unwrap())
            .unwrap_or(Duration::from_secs(0));

        let success_rate = if self.results.total_tests > 0 { 
            self.results.passed_tests as f64 / self.results.total_tests as f64 
        } else { 
            0.0 
        };

        let report = json!({
            "test_run_summary": {
                "timestamp": SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                "total_duration_seconds": total_duration.as_secs(),
                "total_tests": self.results.total_tests,
                "passed_tests": self.results.passed_tests,
                "failed_tests": self.results.total_tests - self.results.passed_tests,
                "success_rate": success_rate,
                "overall_status": if self.results.all_passed() { "PASSED" } else { "FAILED" }
            },
            "test_configuration": {
                "server_url": self.config.server_url,
                "timeout_seconds": self.config.test_timeout_secs,
                "concurrent_clients": self.config.concurrent_clients,
                "attack_simulation_enabled": self.config.enable_attack_simulation
            },
            "performance_metrics": self.results.performance_metrics,
            "errors": self.results.errors,
            "warnings": self.results.warnings,
            "test_categories": {
                "workflow_tests": "Complete authentication workflow validation",
                "cross_platform_tests": "JavaScript/Python/CLI SDK interoperability",
                "real_world_scenarios": "Load testing, network failures, time sync",
                "security_integration": "Replay prevention, attack detection, rate limiting"
            }
        });

        Ok(serde_json::to_string_pretty(&report)?)
    }

    /// Generate JUnit XML report for CI/CD integration
    pub fn generate_junit_xml(&self) -> anyhow::Result<String> {
        let total_duration = self.start_time
            .map(|start| SystemTime::now().duration_since(start).unwrap())
            .unwrap_or(Duration::from_secs(0));

        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');
        xml.push_str(&format!(
            r#"<testsuites name="DataFold E2E Integration Tests" tests="{}" failures="{}" time="{:.3}">"#,
            self.results.total_tests,
            self.results.total_tests - self.results.passed_tests,
            total_duration.as_secs_f64()
        ));
        xml.push('\n');

        // Add test suite
        xml.push_str(&format!(
            r#"  <testsuite name="E2E Integration" tests="{}" failures="{}" time="{:.3}">"#,
            self.results.total_tests,
            self.results.total_tests - self.results.passed_tests,
            total_duration.as_secs_f64()
        ));
        xml.push('\n');

        // Add individual test cases (simplified)
        for (i, error) in self.results.errors.iter().enumerate() {
            xml.push_str(&format!(
                r#"    <testcase name="test_{}" classname="E2EIntegration" time="0.0">"#,
                i + 1
            ));
            xml.push('\n');
            xml.push_str(&format!(r#"      <failure message="{}"/>"#, 
                html_escape::encode_text(error)));
            xml.push('\n');
            xml.push_str("    </testcase>");
            xml.push('\n');
        }

        // Add successful tests
        for i in 0..self.results.passed_tests {
            xml.push_str(&format!(
                r#"    <testcase name="passed_test_{}" classname="E2EIntegration" time="0.0"/>"#,
                i + 1
            ));
            xml.push('\n');
        }

        xml.push_str("  </testsuite>");
        xml.push('\n');
        xml.push_str("</testsuites>");
        xml.push('\n');

        Ok(xml)
    }
}

/// Quick test runner for CI/CD environments
pub async fn run_quick_e2e_tests() -> anyhow::Result<bool> {
    let config = E2ETestConfig {
        server_url: "http://localhost:8080".to_string(),
        test_timeout_secs: 30,
        concurrent_clients: 5,
        enable_attack_simulation: false, // Disabled for quick tests
        temp_dir: std::env::temp_dir(),
    };

    let mut runner = E2ETestRunner::new(config);
    
    // Run just essential tests
    let workflow_results = runner.run_workflow_tests().await?;
    let security_results = runner.run_security_integration_tests().await?;

    let total_tests = workflow_results.total_tests + security_results.total_tests;
    let passed_tests = workflow_results.passed_tests + security_results.passed_tests;
    let success_rate = passed_tests as f64 / total_tests as f64;

    log::info!("Quick E2E Test Results: {}/{} passed ({:.1}%)", 
               passed_tests, total_tests, success_rate * 100.0);

    Ok(success_rate >= 0.95) // 95% success rate threshold
}

/// Comprehensive test runner for full validation
pub async fn run_comprehensive_e2e_tests() -> anyhow::Result<E2ETestResults> {
    let config = E2ETestConfig {
        server_url: "http://localhost:8080".to_string(),
        test_timeout_secs: 60,
        concurrent_clients: 10,
        enable_attack_simulation: true,
        temp_dir: std::env::temp_dir(),
    };

    let mut runner = E2ETestRunner::new(config);
    runner.run_all_tests().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::e2e::init_e2e_environment;

    #[tokio::test]
    async fn test_e2e_test_runner() {
        init_e2e_environment();
        
        let config = E2ETestConfig {
            server_url: "http://localhost:8080".to_string(),
            test_timeout_secs: 10, // Short timeout for unit test
            concurrent_clients: 2,
            enable_attack_simulation: false,
            temp_dir: std::env::temp_dir(),
        };

        let mut runner = E2ETestRunner::new(config);
        
        // Test that the runner can be created and configured
        assert_eq!(runner.results.total_tests, 0);
        assert!(runner.start_time.is_none());
    }

    #[test]
    fn test_junit_xml_generation() {
        let mut results = E2ETestResults::default();
        results.add_result("test1", true, None);
        results.add_result("test2", false, Some("Test failed".to_string()));

        let config = E2ETestConfig::default();
        let mut runner = E2ETestRunner::new(config);
        runner.results = results;

        let junit_xml = runner.generate_junit_xml().unwrap();
        assert!(junit_xml.contains("<?xml version"));
        assert!(junit_xml.contains("testsuites"));
        assert!(junit_xml.contains("testsuite"));
        assert!(junit_xml.contains("testcase"));
    }

    #[test]
    fn test_report_generation() {
        let mut results = E2ETestResults::default();
        results.add_result("test1", true, None);
        results.add_metric("response_time", 100.0);

        let config = E2ETestConfig::default();
        let mut runner = E2ETestRunner::new(config);
        runner.results = results;

        let report = runner.generate_test_report().unwrap();
        assert!(report.contains("test_run_summary"));
        assert!(report.contains("performance_metrics"));
        assert!(report.contains("response_time"));
    }
}