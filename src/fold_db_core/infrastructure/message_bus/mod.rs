//! # Internal Message Bus for FoldDB Core
//!
//! Provides a foundational event-driven messaging system for migrating fold_db_core
//! to an event-driven architecture. This module implements a simple pub/sub message bus
//! using Rust channels for internal communication between components.
//!
//! ## Design Goals
//! - Enable loose coupling between database components
//! - Support both synchronous and asynchronous event handling
//! - Provide a foundation for eventual migration to full event-driven architecture
//! - Maintain high performance with minimal overhead
//!
//! ## Usage Example
//! ```rust
//! use fold_node::fold_db_core::infrastructure::message_bus::{MessageBus, FieldValueSet};
//! use serde_json::json;
//!
//! let mut bus = MessageBus::new();
//!
//! // Register a consumer for field value events
//! let mut receiver = bus.subscribe::<FieldValueSet>();
//!
//! // Send an event
//! bus.publish(FieldValueSet {
//!     field: "user.name".to_string(),
//!     value: json!("Alice"),
//!     source: "mutation_engine".to_string(),
//! });
//!
//! // Receive the event
//! if let Ok(event) = receiver.try_recv() {
//!     println!("Received event: {:?}", event);
//! }
//! ```
//!
//! ## Module Structure
//!
//! The message bus has been decomposed into focused modules:
//!
//! - [`events`] - All event type definitions and the unified Event enum
//! - [`error_handling`] - Error types, retry logic, and dead letter queue support
//! - [`sync_bus`] - Synchronous message bus implementation using std::sync::mpsc
//! - [`async_bus`] - Asynchronous message bus implementation using tokio::sync::mpsc
//! - [`enhanced_bus`] - Enhanced features like retry, dead letter queue, and event sourcing
//! - [`constructors`] - Convenience constructor methods for all event types
//! - [`tests`] - Comprehensive test suite for all components
//!
//! ## Main Components
//!
//! ### Synchronous Message Bus
//! 
//! The [`MessageBus`] provides synchronous pub/sub messaging:
//!
//! ```rust
//! use fold_node::fold_db_core::infrastructure::message_bus::{MessageBus, FieldValueSet};
//! use serde_json::json;
//!
//! let bus = MessageBus::new();
//! let mut consumer = bus.subscribe::<FieldValueSet>();
//! 
//! let event = FieldValueSet::new("user.email", json!("alice@example.com"), "user_service");
//! bus.publish(event).unwrap();
//! 
//! let received_event = consumer.try_recv().unwrap();
//! ```
//!
//! ### Asynchronous Message Bus
//!
//! The [`AsyncMessageBus`] provides async pub/sub messaging:
//!
//! ```rust
//! use fold_node::fold_db_core::infrastructure::message_bus::{AsyncMessageBus, Event, AtomCreated};
//! use serde_json::json;
//!
//! # async fn example() {
//! let bus = AsyncMessageBus::new();
//! let mut consumer = bus.subscribe("AtomCreated").await;
//! 
//! let event = AtomCreated::new("atom-123", json!({"name": "Alice"}));
//! bus.publish_atom_created(event).await.unwrap();
//! 
//! let received_event = consumer.recv().await;
//! # }
//! ```
//!
//! ### Async Message Bus
//!
//! The [`AsyncMessageBus`] provides advanced features:
//!
//! ```rust
//! use fold_node::fold_db_core::infrastructure::message_bus::{AsyncMessageBus, FieldValueSet, Event};
//! use serde_json::json;
//!
//! # async fn example() {
//! let bus = AsyncMessageBus::new();
//!
//! let event = FieldValueSet::new("user.status", json!("active"), "user_service");
//! let wrapped_event = Event::FieldValueSet(event);
//! bus.publish_event(wrapped_event).await.unwrap();
//!
//! // Subscribe to events
//! let mut consumer = bus.subscribe("FieldValueSet").await;
//!
//! // Check for new events
//! let _received = consumer.try_recv();
//! # }
//! ```

// Re-export all public types and traits
pub use events::*;
pub use error_handling::{
    AsyncRecvError, AsyncTryRecvError, DeadLetterEvent, EventHistoryEntry, MessageBusError,
    MessageBusResult, RetryableEvent,
};
pub use sync_bus::{Consumer, MessageBus};
pub use async_bus::{AsyncConsumer, AsyncEventHandler, AsyncMessageBus};

// Import constructor implementations (these add methods to the event types)

// Internal modules
mod events;
mod error_handling;
mod sync_bus;
mod async_bus;
mod constructors;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_message_bus_integration() {
        let bus = MessageBus::new();
        let mut field_consumer = bus.subscribe::<FieldValueSet>();
        let mut atom_consumer = bus.subscribe::<AtomCreated>();

