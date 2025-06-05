//! Enhanced message bus with error recovery and event sourcing
//!
//! This module provides advanced features like retry logic, dead letter queues,
//! and event history for event sourcing capabilities.

use super::async_bus::AsyncMessageBus;
use super::error_handling::{DeadLetterEvent, EventHistoryEntry, MessageBusResult, RetryableEvent};
use super::events::{Event, EventType};
use super::sync_bus::MessageBus;
use std::sync::Arc;

/// Enhanced message bus with error recovery and event sourcing
pub struct EnhancedMessageBus {
    /// Sync message bus
    sync_bus: Arc<MessageBus>,
    /// Async message bus
    async_bus: Arc<AsyncMessageBus>,
    /// Event history for event sourcing
    event_history: Arc<tokio::sync::Mutex<Vec<EventHistoryEntry>>>,
    /// Dead letter queue
    dead_letter_queue: Arc<tokio::sync::Mutex<Vec<DeadLetterEvent>>>,
    /// Retry queue
    retry_queue: Arc<tokio::sync::Mutex<Vec<RetryableEvent>>>,
    /// Event sequence counter
    sequence_counter: Arc<tokio::sync::Mutex<u64>>,
}

impl EnhancedMessageBus {
    /// Create a new enhanced message bus
    pub fn new() -> Self {
        Self {
            sync_bus: Arc::new(MessageBus::new()),
            async_bus: Arc::new(AsyncMessageBus::new()),
            event_history: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            dead_letter_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            retry_queue: Arc::new(tokio::sync::Mutex::new(Vec::new())),
            sequence_counter: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }

    /// Get access to sync message bus
    pub fn sync_bus(&self) -> Arc<MessageBus> {
        Arc::clone(&self.sync_bus)
    }

    /// Get access to async message bus
    pub fn async_bus(&self) -> Arc<AsyncMessageBus> {
        Arc::clone(&self.async_bus)
    }

    /// Publish event with error recovery
    pub async fn publish_with_retry<T: EventType>(&self, event: T, max_retries: u32, source: String) -> MessageBusResult<()> {
        let unified_event = self.to_unified_event(event);
        let retryable_event = RetryableEvent::new(unified_event.clone(), max_retries);
        
        // Record in event history
        self.record_event_history(unified_event, source).await;
        
        // Try to publish
        match self.async_bus.publish_event(retryable_event.event.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => {
                // Add to retry queue
                let mut retry_queue = self.retry_queue.lock().await;
                let mut retry_event = retryable_event;
                retry_event.increment_retry(e.to_string());
                retry_queue.push(retry_event);
                Err(e)
            }
        }
    }

    /// Process retry queue
    pub async fn process_retries(&self) -> usize {
        let mut retry_queue = self.retry_queue.lock().await;
        let mut processed = 0;
        let mut dead_letters = Vec::new();
        let mut successful_retries = Vec::new();

        for (index, retryable_event) in retry_queue.iter_mut().enumerate() {
            if !retryable_event.can_retry() {
                dead_letters.push((index, retryable_event.clone()));
                continue;
            }

            match self.async_bus.publish_event(retryable_event.event.clone()).await {
                Ok(_) => {
                    successful_retries.push(index);
                    processed += 1;
                }
                Err(e) => {
                    retryable_event.increment_retry(e.to_string());
                    if retryable_event.is_dead_letter() {
                        dead_letters.push((index, retryable_event.clone()));
                    }
                }
            }
        }

        // Move dead letters to dead letter queue
        if !dead_letters.is_empty() {
            let mut dlq = self.dead_letter_queue.lock().await;
            for (_, retryable_event) in &dead_letters {
                let dead_letter = DeadLetterEvent::new(
                    retryable_event.clone(),
                    "Max retries exceeded".to_string(),
                );
                dlq.push(dead_letter);
            }
        }

        // Remove processed events from retry queue
        let mut indices_to_remove: Vec<usize> = dead_letters.iter().map(|(i, _)| *i).collect();
        indices_to_remove.extend(successful_retries);
        indices_to_remove.sort_by(|a, b| b.cmp(a)); // Sort in reverse order

        for index in indices_to_remove {
            retry_queue.remove(index);
        }

        processed
    }

    /// Get event history for event sourcing
    pub async fn get_event_history(&self) -> Vec<EventHistoryEntry> {
        let history = self.event_history.lock().await;
        history.clone()
    }

    /// Get event history since a specific sequence number
    pub async fn get_event_history_since(&self, sequence_number: u64) -> Vec<EventHistoryEntry> {
        let history = self.event_history.lock().await;
        history
            .iter()
            .filter(|entry| entry.sequence_number > sequence_number)
            .cloned()
            .collect()
    }

    /// Replay events from history to reconstruct state
    pub async fn replay_events(&self, from_sequence: u64) -> MessageBusResult<usize> {
        let events = self.get_event_history_since(from_sequence).await;
        let mut replayed = 0;

        for entry in events {
            // Replay event through async bus
            self.async_bus.publish_event(entry.event).await?;
            replayed += 1;
        }

        Ok(replayed)
    }

    /// Get dead letter queue contents
    pub async fn get_dead_letters(&self) -> Vec<DeadLetterEvent> {
        let dlq = self.dead_letter_queue.lock().await;
        dlq.clone()
    }

    /// Clear dead letter queue
    pub async fn clear_dead_letters(&self) -> usize {
        let mut dlq = self.dead_letter_queue.lock().await;
        let count = dlq.len();
        dlq.clear();
        count
    }

    /// Get retry queue status
    pub async fn get_retry_queue_status(&self) -> (usize, usize) {
        let retry_queue = self.retry_queue.lock().await;
        let total = retry_queue.len();
        let ready_for_retry = retry_queue.iter().filter(|e| e.can_retry()).count();
        (total, ready_for_retry)
    }

    /// Record event in history
    async fn record_event_history(&self, event: Event, source: String) {
        let mut sequence_counter = self.sequence_counter.lock().await;
        *sequence_counter += 1;
        let sequence_number = *sequence_counter;
        drop(sequence_counter);

        let entry = EventHistoryEntry::new(event, source, sequence_number);
        
        let mut history = self.event_history.lock().await;
        history.push(entry);
    }

    /// Convert typed event to unified event (helper method)
    fn to_unified_event<T: EventType>(&self, _event: T) -> Event {
        // This is a simplified conversion - in practice you'd need proper trait bounds
        // or a more sophisticated event conversion system
        match std::any::type_name::<T>() {
            "fold_node::fold_db_core::message_bus::FieldValueSet" => {
                // For demo purposes - proper implementation would use proper casting
                Event::FieldValueSet(super::events::FieldValueSet::new("unknown", serde_json::Value::Null, "unknown"))
            }
            _ => Event::FieldValueSet(super::events::FieldValueSet::new("unknown", serde_json::Value::Null, "unknown"))
        }
    }
}

impl Default for EnhancedMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fold_db_core::infrastructure::message_bus::events::FieldValueSet;
    use serde_json::json;

