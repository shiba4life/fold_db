//! Infrastructure components for system foundation
//! 
//! This module contains core infrastructure components:
//! - Message bus for event-driven communication
//! - System initialization utilities
//! - Async API for async operations
//! - Event monitoring and observability

pub mod message_bus;
pub mod init;
pub mod async_api;
pub mod event_monitor;

pub use message_bus::MessageBus;
pub use event_monitor::EventMonitor;