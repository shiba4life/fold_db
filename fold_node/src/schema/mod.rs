// Internal modules
pub(crate) mod core;
pub(crate) mod types;

// Public re-exports
pub use core::SchemaCore;
pub use types::{errors::SchemaError, schema::Schema};

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::SchemaCore;
}
