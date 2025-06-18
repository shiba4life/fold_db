//! Test module organization for database operations
//!
//! This module organizes tests for the database operations, particularly
//! focusing on encryption wrapper functionality and related components.

pub mod encryption_wrapper_tests;

// Re-export test utilities for use in other test modules
pub use encryption_wrapper_tests::*;