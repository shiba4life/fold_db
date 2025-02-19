//! FoldDB - A distributed database with atomic versioning and schema validation
//! 
//! This crate provides the core FoldDB functionality and client/server implementations.

// Public modules that are part of the external API
pub mod datafold_node;

// Internal implementation modules
pub(crate) mod atom;
pub(crate) mod fees;
pub(crate) mod permissions;
pub(crate) mod schema;
pub(crate) mod schema_interpreter;
pub(crate) mod folddb;

// Re-export only the types that should be part of the public API
pub use datafold_node::{DataFoldNode, NodeConfig, NodeError, NodeResult};
pub use folddb::FoldDB;
pub use schema::Schema;

/// Testing utilities module, only available with the "test-utils" feature
#[cfg(any(test, feature = "test-utils"))]
pub mod testing {
    //! Module containing types and utilities needed for testing
    //! This module is only available when the "test-utils" feature is enabled

    pub use crate::atom::{Atom, AtomRef};
    pub use crate::fees::{
        payment_calculator::*,
        payment_manager::*,
        payment_config::SchemaPaymentConfig,
        types::*,
    };
    pub use crate::permissions::{
        permission_manager::*,
        types::policy::*,
        PermissionWrapper,
    };
    pub use crate::schema::{
        Schema,
        schema_manager::SchemaManager,
        types::{SchemaError, SchemaField, Mutation, Query},
    };
    pub use crate::schema_interpreter::{SchemaInterpreter, JsonSchemaDefinition};
}
