// Internal modules
pub(crate) mod core;
pub(crate) mod types;

// Public re-exports
pub use types::{
    schema::Schema,
    errors::SchemaError,
};
pub use core::SchemaCore;

/// Public prelude module containing types needed by tests and external code
pub mod prelude {
    pub use super::SchemaCore;
}
