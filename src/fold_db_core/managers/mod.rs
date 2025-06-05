//! Core managers responsible for different aspects of data management
//! 
//! This module contains all the core manager components that handle
//! specific aspects of the FoldDB system:
//! - Atom management (storage and retrieval)
//! - Field management (field operations and updates)
//! - Collection management (collection operations)
//! - Schema management (schema lifecycle and state)

pub mod atom;
pub mod field;
// pub mod collection; // Temporarily disabled for UI testing
pub mod schema;

pub use atom::AtomManager;
pub use field::FieldManager;
// pub use collection::CollectionManager; // Temporarily disabled for UI testing
pub use schema::*;