//! Test modules for the message bus system
//!
//! This module contains comprehensive tests for all message bus components

#[cfg(test)]
mod tests {
    use super::super::events::*;
    use super::super::async_bus::AsyncMessageBus;
    use super::super::enhanced_bus::EnhancedMessageBus;
    use serde_json::json;

    #[test]
    fn test_unified_event_types() {
        let field_event = Event::FieldValueSet(FieldValueSet::new("field", json!("value"), "source"));
        let atom_event = Event::AtomCreated(AtomCreated::new("atom-id", json!({})));
        let query_event = Event::QueryExecuted(QueryExecuted::new("range_query", "test_schema", 150, 5));
        let mutation_event = Event::MutationExecuted(MutationExecuted::new("create", "test_schema", 75, 3));
        
        assert_eq!(field_event.event_type(), "FieldValueSet");
        assert_eq!(atom_event.event_type(), "AtomCreated");
        assert_eq!(query_event.event_type(), "QueryExecuted");
        assert_eq!(mutation_event.event_type(), "MutationExecuted");
    }

    #[test]
    fn test_query_mutation_event_serialization() {
        let query_event = QueryExecuted::new("single_query", "Analytics", 75, 1);
        let serialized = serde_json::to_string(&query_event).unwrap();
        let deserialized: QueryExecuted = serde_json::from_str(&serialized).unwrap();
        assert_eq!(query_event, deserialized);

        let mutation_event = MutationExecuted::new("create", "Inventory", 100, 4);
        let serialized = serde_json::to_string(&mutation_event).unwrap();
        let deserialized: MutationExecuted = serde_json::from_str(&serialized).unwrap();
        assert_eq!(mutation_event, deserialized);
    }

    // ========== Request/Response Event Tests ==========

    #[test]
    fn test_atom_create_request_response() {
        let request = AtomCreateRequest::new(
            "req-123".to_string(),
            "User".to_string(),
            "pub-key-456".to_string(),
            Some("prev-atom-789".to_string()),
            json!({"name": "Alice"}),
            Some("active".to_string()),
        );

        assert_eq!(request.correlation_id, "req-123");
        assert_eq!(request.schema_name, "User");
        assert_eq!(request.source_pub_key, "pub-key-456");
        assert_eq!(request.prev_atom_uuid, Some("prev-atom-789".to_string()));
        assert_eq!(request.content, json!({"name": "Alice"}));
        assert_eq!(request.status, Some("active".to_string()));

        let response = AtomCreateResponse::new(
            "req-123".to_string(),
            true,
            Some("atom-999".to_string()),
            None,
            Some(json!({"id": "atom-999", "name": "Alice"})),
        );

        assert_eq!(response.correlation_id, "req-123");
        assert!(response.success);
        assert_eq!(response.atom_uuid, Some("atom-999".to_string()));
        assert!(response.error.is_none());
        assert!(response.atom_data.is_some());
    }

