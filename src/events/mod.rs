//! Security Operations Event Bus
//!
//! Centralized event bus architecture for verification monitoring across all DataFold platforms.
//! Provides real-time security event correlation, pluggable handlers, and cross-platform support.

pub mod verification_bus;
pub mod event_types;
pub mod handlers;
pub mod correlation;
pub mod transport;

pub use verification_bus::*;
pub use event_types::*;
pub use handlers::*;
pub use correlation::*;
pub use transport::*;