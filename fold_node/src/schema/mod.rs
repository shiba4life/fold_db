//! # Schema System
//!
//! The schema module defines the structure and behavior of data in the DataFold system.
//! Schemas define fields, their types, permissions, and payment requirements.
//!
//! ## Components
//!
//! * `core` - Core schema functionality including loading, validation, and field mapping
//! * `types` - Schema-related data structures and type definitions
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
pub(crate) mod core;
pub(crate) mod storage;
pub mod types;


// Public re-exports
pub use core::SchemaCore;
pub use storage::SchemaStorage;
pub use types::{errors::SchemaError, schema::Schema, Transform};
pub mod validator;
pub use validator::SchemaValidator;
pub use crate::{Operation, MutationType};

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::SchemaCore;
}
