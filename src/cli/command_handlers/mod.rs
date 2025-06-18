//! Command handler modules for the DataFold CLI
//! 
//! This module organizes command implementations into logical groups,
//! extracted from the main CLI binary for better maintainability.

pub mod auth_handler;
pub mod crypto_handler;
pub mod schema_handler;

// Re-export main handler functions for easy access
pub use auth_handler::*;
pub use crypto_handler::*;
pub use schema_handler::*;