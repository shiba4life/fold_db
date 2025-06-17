//! Consolidated utilities module
//!
//! This module consolidates all utility functions and types that were previously
//! scattered across individual files. It provides:
//! - Configuration utilities and factories
//! - Testing utilities and database setup
//! - Validation helpers and common patterns

pub mod config;
pub mod test;
pub mod validation;

// Re-export commonly used types and functions for convenience
pub use config::{
    ConfigFactory, ConfigBuilder, MetadataBuilder, FieldsBuilder, VariablesBuilder,
    DefaultFieldConfig, StandardInitializers, EnvironmentConfig, EnvironmentConfiguration,
    PendingOperationsInit, PendingOperation,
};

pub use test::TestDatabaseFactory;

pub use validation::ValidationUtils;