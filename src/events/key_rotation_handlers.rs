//! Event handlers for key rotation events
//!
//! This module provides specialized event handlers for processing key rotation events,
//! including network propagation, cache invalidation, and consistency guarantees.

use super::event_types::{
    KeyPropagationStatus, KeyRotationEvent, KeyRotationEventType, SecurityEvent,
};
use super::handlers::{EventHandler, EventHandlerResult};
use super::transport::{EventEnvelope, EventTransport, PlatformInfo};
use super::verification_bus::VerificationEventBus;
use crate::network::NetworkCore;
use async_trait::async_trait;
use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Configuration for key rotation event handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationHandlerConfig {
    /// Enable network propagation
    pub enable_propagation: bool,
    /// Maximum propagation timeout in seconds
    pub propagation_timeout_secs: u64,
    /// Maximum retry attempts for failed propagations
    pub max_retry_attempts: u32,
    /// Enable cache invalidation
    pub enable_cache_invalidation: bool,
    /// Target nodes for propagation (empty means all known nodes)
    pub target_nodes: Vec<String>,
    /// Enable conflict resolution
    pub enable_conflict_resolution: bool,
    /// Propagation batch size
    pub batch_size: usize,
}

impl Default for KeyRotationHandlerConfig {
    fn default() -> Self {
        Self {
            enable_propagation: true,
            propagation_timeout_secs: 300, // 5 minutes per requirements
            max_retry_attempts: 3,
            enable_cache_invalidation: true,
            target_nodes: Vec::new(),
            enable_conflict_resolution: true,
            batch_size: 10,
        }
    }
}

/// Status of network propagation for a specific operation
#[derive(Debug, Clone)]
pub struct PropagationStatus {
    /// Operation ID
    pub operation_id: Uuid,
    /// Total target nodes
    pub total_nodes: usize,
    /// Successfully updated nodes
    pub successful_nodes: Vec<String>,
    /// Failed nodes with error details
    pub failed_nodes: HashMap<String, String>,
    /// Start timestamp
    pub started_at: chrono::DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<chrono::DateTime<Utc>>,
    /// Current status
    pub status: KeyPropagationStatus,
}

/// Key rotation network propagation handler
pub struct KeyRotationPropagationHandler {
    /// Handler configuration
    config: KeyRotationHandlerConfig,
    /// Network core for peer communication
    network: Arc<RwLock<NetworkCore>>,
    /// Event transport for reliable delivery
    transport: Arc<Mutex<dyn EventTransport + Send + Sync>>,
    /// Event bus for publishing status updates
    event_bus: Arc<RwLock<VerificationEventBus>>,
    /// Active propagation operations
    active_operations: Arc<RwLock<HashMap<Uuid, PropagationStatus>>>,
    /// Handler name
    name: String,
}

impl KeyRotationPropagationHandler {
    /// Create a new key rotation propagation handler
    pub fn new(
        config: KeyRotationHandlerConfig,
        network: Arc<RwLock<NetworkCore>>,
        transport: Arc<Mutex<dyn EventTransport + Send + Sync>>,
        event_bus: Arc<RwLock<VerificationEventBus>>,
    ) -> Self {
        Self {
            config,
            network,
            transport,
            event_bus,
            active_operations: Arc::new(RwLock::new(HashMap::new())),
            name: "key_rotation_propagation_handler".to_string(),
        }
    }

    /// Set handler name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Propagate key rotation to network nodes
    async fn propagate_key_rotation(&self, event: &KeyRotationEvent) -> EventHandlerResult {
        let start_time = std::time::Instant::now();

        if !self.config.enable_propagation {
            return EventHandlerResult {
                handler_name: self.name.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert(
                        "propagation_disabled".to_string(),
                        serde_json::Value::Bool(true),
                    );
                    metadata
                },
            };
        }

        let operation_id = Uuid::parse_str(
            event
                .operation_id
                .as_ref()
                .unwrap_or(&"unknown".to_string()),
        )
        .unwrap_or_else(|_| Uuid::new_v4());

        // Determine target nodes
        let target_nodes = if self.config.target_nodes.is_empty() {
            self.get_all_known_nodes().await
        } else {
            self.config.target_nodes.clone()
        };

