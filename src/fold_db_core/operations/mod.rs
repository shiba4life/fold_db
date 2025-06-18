//! Operations modules for FoldDB
//!
//! This module contains the split operations from the main FoldDB coordinator:
//! - **mutations**: Event-driven mutation processing and validation
//! - **queries**: Query processing with range schema support and optimization
//! - **encryption**: Encryption wrapper management and key operations

pub mod encryption;
pub mod mutations;
pub mod queries;

pub use encryption::EncryptionOperations;
pub use mutations::MutationOperations;
pub use queries::QueryOperations;