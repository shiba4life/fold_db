//! Test modules for TransformManager decomposition validation
//!
//! This module organizes integration and unit tests for validating that the
//! decomposition of transform_manager/manager.rs into multiple focused modules
//! maintains all functionality while improving code organization.

pub mod integration;
pub mod unit;

// Common test utilities and fixtures
pub mod common;