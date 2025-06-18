//! CLI argument definitions and parsing structures
//! 
//! This module contains all the argument-related types, enums, and structs
//! used by the DataFold CLI. These were extracted from the main CLI binary
//! to improve code organization and maintainability.


// Re-export all types from the split modules to maintain backward compatibility
pub use crate::cli::cli_types::*;
pub use crate::cli::crypto_commands::*;
pub use crate::cli::schema_commands::*;
pub use crate::cli::auth_commands::*;
pub use crate::cli::verification_commands::*;
pub use crate::cli::storage_types::*;

// The Commands enum is now defined in cli_types.rs to avoid circular dependencies