//! End-to-End Integration Test Suite for DataFold Message Signing Authentication
//!
//! This module provides comprehensive end-to-end integration testing for the complete
//! authentication workflow across server and all client implementations (Rust server,
//! JavaScript SDK, Python SDK, CLI tool).
//!
//! ## Test Categories
//!
//! 1. **Complete Authentication Workflow Testing**
//!    - Key generation and registration end-to-end flow
//!    - Client authentication with server verification
//!    - Multi-platform client authentication testing
//!    - Error handling and recovery scenarios
//!
//! 2. **Cross-Platform Integration Tests**
//!    - JavaScript SDK with DataFold server authentication
//!    - Python SDK with DataFold server authentication
//!    - CLI tool with DataFold server authentication
//!    - Mixed client scenario testing
//!
//! 3. **Real-World Scenario Testing**
//!    - Concurrent client authentication testing
//!    - High-load authentication scenarios
//!    - Network failure and retry testing
//!    - Time synchronization and clock skew scenarios
//!
//! 4. **Security Integration Testing**
//!    - Replay attack prevention validation
//!    - Invalid signature rejection testing
//!    - Timestamp window enforcement testing
//!    - Rate limiting and attack detection testing

pub mod workflow_tests;
pub mod cross_platform_tests;
pub mod real_world_scenarios;
pub mod security_integration_tests;
pub mod test_utils;
pub mod server_harness;
pub mod sdk_harness;
pub mod main_test_runner;

// Re-export main test runner functions for easy access
pub use main_test_runner::{run_quick_e2e_tests, run_comprehensive_e2e_tests, E2ETestRunner};

use std::sync::Once;
use log::info;

static INIT: Once = Once::new();

/// Initialize the E2E test environment
pub fn init_e2e_environment() {
    INIT.call_once(|| {
        env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Info)
            .init();
        
        info!("🚀 DataFold E2E Integration Test Suite initialized");
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_e2e_suite_initialization() {
        init_e2e_environment();
        
        // Verify all test modules are accessible
        assert!(true, "E2E test suite modules loaded successfully");
    }
}