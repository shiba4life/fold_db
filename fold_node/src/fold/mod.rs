//! # Fold Management
//!
//! Provides basic management for [`Fold`] structures including
//! loading from disk and persistence.

pub mod manager;
pub mod error;

pub use manager::FoldManager;
pub use error::FoldError;
