//! Orchestration components for coordinating system operations
//!
//! This module contains orchestration components that coordinate
//! complex operations across multiple system components:
//! - Transform orchestration and coordination
//! - Event-driven FoldDB orchestration
//! - Event-driven database operations

pub mod transform_orchestrator;

// New decomposed orchestration components
pub mod queue_manager;
pub mod persistence_manager;
pub mod event_monitor;
pub mod execution_coordinator;

pub use transform_orchestrator::{TransformOrchestrator, TransformQueue};
pub use queue_manager::{QueueManager, QueueItem, QueueState};
pub use persistence_manager::PersistenceManager;
pub use event_monitor::EventMonitor;
pub use execution_coordinator::{ExecutionCoordinator, ExecutionStats};