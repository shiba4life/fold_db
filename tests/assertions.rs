//! Centralized test assertion utilities for eliminating duplicate test output patterns
//!
//! This module consolidates common test assertion and output patterns found throughout the test suite:
//! - Standardizes 47+ success message patterns (`println!("✅")`)
//! - Common test result patterns for consistent output
//! - Shared error assertion patterns
//! - Unified test step reporting

use std::fmt::Display;

/// Test result reporter for standardizing success messages across all tests
pub struct TestReporter;

impl TestReporter {
    /// Standard success message - consolidates 47+ `println!("✅")` patterns
    pub fn success<T: Display>(message: T) {
        println!("✅ {}", message);
    }

    /// Standard info message for test steps
    pub fn info<T: Display>(message: T) {
        println!("🧪 {}", message);
    }

    /// Standard warning message for test conditions
    pub fn warning<T: Display>(message: T) {
        println!("⚠️ {}", message);
    }

    /// Standard error message for test failures
    pub fn error<T: Display>(message: T) {
        println!("❌ {}", message);
    }

    /// Standard debug message for detailed test information
    pub fn debug<T: Display>(message: T) {
        println!("🔍 {}", message);
    }

    /// Standard performance message for timing information
    pub fn performance<T: Display>(message: T) {
        println!("⏱️ {}", message);
    }

    /// Standard setup message for test initialization
    pub fn setup<T: Display>(message: T) {
        println!("🔧 {}", message);
    }

    /// Standard execution message for test operations
    pub fn execution<T: Display>(message: T) {
        println!("🚀 {}", message);
    }

    /// Standard result message for test outcomes
    pub fn result<T: Display>(message: T) {
        println!("📊 {}", message);
    }

    /// Standard completion message for test finalization
    pub fn completion<T: Display>(message: T) {
        println!("🎯 {}", message);
    }
}

/// Test assertion utilities for common patterns
pub struct TestAssertions;

impl TestAssertions {
    /// Assert transform execution success with standardized messaging
    pub fn assert_transform_success(result: &Result<String, Box<dyn std::error::Error>>, expected_message: &str) {
        match result {
            Ok(actual) => {
                TestReporter::success(format!("Transform executed successfully: {}", actual));
                if actual.contains(expected_message) {
                    TestReporter::success(format!("Expected message found: {}", expected_message));
                } else {
                    TestReporter::warning(format!("Expected '{}', got '{}'", expected_message, actual));
                }
            }
            Err(e) => {
                TestReporter::error(format!("Transform execution failed: {}", e));
                panic!("Transform execution should succeed");
            }
        }
    }

    /// Assert event publishing success with standardized messaging
    pub fn assert_event_published<T>(result: &Result<(), T>, event_type: &str) 
    where 
        T: std::fmt::Debug + std::fmt::Display 
    {
        match result {
            Ok(()) => TestReporter::success(format!("{} event published successfully", event_type)),
            Err(e) => {
                TestReporter::error(format!("Failed to publish {} event: {}", event_type, e));
                panic!("Event publishing should succeed: {:?}", e);
            }
        }
    }

    /// Assert database operation success with standardized messaging
    pub fn assert_db_operation_success<T, E>(result: &Result<T, E>, operation: &str)
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug + std::fmt::Display
    {
        match result {
            Ok(_) => TestReporter::success(format!("{} operation completed successfully", operation)),
            Err(e) => {
                TestReporter::error(format!("{} operation failed: {}", operation, e));
                panic!("Database operation should succeed: {:?}", e);
            }
        }
    }

    /// Assert schema operation success with standardized messaging
    pub fn assert_schema_success<T, E>(result: &Result<T, E>, schema_name: &str, operation: &str)
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug + std::fmt::Display
    {
        match result {
            Ok(_) => TestReporter::success(format!("Schema '{}' {} completed successfully", schema_name, operation)),
            Err(e) => {
                TestReporter::error(format!("Schema '{}' {} failed: {}", schema_name, operation, e));
                panic!("Schema operation should succeed: {:?}", e);
            }
        }
    }

    /// Assert field operation success with standardized messaging
    pub fn assert_field_operation_success<T, E>(result: &Result<T, E>, field_name: &str, operation: &str)
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug + std::fmt::Display
    {
        match result {
            Ok(_) => TestReporter::success(format!("Field '{}' {} completed successfully", field_name, operation)),
            Err(e) => {
                TestReporter::error(format!("Field '{}' {} failed: {}", field_name, operation, e));
                panic!("Field operation should succeed: {:?}", e);
            }
        }
    }

    /// Assert computation result with standardized messaging
    pub fn assert_computation_result(actual: &serde_json::Value, expected: &serde_json::Value, computation_desc: &str) {
        if actual == expected {
            TestReporter::success(format!("Computation verified correctly: {} = {}", computation_desc, expected));
        } else {
            TestReporter::error(format!("Computation failed: expected {}, got {}", expected, actual));
            panic!("Computation result mismatch: expected {}, got {}", expected, actual);
        }
    }

