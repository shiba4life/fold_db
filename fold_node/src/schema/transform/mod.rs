//! # Transform System
//!
//! The transform module implements a domain-specific language (DSL) for writing
//! secure, auditable transformations in the Datafold platform.
//!
//! ## Components
//!
//! * `ast` - Abstract Syntax Tree definitions for the transform DSL
//! * `parser` - Parser for the transform DSL
//! * `interpreter` - Interpreter for executing transforms
//! * `executor` - High-level executor for applying transforms to field values
//!
//! ## Architecture
//!
//! Transforms in Datafold define how data from source fields is processed to produce
//! derived values. The transform system consists of:
//!
//! 1. A parser that converts transform DSL code into an AST
//! 2. An interpreter that executes the AST to produce a result
//! 3. An executor that handles the integration with the schema system

pub mod ast;
pub mod parser;
pub mod interpreter;
pub mod executor;
pub mod registry;
#[cfg(test)]
mod simple_test;

// Public re-exports
pub use ast::{Expression, Value, Operator, UnaryOperator};
pub use interpreter::Interpreter;
pub use parser::TransformParser;
pub use executor::TransformExecutor;
pub use registry::{TransformRegistry, GetAtomFn, CreateAtomFn, UpdateAtomRefFn};
pub use crate::schema::types::Transform;