        if target_nodes.is_empty() {
            warn!("No target nodes available for key rotation propagation");
            return EventHandlerResult {
                handler_name: self.name.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metadata: {
                    let mut metadata = HashMap::new();
                    metadata.insert("no_target_nodes".to_string(), serde_json::Value::Bool(true));
                    metadata
                },
            };
        }

        // Initialize propagation status
        let propagation_status = PropagationStatus {
            operation_id,
            total_nodes: target_nodes.len(),
            successful_nodes: Vec::new(),
            failed_nodes: HashMap::new(),
            started_at: Utc::now(),
            completed_at: None,
            status: KeyPropagationStatus::InProgress,
        };

        // Track active operation
        {
            let mut active_ops = self.active_operations.write().await;
            active_ops.insert(operation_id, propagation_status.clone());
        }

        // Publish propagation started event
        self.publish_propagation_event(
            event,
            KeyRotationEventType::PropagationStarted,
            KeyPropagationStatus::InProgress,
            &target_nodes,
        )
        .await;

        // Propagate to nodes in batches
        let mut all_successful = true;
        let mut successful_nodes = Vec::new();
        let mut failed_nodes = HashMap::new();

        for batch in target_nodes.chunks(self.config.batch_size) {
            let batch_results = self.propagate_to_batch(event, batch.to_vec()).await;

            for (node_id, result) in batch_results {
                match result {
                    Ok(_) => {
                        successful_nodes.push(node_id.clone());
                        info!("Successfully propagated key rotation to node: {}", node_id);

                        // Publish node updated event
                        self.publish_node_updated_event(event, &node_id).await;
                    }
                    Err(error) => {
                        failed_nodes.insert(node_id.clone(), error);
                        all_successful = false;
                        warn!(
                            "Failed to propagate key rotation to node {}: {}",
                            node_id,
                            failed_nodes.get(&node_id).unwrap()
                        );
                    }
                }
            }
        }

        // Update final status
        let final_status = if all_successful {
            KeyPropagationStatus::Completed
        } else if successful_nodes.is_empty() {
            KeyPropagationStatus::Failed
        } else {
            KeyPropagationStatus::PartialFailure
        };

        // Update propagation status
        {
            let mut active_ops = self.active_operations.write().await;
            if let Some(status) = active_ops.get_mut(&operation_id) {
                status.successful_nodes = successful_nodes.clone();
                status.failed_nodes = failed_nodes.clone();
                status.completed_at = Some(Utc::now());
                status.status = final_status.clone();
            }
        }

        // Publish final propagation event
        let final_event_type = match final_status {
            KeyPropagationStatus::Completed => KeyRotationEventType::PropagationCompleted,
            _ => KeyRotationEventType::PropagationFailed,
        };

        self.publish_propagation_event(
            event,
            final_event_type,
            final_status.clone(),
            &target_nodes,
        )
        .await;

        // Create result metadata
        let mut metadata = HashMap::new();
        metadata.insert(
            "total_nodes".to_string(),
            serde_json::Value::Number(target_nodes.len().into()),
        );
        metadata.insert(
            "successful_nodes".to_string(),
            serde_json::Value::Number(successful_nodes.len().into()),
        );
        metadata.insert(
            "failed_nodes".to_string(),
            serde_json::Value::Number(failed_nodes.len().into()),
        );
        metadata.insert(
            "operation_id".to_string(),
            serde_json::Value::String(operation_id.to_string()),
        );
        metadata.insert(
            "propagation_status".to_string(),
            serde_json::json!(final_status),
        );

        if !failed_nodes.is_empty() {
            metadata.insert("failures".to_string(), serde_json::json!(failed_nodes));
        }