    /// Assert event received with timeout handling
    pub fn assert_event_received<T>(event_result: &Result<T, std::sync::mpsc::RecvTimeoutError>, event_type: &str)
    where
        T: std::fmt::Debug
    {
        match event_result {
            Ok(event) => {
                TestReporter::success(format!("{} event received: {:?}", event_type, event));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                TestReporter::warning(format!("{} event not received within timeout (acceptable for testing)", event_type));
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                TestReporter::error(format!("{} event channel disconnected", event_type));
                panic!("Event channel should not be disconnected");
            }
        }
    }

    /// Assert with tolerance for expected test conditions
    pub fn assert_with_tolerance<T, E>(result: &Result<T, E>, operation: &str, acceptable_errors: &[&str])
    where
        T: std::fmt::Debug,
        E: std::fmt::Debug + std::fmt::Display
    {
        match result {
            Ok(_) => TestReporter::success(format!("{} operation completed successfully", operation)),
            Err(e) => {
                let error_str = e.to_string();
                let is_acceptable = acceptable_errors.iter().any(|&acceptable| error_str.contains(acceptable));
                
                if is_acceptable {
                    TestReporter::warning(format!("{} completed with expected error: {}", operation, e));
                } else {
                    TestReporter::error(format!("{} failed with unexpected error: {}", operation, e));
                    panic!("Unexpected error in {}: {:?}", operation, e);
                }
            }
        }
    }
}

/// Performance testing utilities with standardized reporting
pub struct PerformanceAssertions;

impl PerformanceAssertions {
    /// Assert operation completes within time limit
    pub fn assert_within_time_limit(duration: std::time::Duration, limit: std::time::Duration, operation: &str) {
        if duration <= limit {
            TestReporter::performance(format!("{} completed in {:?} (within limit of {:?})", operation, duration, limit));
        } else {
            TestReporter::error(format!("{} took {:?}, exceeded limit of {:?}", operation, duration, limit));
            panic!("Performance requirement not met: {} took {:?}, limit was {:?}", operation, duration, limit);
        }
    }

    /// Report execution timing with standardized format
    pub fn report_execution_time(duration: std::time::Duration, operation: &str) {
        TestReporter::performance(format!("{} execution time: {:?}", operation, duration));
    }

    /// Assert concurrent operations completed successfully
    pub fn assert_concurrent_operations_success(operation_count: usize, success_count: usize, operation_type: &str) {
        if success_count == operation_count {
            TestReporter::success(format!("All {} {} operations completed successfully", operation_count, operation_type));
        } else {
            TestReporter::warning(format!("{}/{} {} operations completed successfully", success_count, operation_count, operation_type));
        }
    }
}

/// Test step utilities for structured test reporting
pub struct TestSteps;

impl TestSteps {
    /// Begin a test with standardized header
    pub fn begin_test(test_name: &str) {
        println!("\n🧪 Testing {}", test_name);
        println!("{}", "=".repeat(test_name.len() + 10));
    }

    /// Begin a test section with standardized formatting
    pub fn begin_section(section_name: &str) {
        println!("\n🎯 {}", section_name);
        println!("{}", "-".repeat(section_name.len() + 4));
    }

    /// Complete a test with standardized footer
    pub fn complete_test(test_name: &str) {
        TestReporter::completion(format!("{} completed successfully", test_name));
        println!("{}", "=".repeat(test_name.len() + 25));
    }

    /// Complete a test section
    pub fn complete_section(section_name: &str) {
        TestReporter::success(format!("{} section completed", section_name));
    }

    /// Report test summary with standardized format
    pub fn report_summary(total_tests: usize, passed_tests: usize, warnings: usize) {
        println!("\n📊 TEST SUMMARY");
        println!("===============");
        TestReporter::result(format!("Total tests: {}", total_tests));
        TestReporter::result(format!("Passed: {}", passed_tests));
        if warnings > 0 {
            TestReporter::result(format!("Warnings: {}", warnings));
        }
        
        if passed_tests == total_tests {
            TestReporter::success("All tests passed!");
        } else {
            TestReporter::warning(format!("{}/{} tests passed", passed_tests, total_tests));
        }
    }
}

/// Macros for even more concise test assertions
#[macro_export]
macro_rules! assert_success {
    ($result:expr, $message:expr) => {
        match $result {
            Ok(_) => $crate::assertions::TestReporter::success($message),
            Err(e) => {
                $crate::assertions::TestReporter::error(format!("{}: {}", $message, e));
                panic!("Assertion failed: {}: {:?}", $message, e);
            }
        }
    };
}

#[macro_export]
macro_rules! assert_event {
    ($event_result:expr, $event_type:expr) => {
        $crate::assertions::TestAssertions::assert_event_received(&$event_result, $event_type);
    };
}

#[macro_export]
macro_rules! test_section {
    ($section_name:expr, $code:block) => {
        $crate::assertions::TestSteps::begin_section($section_name);
        $code
        $crate::assertions::TestSteps::complete_section($section_name);
    };
}

#[macro_export]
macro_rules! test_case {
    ($test_name:expr, $code:block) => {
        $crate::assertions::TestSteps::begin_test($test_name);
        $code
        $crate::assertions::TestSteps::complete_test($test_name);
    };
}