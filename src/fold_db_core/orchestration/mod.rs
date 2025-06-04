//! Orchestration components for coordinating system operations
//! 
//! This module contains orchestration components that coordinate
//! complex operations across multiple system components:
//! - Transform orchestration and coordination
//! - Event-driven FoldDB orchestration
//! - Event-driven database operations

pub mod transform_orchestrator;
pub mod event_driven_folddb;
pub mod event_driven_db_operations;

pub use transform_orchestrator::TransformOrchestrator;