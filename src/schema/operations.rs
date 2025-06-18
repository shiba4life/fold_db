//! Schema operations - unified interface
//!
//! This module provides a unified interface to schema operations by re-exporting
//! functionality from the specialized operation modules:
//! - `schema_crud` - Schema CRUD operations (add, approve, block)
//! - `schema_discovery` - Schema discovery and loading from disk
//! - `schema_field_mapping` - Field mapping between schemas
//! - `schema_state_management` - Schema state transitions and management
//!
//! This module maintains backwards compatibility while providing a cleaner
//! internal organization of schema operation code.

// Re-export all functionality from the specialized modules
// This ensures that existing code using `use crate::schema::operations::*` continues to work

pub use super::schema_crud::*;
pub use super::schema_discovery::*;
pub use super::schema_field_mapping::*;
pub use super::schema_state_management::*;

// Note: Tests are now in schema_operations_tests.rs
// This provides better organization and separation of concerns