// Internal modules
pub(crate) mod schema_manager;
pub(crate) mod types;

// Re-export the public Schema type
pub use types::schema::Schema;

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::schema_manager::SchemaManager;
    pub use super::types::{
        errors::SchemaError,
        fields::SchemaField,
        operations::{Mutation, Query},
    };
}

// Internal re-exports for use within the crate
pub(crate) use schema_manager::SchemaManager;
pub(crate) use types::{SchemaError, SchemaField, Mutation, Query};
