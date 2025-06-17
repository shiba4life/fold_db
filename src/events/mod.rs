//! Security Operations Event Bus
//!
//! Centralized event bus architecture for verification monitoring across all DataFold platforms.
//! Provides real-time security event correlation, pluggable handlers, and cross-platform support.

pub mod correlation;
pub mod event_types;
pub mod handlers;
pub mod key_rotation_handlers;
pub mod transport;
pub mod verification_bus;
pub mod verification_bus_config;
pub mod verification_bus_types;
pub mod verification_processing;
pub mod verification_statistics;

#[cfg(test)]
pub mod verification_bus_tests;

pub use correlation::*;
pub use event_types::*;
pub use handlers::*;
pub use key_rotation_handlers::*;
pub use transport::*;
pub use verification_bus::{VerificationEventBus};
pub use verification_bus_config::VerificationBusConfig;
pub use verification_bus_types::{EventBusStatistics, EventProcessingResult};