    #[test]
    fn test_field_value_set_request_response() {
        let request = FieldValueSetRequest::new(
            "field-req-001".to_string(),
            "Product".to_string(),
            "price".to_string(),
            json!(99.99),
            "seller-key".to_string(),
        );

        assert_eq!(request.correlation_id, "field-req-001");
        assert_eq!(request.schema_name, "Product");
        assert_eq!(request.field_name, "price");
        assert_eq!(request.value, json!(99.99));
        assert_eq!(request.source_pub_key, "seller-key");

        let response = FieldValueSetResponse::new(
            "field-req-001".to_string(),
            true,
            Some("aref-555".to_string()),
            None,
        );

        assert_eq!(response.correlation_id, "field-req-001");
        assert!(response.success);
        assert_eq!(response.aref_uuid, Some("aref-555".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_schema_load_request_response() {
        let request = SchemaLoadRequest::new(
            "schema-load-001".to_string(),
            "UserProfile".to_string(),
        );

        assert_eq!(request.correlation_id, "schema-load-001");
        assert_eq!(request.schema_name, "UserProfile");

        let response = SchemaLoadResponse::new(
            "schema-load-001".to_string(),
            true,
            Some(json!({"name": "UserProfile", "fields": {}})),
            None,
        );

        assert_eq!(response.correlation_id, "schema-load-001");
        assert!(response.success);
        assert!(response.schema_data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_atom_history_request_response() {
        let request = AtomHistoryRequest::new(
            "history-req-001".to_string(),
            "aref-123".to_string(),
        );

        assert_eq!(request.correlation_id, "history-req-001");
        assert_eq!(request.aref_uuid, "aref-123");

        let response = AtomHistoryResponse::new(
            "history-req-001".to_string(),
            true,
            Some(vec![
                json!({"version": 1, "data": "old"}),
                json!({"version": 2, "data": "new"}),
            ]),
            None,
        );

        assert_eq!(response.correlation_id, "history-req-001");
        assert!(response.success);
        assert_eq!(response.history.as_ref().unwrap().len(), 2);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_collection_update_request_response() {
        let request = CollectionUpdateRequest::new(
            "collection-001".to_string(),
            "BlogPost".to_string(),
            "tags".to_string(),
            "add".to_string(),
            json!("rust"),
            "author-key".to_string(),
            None,
        );

        assert_eq!(request.correlation_id, "collection-001");
        assert_eq!(request.schema_name, "BlogPost");
        assert_eq!(request.field_name, "tags");
        assert_eq!(request.operation, "add");
        assert_eq!(request.value, json!("rust"));
        assert_eq!(request.source_pub_key, "author-key");
        assert!(request.item_id.is_none());

        let response = CollectionUpdateResponse::new(
            "collection-001".to_string(),
            true,
            None,
        );

        assert_eq!(response.correlation_id, "collection-001");
        assert!(response.success);
        assert!(response.error.is_none());
    }

    #[test]
    fn test_system_initialization_request_response() {
        let request = SystemInitializationRequest::new(
            "sys-init-001".to_string(),
            "/path/to/db".to_string(),
            Some(json!({"worker_threads": 4})),
        );

        assert_eq!(request.correlation_id, "sys-init-001");
        assert_eq!(request.db_path, "/path/to/db");
        assert!(request.orchestrator_config.is_some());

        let response = SystemInitializationResponse::new(
            "sys-init-001".to_string(),
            true,
            None,
        );

        assert_eq!(response.correlation_id, "sys-init-001");
        assert!(response.success);
        assert!(response.error.is_none());
    }

    // ========== Event Type Tests ==========

    #[test]
    fn test_all_event_type_ids() {
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
        
        // Request/Response types
        assert_eq!(AtomCreateRequest::type_id(), "AtomCreateRequest");
        assert_eq!(AtomCreateResponse::type_id(), "AtomCreateResponse");
        assert_eq!(FieldValueSetRequest::type_id(), "FieldValueSetRequest");
        assert_eq!(FieldValueSetResponse::type_id(), "FieldValueSetResponse");
        assert_eq!(SchemaLoadRequest::type_id(), "SchemaLoadRequest");
        assert_eq!(SchemaLoadResponse::type_id(), "SchemaLoadResponse");
        assert_eq!(AtomHistoryRequest::type_id(), "AtomHistoryRequest");
        assert_eq!(AtomHistoryResponse::type_id(), "AtomHistoryResponse");
        assert_eq!(CollectionUpdateRequest::type_id(), "CollectionUpdateRequest");
        assert_eq!(CollectionUpdateResponse::type_id(), "CollectionUpdateResponse");
        assert_eq!(SystemInitializationRequest::type_id(), "SystemInitializationRequest");
        assert_eq!(SystemInitializationResponse::type_id(), "SystemInitializationResponse");
    }

    // ========== Async Tests ==========

    #[tokio::test]
    async fn test_async_enhanced_bus_integration() {
        let bus = EnhancedMessageBus::new();
        
        // Test basic operations
        assert_eq!(bus.get_event_history().await.len(), 0);
        assert_eq!(bus.get_dead_letters().await.len(), 0);
        assert_eq!(bus.get_retry_queue_status().await, (0, 0));
        
        // Test accessing sub-buses
        let sync_bus = bus.sync_bus();
        let async_bus = bus.async_bus();
        
        assert_eq!(sync_bus.subscriber_count::<FieldValueSet>(), 0);
        assert_eq!(async_bus.subscriber_count("AtomCreated").await, 0);
    }

    #[tokio::test]
    async fn test_async_message_bus_subscription() {
        let bus = AsyncMessageBus::new();
        
        // Test subscription
        let _consumer1 = bus.subscribe("FieldValueSet").await;
        let _consumer2 = bus.subscribe("AtomCreated").await;
        
        assert_eq!(bus.subscriber_count("FieldValueSet").await, 1);
        assert_eq!(bus.subscriber_count("AtomCreated").await, 1);
        assert_eq!(bus.subscriber_count("UnknownEvent").await, 0);
    }

    #[test]
    fn test_error_types() {
        use super::super::error_handling::*;
        
        let send_error = MessageBusError::SendFailed {
            reason: "Channel closed".to_string(),
        };
        assert!(send_error.to_string().contains("Failed to send message"));
        
        let reg_error = MessageBusError::RegistrationFailed {
            event_type: "TestEvent".to_string(),
        };
        assert!(reg_error.to_string().contains("Failed to register consumer"));
        
        let disc_error = MessageBusError::ChannelDisconnected {
            event_type: "TestEvent".to_string(),
        };
        assert!(disc_error.to_string().contains("Channel disconnected"));
    }

    #[test]
    fn test_retryable_event() {
        use super::super::error_handling::RetryableEvent;
        
        let event = Event::FieldValueSet(FieldValueSet::new("test", json!("value"), "source"));
        let mut retryable = RetryableEvent::new(event, 3);
        
        assert!(retryable.can_retry());
        assert!(!retryable.is_dead_letter());
        assert_eq!(retryable.retry_count, 0);
        
        retryable.increment_retry("First error".to_string());
        assert!(retryable.can_retry());
        assert!(!retryable.is_dead_letter());
        assert_eq!(retryable.retry_count, 1);
        assert_eq!(retryable.last_error, Some("First error".to_string()));
        
        // Exhaust retries
        retryable.increment_retry("Second error".to_string());
        retryable.increment_retry("Third error".to_string());
        
        assert!(!retryable.can_retry());
        assert!(retryable.is_dead_letter());
        assert_eq!(retryable.retry_count, 3);
    }

    #[test]
    fn test_dead_letter_event() {
        use super::super::error_handling::{DeadLetterEvent, RetryableEvent};
        
        let event = Event::AtomCreated(AtomCreated::new("atom-123", json!({})));
        let retryable = RetryableEvent::new(event, 2);
        
        let dead_letter = DeadLetterEvent::new(retryable, "Max retries exceeded".to_string());
        
        assert_eq!(dead_letter.reason, "Max retries exceeded");
        assert_eq!(dead_letter.retryable_event.max_retries, 2);
    }

    #[test]
    fn test_event_history_entry() {
        use super::super::error_handling::EventHistoryEntry;
        
        let event = Event::SchemaLoaded(SchemaLoaded::new("TestSchema", "success"));
        let entry = EventHistoryEntry::new(event, "test_component".to_string(), 42);
        
        assert_eq!(entry.source, "test_component");
        assert_eq!(entry.sequence_number, 42);
        assert!(!entry.event_id.is_empty());
        
        match entry.event {
            Event::SchemaLoaded(schema_event) => {
                assert_eq!(schema_event.schema_name, "TestSchema");
                assert_eq!(schema_event.status, "success");
            }
            _ => panic!("Expected SchemaLoaded event"),
        }
    }
}