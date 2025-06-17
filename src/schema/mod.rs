//! # Schema System
//!
//! The schema module defines the structure and behavior of data in the DataFold system.
//! Schemas define fields, their types, permissions, and payment requirements.
//!
//! ## Components
//!
//! * `core` - Unified core schema functionality (refactored into modular components)
//! * `core_types` - Core schema types including SchemaCore struct and related enums
//! * `operations` - CRUD operations, state management, loading/unloading operations
//! * `parsing` - JSON parsing, field conversion, schema loading from files
//! * `transforms` - Transform registration, mapping, field transformation
//! * `validation` - Schema validation logic and validator interface
//! * `utils` - File/directory helpers and utility functions
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

// Internal modules - refactored modular components
pub mod core_types;
pub mod operations;
pub mod parsing;
pub mod transforms;
pub mod validation;
pub mod utils;

// Internal modules - existing components
pub mod core;
pub mod duplicate_detection;
pub mod field_factory;
pub mod file_operations;
pub mod hasher;
pub mod types;

// Public re-exports from refactored modules
pub use core_types::{SchemaCore, SchemaLoadingReport, SchemaSource, SchemaState};
pub use operations::*;
pub use parsing::*;
pub use transforms::*;
pub use validation::*;
pub use utils::*;

// Public re-exports from existing modules
pub use field_factory::{
    DatabaseInitHelper, FieldBuilder, FieldFactory, TestEnvironment, TransformSetupHelper,
};
pub use types::{errors::SchemaError, schema::Schema, Transform};
pub mod validator;
pub use crate::{MutationType, Operation};
pub use duplicate_detection::SchemaDuplicateDetector;
pub use file_operations::SchemaFileOperations;
pub use hasher::SchemaHasher;
pub use validator::SchemaValidator;

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::SchemaCore;
}
