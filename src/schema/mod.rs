// Internal modules
pub(crate) mod schema_manager;
pub(crate) mod types;

// Public re-exports
pub use types::schema::Schema;
pub use types::errors::SchemaError;

/// Public prelude module containing types needed by tests and external code
pub mod prelude {}