        EventHandlerResult {
            handler_name: self.name.clone(),
            success: matches!(
                final_status,
                KeyPropagationStatus::Completed | KeyPropagationStatus::PartialFailure
            ),
            duration: start_time.elapsed(),
            error: if all_successful {
                None
            } else {
                Some("Partial or complete propagation failure".to_string())
            },
            metadata,
        }
    }

    /// Propagate to a batch of nodes
    async fn propagate_to_batch(
        &self,
        event: &KeyRotationEvent,
        node_ids: Vec<String>,
    ) -> HashMap<String, Result<(), String>> {
        let mut results = HashMap::new();

        // Create event envelope for transport
        let envelope = EventEnvelope {
            version: "1.0".to_string(),
            source: PlatformInfo {
                platform_type: "datafold-node".to_string(),
                version: "1.0.0".to_string(),
                host: None,
                instance_id: Some(Uuid::new_v4().to_string()),
                metadata: HashMap::new(),
            },
            target: None,
            event: SecurityEvent::KeyRotation(event.clone()),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "propagation_target".to_string(),
                    serde_json::Value::Bool(true),
                );
                metadata.insert("target_nodes".to_string(), serde_json::json!(node_ids));
                metadata
            },
            envelope_timestamp: Utc::now(),
            envelope_id: Uuid::new_v4(),
        };

        // Send to each node with timeout
        for node_id in node_ids {
            let timeout_duration = Duration::from_secs(self.config.propagation_timeout_secs);
            let transport = self.transport.lock().await;

            let result =
                tokio::time::timeout(timeout_duration, transport.send_event(envelope.clone()))
                    .await;

            match result {
                Ok(Ok(transport_result)) => {
                    if transport_result.success {
                        results.insert(node_id, Ok(()));
                    } else {
                        results.insert(
                            node_id,
                            Err(transport_result
                                .error
                                .unwrap_or("Unknown transport error".to_string())),
                        );
                    }
                }
                Ok(Err(transport_error)) => {
                    results.insert(node_id, Err(transport_error.to_string()));
                }
                Err(_) => {
                    results.insert(node_id, Err("Timeout during propagation".to_string()));
                }
            }
        }

        results
    }

    /// Get all known network nodes
    async fn get_all_known_nodes(&self) -> Vec<String> {
        let network = self.network.read().await;
        let known_peers = network.known_peers();

        // Convert peer IDs to node IDs or use peer IDs as node identifiers
        known_peers
            .iter()
            .filter_map(|peer_id| network.get_node_id_for_peer(peer_id))
            .collect()
    }

    /// Publish a propagation status event
    async fn publish_propagation_event(
        &self,
        original_event: &KeyRotationEvent,
        event_type: KeyRotationEventType,
        status: KeyPropagationStatus,
        target_nodes: &[String],
    ) {
        let mut propagation_event = original_event.clone();
        propagation_event.rotation_type = event_type;
        propagation_event.propagation_status = status;
        propagation_event.target_nodes = target_nodes.to_vec();

        let security_event = SecurityEvent::KeyRotation(propagation_event);

        let event_bus = self.event_bus.read().await;
        if let Err(e) = event_bus.publish_event(security_event).await {
            error!("Failed to publish propagation event: {}", e);
        }
    }

    /// Publish a node updated event
    async fn publish_node_updated_event(&self, original_event: &KeyRotationEvent, node_id: &str) {
        let mut node_event = original_event.clone();
        node_event.rotation_type = KeyRotationEventType::NodeUpdated;
        node_event.target_nodes = vec![node_id.to_string()];

        let security_event = SecurityEvent::KeyRotation(node_event);

        let event_bus = self.event_bus.read().await;
        if let Err(e) = event_bus.publish_event(security_event).await {
            error!("Failed to publish node updated event: {}", e);
        }
    }

    /// Get propagation status for an operation
    pub async fn get_propagation_status(&self, operation_id: &Uuid) -> Option<PropagationStatus> {
        let active_ops = self.active_operations.read().await;
        active_ops.get(operation_id).cloned()
    }

    /// Clean up completed operations (call periodically)
    pub async fn cleanup_completed_operations(&self, max_age: Duration) {
        let mut active_ops = self.active_operations.write().await;
        let cutoff = Utc::now() - chrono::Duration::from_std(max_age).unwrap_or_default();

        active_ops.retain(|_, status| {
            match status.completed_at {
                Some(completed_at) => completed_at > cutoff,
                None => true, // Keep ongoing operations
            }
        });
    }
}

