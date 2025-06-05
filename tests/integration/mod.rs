//! Integration tests for TransformManager decomposition and system functionality
//!
//! These tests validate that all decomposed modules work together correctly
//! as a cohesive system, ensuring no functionality was lost during decomposition.
//! Also includes tests for system-level functionality like database reset.

pub mod transform_manager_decomposition_tests;
pub mod transform_result_persistence_tests;
pub mod system_routes_tests;