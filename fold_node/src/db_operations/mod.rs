// Core database operations
pub mod core;
mod atom_operations;
mod metadata_operations;
mod schema_operations;
mod transform_operations;
mod orchestrator_operations;
mod utility_operations;
mod error_utils;

// Tests module
mod tests;

// Re-export the main DbOperations struct and error utilities
pub use core::DbOperations;
pub use error_utils::ErrorUtils;
