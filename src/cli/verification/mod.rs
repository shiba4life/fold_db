//! CLI signature verification utilities implementing RFC 9421 HTTP Message Signatures
//!
//! This module provides comprehensive signature verification capabilities for the DataFold CLI,
//! enabling validation of server responses and command-line signature verification tools.

// Core types and errors
pub mod verification_types;
pub use verification_types::*;

// Configuration and policy management
pub mod verification_config;
pub use verification_config::*;

// Signature data structures and content digest handling
pub mod signature_data;
pub use signature_data::*;

// Core verification engine and algorithms
pub mod verification_engine;
pub use verification_engine::*;

// Debugging and analysis tools
pub mod verification_inspector;
pub use verification_inspector::*;

// Test suite
#[cfg(test)]
pub mod verification_tests;