    #[tokio::test]
    async fn test_enhanced_bus_creation() {
        let bus = EnhancedMessageBus::new();
        assert_eq!(bus.get_event_history().await.len(), 0);
        assert_eq!(bus.get_dead_letters().await.len(), 0);
        assert_eq!(bus.get_retry_queue_status().await, (0, 0));
    }

    #[tokio::test]
    async fn test_event_history_recording() {
        let bus = EnhancedMessageBus::new();
        let event = FieldValueSet::new("test.field", json!("value"), "source");
        
        // This would need proper event conversion to work fully
        let result = bus.publish_with_retry(event, 3, "test_source".to_string()).await;
        
        // Should have recorded in history regardless of publish result
        assert!(bus.get_event_history().await.len() > 0);
    }

    #[tokio::test]
    async fn test_dead_letter_queue_operations() {
        let bus = EnhancedMessageBus::new();
        
        // Initially empty
        assert_eq!(bus.get_dead_letters().await.len(), 0);
        
        // Clear empty queue
        assert_eq!(bus.clear_dead_letters().await, 0);
    }

    #[tokio::test]
    async fn test_retry_queue_status() {
        let bus = EnhancedMessageBus::new();
        let (total, ready) = bus.get_retry_queue_status().await;
        assert_eq!(total, 0);
        assert_eq!(ready, 0);
    }

    #[tokio::test]
    async fn test_process_retries_empty_queue() {
        let bus = EnhancedMessageBus::new();
        let processed = bus.process_retries().await;
        assert_eq!(processed, 0);
    }

    #[tokio::test]
    async fn test_replay_events_empty_history() {
        let bus = EnhancedMessageBus::new();
        let result = bus.replay_events(0).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_event_history_since() {
        let bus = EnhancedMessageBus::new();
        let history = bus.get_event_history_since(100).await;
        assert_eq!(history.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_and_async_bus_access() {
        let bus = EnhancedMessageBus::new();
        
        let sync_bus = bus.sync_bus();
        let async_bus = bus.async_bus();
        
        // Should be able to access both buses
        assert!(sync_bus.subscriber_count::<FieldValueSet>() == 0);
        assert!(async_bus.subscriber_count("FieldValueSet").await == 0);
    }
}