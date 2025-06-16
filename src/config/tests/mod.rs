//! Comprehensive testing framework for cross-platform configuration management
//!
//! This module contains the complete test suite for PBI 27, including:
//! - Cross-platform testing with mock implementations
//! - Performance testing and benchmarking
//! - Security validation testing
//! - Integration testing with existing systems
//! - Error handling and edge case testing
//! - Performance verification against requirements

pub mod benchmarks;
pub mod cross_platform;
pub mod error_handling;
pub mod integration;
pub mod mocks;
pub mod performance;
pub mod security;
pub mod utils;

use std::sync::Once;
use tempfile::TempDir;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        // Initialize logging for tests
        env_logger::builder()
            .filter_level(log::LevelFilter::Debug)
            .init();
    });
}

/// Create temporary test directory
pub fn create_test_dir(name: &str) -> TempDir {
    tempfile::Builder::new()
        .prefix(&format!("datafold_test_{}_", name))
        .tempdir()
        .expect("Failed to create test directory")
}

/// Test configuration constants
pub mod constants {
    use std::time::Duration;

    /// Maximum allowed configuration load time (requirement: < 10ms)
    pub const MAX_LOAD_TIME: Duration = Duration::from_millis(10);

    /// Maximum allowed memory usage (requirement: < 1MB)
    pub const MAX_MEMORY_USAGE_MB: usize = 1;

    /// Maximum allowed hot reload time (requirement: < 1s)
    pub const MAX_HOT_RELOAD_TIME: Duration = Duration::from_secs(1);

    /// Test timeout for async operations
    pub const TEST_TIMEOUT: Duration = Duration::from_secs(30);

    /// Number of performance test iterations
    pub const PERF_TEST_ITERATIONS: usize = 1000;

    /// Size of large configuration for memory tests
    pub const LARGE_CONFIG_SECTIONS: usize = 100;
}
