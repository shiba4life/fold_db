//! Interpreter module for the transform DSL.
//!
//! This module contains a modular interpreter for the transform DSL.
//! It evaluates an AST to produce a result value.

// Module declarations
pub mod builtins;
pub mod engine;

#[cfg(test)]
pub mod tests;

// Public re-exports for the main API
pub use builtins::{builtin_functions, TransformFunction};
pub use engine::Interpreter;