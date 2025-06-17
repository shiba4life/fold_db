//! Core schema functionality - compatibility layer
//!
//! This module provides backward compatibility and acts as a thin wrapper
//! around the refactored modular schema components.
//!
//! The actual functionality has been moved to:
//! - `core_types` - Core types and SchemaCore struct
//! - `operations` - CRUD and state management operations
//! - `parsing` - JSON parsing and field conversion
//! - `transforms` - Transform registration and mapping
//! - `validation` - Schema validation logic
//! - `utils` - Utility functions for file operations

// Re-export all core types and functionality for backward compatibility

// Make sure these types are available at the schema::core path for backward compatibility
pub use super::core_types::{SchemaCore, SchemaLoadingReport, SchemaSource, SchemaState};

// This file now serves as a compatibility layer.
// All the actual implementation has been moved to the modular components:
//
// SchemaCore struct and basic methods -> core_types.rs
// CRUD operations and state management -> operations.rs  
// JSON parsing and field conversion -> parsing.rs
// Transform registration and mapping -> transforms.rs
// Schema validation logic -> validation.rs
// Utility functions and file operations -> utils.rs
//
// The modules are designed to work together while being independently testable
// and maintainable. Each module has a clear, focused responsibility.