        // Test that different event types work correctly
        let field_event = FieldValueSet::new("integration.test", json!("success"), "test");
        let atom_event = AtomCreated::new("integration-atom", json!({"test": true}));

        bus.publish(field_event.clone()).unwrap();
        bus.publish(atom_event.clone()).unwrap();

        // Verify events are received by correct consumers
        assert_eq!(field_consumer.try_recv().unwrap(), field_event);
        assert_eq!(atom_consumer.try_recv().unwrap(), atom_event);

        // Verify no cross-contamination
        assert!(field_consumer.try_recv().is_err());
        assert!(atom_consumer.try_recv().is_err());
    }

    #[test]
    fn test_unified_event_publishing() {
        let bus = MessageBus::new();
        let mut consumer = bus.subscribe::<QueryExecuted>();

        let query_event = QueryExecuted::new("integration_query", "TestSchema", 100, 5);
        let unified_event = Event::QueryExecuted(query_event.clone());

        bus.publish_event(unified_event).unwrap();

        assert_eq!(consumer.try_recv().unwrap(), query_event);
    }

    #[tokio::test]
    async fn test_async_bus_integration() {
        let bus = AsyncMessageBus::new();
        let mut consumer = bus.subscribe("MutationExecuted").await;

        let mutation_event = MutationExecuted::new("create", "User", 75, 3);
        bus.publish_mutation_executed(mutation_event).await.unwrap();

        let received = consumer.recv().await;
        assert!(received.is_some());
    }


    #[test]
    fn test_event_constructors() {
        // Test that all constructor methods work correctly
        let field_event = FieldValueSet::new("test.field", json!("value"), "source");
        assert_eq!(field_event.field, "test.field");
        assert_eq!(field_event.value, json!("value"));
        assert_eq!(field_event.source, "source");

        let atom_event = AtomCreated::new("atom-id", json!({"data": "test"}));
        assert_eq!(atom_event.atom_id, "atom-id");
        assert_eq!(atom_event.data, json!({"data": "test"}));

        let query_event = QueryExecuted::new("range_query", "Schema", 150, 10);
        assert_eq!(query_event.query_type, "range_query");
        assert_eq!(query_event.schema, "Schema");
        assert_eq!(query_event.execution_time_ms, 150);
        assert_eq!(query_event.result_count, 10);

        let request = AtomCreateRequest::new(
            "req-123".to_string(),
            "User".to_string(),
            "pub-key".to_string(),
            None,
            json!({"name": "Test"}),
            Some("active".to_string()),
        );
        assert_eq!(request.correlation_id, "req-123");
        assert_eq!(request.schema_name, "User");
        assert_eq!(request.source_pub_key, "pub-key");
    }

    #[test]
    fn test_all_event_types_have_type_ids() {
        // Ensure all event types properly implement EventType
        assert_eq!(FieldValueSet::type_id(), "FieldValueSet");
        assert_eq!(AtomCreated::type_id(), "AtomCreated");
        assert_eq!(AtomUpdated::type_id(), "AtomUpdated");
        assert_eq!(AtomRefCreated::type_id(), "AtomRefCreated");
        assert_eq!(AtomRefUpdated::type_id(), "AtomRefUpdated");
        assert_eq!(SchemaLoaded::type_id(), "SchemaLoaded");
        assert_eq!(TransformExecuted::type_id(), "TransformExecuted");
        assert_eq!(SchemaChanged::type_id(), "SchemaChanged");
        assert_eq!(TransformTriggered::type_id(), "TransformTriggered");
        assert_eq!(QueryExecuted::type_id(), "QueryExecuted");
        assert_eq!(MutationExecuted::type_id(), "MutationExecuted");
        assert_eq!(Event::type_id(), "Event");

        // Test request/response types
        assert_eq!(AtomCreateRequest::type_id(), "AtomCreateRequest");
        assert_eq!(AtomCreateResponse::type_id(), "AtomCreateResponse");
        assert_eq!(FieldValueSetRequest::type_id(), "FieldValueSetRequest");
        assert_eq!(FieldValueSetResponse::type_id(), "FieldValueSetResponse");
        assert_eq!(SystemInitializationRequest::type_id(), "SystemInitializationRequest");
        assert_eq!(SystemInitializationResponse::type_id(), "SystemInitializationResponse");
    }

    #[test]
    fn test_error_types_display() {
        let send_error = MessageBusError::SendFailed {
            reason: "Test error".to_string(),
        };
        assert!(send_error.to_string().contains("Failed to send message"));

        let reg_error = MessageBusError::RegistrationFailed {
            event_type: "TestEvent".to_string(),
        };
        assert!(reg_error.to_string().contains("Failed to register consumer"));

        let disconnected_error = MessageBusError::ChannelDisconnected {
            event_type: "TestEvent".to_string(),
        };
        assert!(disconnected_error.to_string().contains("Channel disconnected"));
    }
}