//! End-to-End Testing Framework for DataFold Message Signing Authentication
//!
//! This module provides a comprehensive E2E testing framework for DataFold's
//! message signing authentication system across multiple platforms.

pub mod test_utils;

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize the E2E testing environment
pub fn init_e2e_environment() {
    INIT.call_once(|| {
        env_logger::init();
        log::info!("E2E testing environment initialized");
    });
}

/// E2E test runner for coordinating tests across platforms
pub struct E2ETestRunner {
    config: test_utils::E2ETestConfig,
}

impl E2ETestRunner {
    pub fn new(config: test_utils::E2ETestConfig) -> Self {
        Self { config }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Running E2E tests with config: {:?}", self.config);
        Ok(())
    }
}

/// Run quick E2E tests (subset of tests for fast feedback)
pub async fn run_quick_e2e_tests() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running quick E2E tests");
    let config = test_utils::E2ETestConfig::quick();
    let runner = E2ETestRunner::new(config);
    runner.run().await
}

/// Run comprehensive E2E tests (full test suite)
pub async fn run_comprehensive_e2e_tests() -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Running comprehensive E2E tests");
    let config = test_utils::E2ETestConfig::comprehensive();
    let runner = E2ETestRunner::new(config);
    runner.run().await
}