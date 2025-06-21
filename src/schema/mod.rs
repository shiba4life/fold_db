//! # Schema System
//!
//! The schema module defines the structure and behavior of data in the DataFold system.
//! Schemas define fields, their types, permissions, and payment requirements.
//!
//! ## Components
//!
//! * `core` - Core schema functionality including loading, validation, and field mapping
//! * `types` - Schema-related data structures and type definitions
//! * `hasher` - Schema hashing and integrity verification with configurable directory paths
//! * `file_operations` - File-based operations for reading and writing schemas
//! * `duplicate_detection` - Duplicate detection and conflict resolution for schemas
//! * `validator` - Schema validation logic
//!
//! ## Architecture
//!
//! Schemas in DataFold define the structure of data and the operations that can be
//! performed on it. Each schema has a name and a set of fields, each with its own
//! type, permissions, and payment requirements.
//!
//! The schema system supports field mapping between schemas, allowing fields from
//! one schema to reference fields in another. This creates a graph-like structure
//! of related data across schemas.
//!
//! Schemas are loaded from JSON definitions, validated, and then used to process
//! queries and mutations against the database.

// Internal modules
pub mod core;
pub mod discovery;
pub mod persistence;
pub mod transform;
pub mod duplicate_detection;
pub mod field_factory;
pub mod file_operations;
pub mod hasher;
pub mod types;
pub mod validator;

// Public re-exports
pub use core::*;
pub use field_factory::*;
pub use types::*;
pub use duplicate_detection::SchemaDuplicateDetector;
pub use file_operations::SchemaFileOperations;
pub use hasher::SchemaHasher;
pub use validator::SchemaValidator;

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::SchemaCore;
}