#[async_trait]
impl EventHandler for KeyRotationPropagationHandler {
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult {
        match event {
            SecurityEvent::KeyRotation(key_rotation_event) => {
                match key_rotation_event.rotation_type {
                    KeyRotationEventType::RotationStarted
                    | KeyRotationEventType::RotationCompleted => {
                        self.propagate_key_rotation(key_rotation_event).await
                    }
                    _ => {
                        // Don't handle other key rotation event types
                        EventHandlerResult {
                            handler_name: self.name.clone(),
                            success: true,
                            duration: Duration::from_millis(0),
                            error: None,
                            metadata: {
                                let mut metadata = HashMap::new();
                                metadata
                                    .insert("skipped".to_string(), serde_json::Value::Bool(true));
                                metadata.insert(
                                    "reason".to_string(),
                                    serde_json::Value::String(
                                        "Not a propagation event".to_string(),
                                    ),
                                );
                                metadata
                            },
                        }
                    }
                }
            }
            _ => {
                // Don't handle non-key-rotation events
                EventHandlerResult {
                    handler_name: self.name.clone(),
                    success: true,
                    duration: Duration::from_millis(0),
                    error: None,
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("skipped".to_string(), serde_json::Value::Bool(true));
                        metadata.insert(
                            "reason".to_string(),
                            serde_json::Value::String("Not a key rotation event".to_string()),
                        );
                        metadata
                    },
                }
            }
        }
    }

    fn handler_name(&self) -> String {
        self.name.clone()
    }

    fn can_handle(&self, event: &SecurityEvent) -> bool {
        matches!(event, SecurityEvent::KeyRotation(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{
        OperationResult, PlatformSource, SecurityEventCategory, VerificationEvent,
    };
    use crate::events::transport::{InMemoryTransport, TransportConfig};
    use crate::network::NetworkConfig;
    use crate::security_types::Severity;
    use std::time::Duration;

    #[tokio::test]
    async fn test_key_rotation_propagation_handler() {
        let config = KeyRotationHandlerConfig::default();
        let network_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/0");
        let network = Arc::new(RwLock::new(NetworkCore::new(network_config).await.unwrap()));

        let transport_config = TransportConfig::default();
        let platform_info = PlatformInfo {
            platform_type: "test".to_string(),
            version: "1.0.0".to_string(),
            host: None,
            instance_id: None,
            metadata: HashMap::new(),
        };
        let transport = Arc::new(Mutex::new(InMemoryTransport::new(
            transport_config,
            platform_info,
        )));

        let event_bus = Arc::new(RwLock::new(VerificationEventBus::with_default_config()));

        let handler = KeyRotationPropagationHandler::new(config, network, transport, event_bus);

        // Create test key rotation event
        let base_event = VerificationEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: SecurityEventCategory::KeyRotation,
            severity: Severity::Info,
            platform: PlatformSource::DataFoldNode,
            component: "key_rotation".to_string(),
            operation: "rotate_key".to_string(),
            actor: Some("test_user".to_string()),
            result: OperationResult::Success,
            duration: Some(Duration::from_millis(100)),
            metadata: HashMap::new(),
            correlation_id: Some(Uuid::new_v4()),
            trace_id: None,
            session_id: None,
            environment: Some("test".to_string()),
        };

        let key_rotation_event = KeyRotationEvent {
            base: base_event,
            rotation_type: KeyRotationEventType::RotationStarted,
            user_id: Some("test_user".to_string()),
            old_key_id: Some("old_key_123".to_string()),
            new_key_id: Some("new_key_456".to_string()),
            rotation_reason: "Scheduled".to_string(),
            operation_id: Some(Uuid::new_v4().to_string()),
            target_nodes: vec!["node1".to_string(), "node2".to_string()],
            propagation_status: KeyPropagationStatus::Pending,
            affected_associations: Some(5),
            rotation_metadata: HashMap::new(),
        };

        let security_event = SecurityEvent::KeyRotation(key_rotation_event);

        // Test handling the event
        let result = handler.handle_event(&security_event).await;

        assert!(result.success);
        assert_eq!(result.handler_name, "key_rotation_propagation_handler");
    }
}
