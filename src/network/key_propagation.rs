//! Network propagation for key rotation events
//!
//! This module handles the reliable distribution of key rotation events across
//! the DataFold network, ensuring eventual consistency and providing conflict resolution.

use super::core::NetworkCore;
use super::error::{NetworkError, NetworkResult};
use crate::events::event_types::{
    KeyPropagationStatus, KeyRotationEvent, SecurityEvent,
};
use crate::events::transport::{EventEnvelope, EventTransport, PlatformInfo, TransportResult};
use chrono::{DateTime, Utc};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Configuration for key propagation across the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyPropagationConfig {
    /// Maximum time to wait for propagation completion
    pub propagation_timeout_secs: u64,
    /// Maximum number of retry attempts for failed nodes
    pub max_retry_attempts: u32,
    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,
    /// Batch size for parallel propagation
    pub batch_size: usize,
    /// Enable conflict resolution for concurrent rotations
    pub enable_conflict_resolution: bool,
    /// Minimum number of confirmations required
    pub min_confirmations: usize,
    /// Network partition recovery timeout in seconds
    pub partition_recovery_timeout_secs: u64,
    /// Enable graceful degradation mode
    pub graceful_degradation: bool,
}

impl Default for KeyPropagationConfig {
    fn default() -> Self {
        Self {
            propagation_timeout_secs: 300, // 5 minutes per requirements
            max_retry_attempts: 3,
            retry_delay_ms: 1000,
            batch_size: 10,
            enable_conflict_resolution: true,
            min_confirmations: 1,
            partition_recovery_timeout_secs: 1800, // 30 minutes
            graceful_degradation: true,
        }
    }
}

/// Status of a propagation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationOperation {
    /// Operation identifier
    pub operation_id: Uuid,
    /// Key rotation event being propagated
    pub event: KeyRotationEvent,
    /// Target nodes for propagation
    pub target_nodes: Vec<String>,
    /// Nodes that have successfully received the update
    pub confirmed_nodes: HashSet<String>,
    /// Nodes that failed to receive the update
    pub failed_nodes: HashMap<String, String>,
    /// Nodes currently being processed
    pub pending_nodes: HashSet<String>,
    /// Current retry attempt
    pub retry_attempt: u32,
    /// Operation start time
    pub started_at: DateTime<Utc>,
    /// Operation completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Current status
    pub status: KeyPropagationStatus,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

impl PropagationOperation {
    /// Create a new propagation operation
    pub fn new(event: KeyRotationEvent, target_nodes: Vec<String>) -> Self {
        let operation_id = Uuid::new_v4();
        let now = Utc::now();

        Self {
            operation_id,
            event,
            target_nodes: target_nodes.clone(),
            confirmed_nodes: HashSet::new(),
            failed_nodes: HashMap::new(),
            pending_nodes: target_nodes.into_iter().collect(),
            retry_attempt: 0,
            started_at: now,
            completed_at: None,
            status: KeyPropagationStatus::Pending,
            last_activity: now,
        }
    }

    /// Check if the operation is complete
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            KeyPropagationStatus::Completed
                | KeyPropagationStatus::Failed
                | KeyPropagationStatus::RolledBack
        )
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        let total = self.target_nodes.len() as f64;
        if total == 0.0 {
            return 1.0;
        }
        self.confirmed_nodes.len() as f64 / total
    }

    /// Update operation status based on current state
    pub fn update_status(&mut self) {
        if self.pending_nodes.is_empty() {
            if self.failed_nodes.is_empty() {
                self.status = KeyPropagationStatus::Completed;
            } else if self.confirmed_nodes.is_empty() {
                self.status = KeyPropagationStatus::Failed;
            } else {
                self.status = KeyPropagationStatus::PartialFailure;
            }
            self.completed_at = Some(Utc::now());
        } else if !self.confirmed_nodes.is_empty() || !self.failed_nodes.is_empty() {
            self.status = KeyPropagationStatus::InProgress;
        }

        self.last_activity = Utc::now();
    }
}

/// Conflict resolution strategy for concurrent key rotations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictResolutionStrategy {
    /// Last writer wins based on timestamp
    LastWriterWins,
    /// Fail if conflicts are detected
    FailOnConflict,
    /// Manual resolution required
    ManualResolution,
}

/// Network key propagation manager
pub struct KeyPropagationManager {
    /// Configuration
    config: KeyPropagationConfig,
    /// Network core for peer communication
    network: Arc<RwLock<NetworkCore>>,
    /// Active propagation operations
    operations: Arc<RwLock<HashMap<Uuid, PropagationOperation>>>,
    /// Transport for reliable event delivery
    transport: Arc<Mutex<dyn EventTransport + Send + Sync>>,
    /// Conflict resolution strategy
    conflict_strategy: ConflictResolutionStrategy,
    /// Background task handle
    background_task: Option<tokio::task::JoinHandle<()>>,
}

impl KeyPropagationManager {
    /// Create a new key propagation manager
    pub fn new(
        config: KeyPropagationConfig,
        network: Arc<RwLock<NetworkCore>>,
        transport: Arc<Mutex<dyn EventTransport + Send + Sync>>,
    ) -> Self {
        Self {
            config,
            network,
            operations: Arc::new(RwLock::new(HashMap::new())),
            transport,
            conflict_strategy: ConflictResolutionStrategy::LastWriterWins,
            background_task: None,
        }
    }

    /// Set conflict resolution strategy
    pub fn with_conflict_strategy(mut self, strategy: ConflictResolutionStrategy) -> Self {
        self.conflict_strategy = strategy;
        self
    }

    /// Start the propagation manager
    pub async fn start(&mut self) -> NetworkResult<()> {
        // Start background task for operation monitoring
        let operations = Arc::clone(&self.operations);
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            Self::background_monitor(operations, config).await;
        });

        self.background_task = Some(handle);
        info!("Key propagation manager started");
        Ok(())
    }

    /// Stop the propagation manager
    pub async fn stop(&mut self) {
        if let Some(handle) = self.background_task.take() {
            handle.abort();
        }
        info!("Key propagation manager stopped");
    }

    /// Propagate a key rotation event to the network
    pub async fn propagate_key_rotation(&self, event: KeyRotationEvent) -> NetworkResult<Uuid> {
        let target_nodes = self.determine_target_nodes(&event).await?;

        if target_nodes.is_empty() {
            warn!("No target nodes found for key rotation propagation");
            return Err(NetworkError::ConfigurationError(
                "No target nodes available".to_string(),
            ));
        }

        // Check for conflicts if enabled
        if self.config.enable_conflict_resolution {
            self.check_for_conflicts(&event).await?;
        }

        let mut operation = PropagationOperation::new(event.clone(), target_nodes);
        operation.status = KeyPropagationStatus::InProgress;

        let operation_id = operation.operation_id;

        // Store the operation
        {
            let mut operations = self.operations.write().await;
            operations.insert(operation_id, operation);
        }

        info!(
            "Starting key rotation propagation for operation {}",
            operation_id
        );

        // Start propagation
        self.execute_propagation(operation_id).await?;

        Ok(operation_id)
    }

    /// Get the status of a propagation operation
    pub async fn get_operation_status(&self, operation_id: &Uuid) -> Option<PropagationOperation> {
        let operations = self.operations.read().await;
        operations.get(operation_id).cloned()
    }

    /// Wait for a propagation operation to complete
    pub async fn wait_for_completion(
        &self,
        operation_id: &Uuid,
        timeout: Duration,
    ) -> NetworkResult<PropagationOperation> {
        let start = Instant::now();

        loop {
            if start.elapsed() > timeout {
                return Err(NetworkError::Timeout(
                    "Propagation operation timeout".to_string(),
                ));
            }

            if let Some(operation) = self.get_operation_status(operation_id).await {
                if operation.is_complete() {
                    return Ok(operation);
                }
            } else {
                return Err(NetworkError::OperationNotFound(format!(
                    "Operation {} not found",
                    operation_id
                )));
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get all active propagation operations
    pub async fn get_active_operations(&self) -> Vec<PropagationOperation> {
        let operations = self.operations.read().await;
        operations
            .values()
            .filter(|op| !op.is_complete())
            .cloned()
            .collect()
    }

    /// Cancel a propagation operation
    pub async fn cancel_operation(&self, operation_id: &Uuid) -> NetworkResult<()> {
        let mut operations = self.operations.write().await;

        if let Some(mut operation) = operations.remove(operation_id) {
            operation.status = KeyPropagationStatus::Failed;
            operation.completed_at = Some(Utc::now());
            operations.insert(*operation_id, operation);
            info!("Cancelled propagation operation {}", operation_id);
            Ok(())
        } else {
            Err(NetworkError::OperationNotFound(format!(
                "Operation {} not found",
                operation_id
            )))
        }
    }

    /// Determine target nodes for propagation
    async fn determine_target_nodes(&self, event: &KeyRotationEvent) -> NetworkResult<Vec<String>> {
        if !event.target_nodes.is_empty() {
            // Use explicitly specified target nodes
            Ok(event.target_nodes.clone())
        } else {
            // Use all known nodes in the network
            let network = self.network.read().await;
            let known_peers = network.known_peers();

            // Convert peer IDs to node IDs
            let mut node_ids = Vec::new();
            for peer_id in known_peers {
                if let Some(node_id) = network.get_node_id_for_peer(peer_id) {
                    node_ids.push(node_id);
                } else {
                    // Use peer ID as node ID if no mapping exists
                    node_ids.push(peer_id.to_string());
                }
            }

            Ok(node_ids)
        }
    }

    /// Check for conflicts with other ongoing rotations
    async fn check_for_conflicts(&self, event: &KeyRotationEvent) -> NetworkResult<()> {
        let operations = self.operations.read().await;

        for operation in operations.values() {
            if !operation.is_complete() {
                // Check if this is for the same key
                if operation.event.old_key_id == event.old_key_id
                    || operation.event.new_key_id == event.new_key_id
                {
                    match self.conflict_strategy {
                        ConflictResolutionStrategy::FailOnConflict => {
                            return Err(NetworkError::ConflictDetected(format!(
                                "Concurrent key rotation detected for key: {:?}",
                                event.old_key_id
                            )));
                        }
                        ConflictResolutionStrategy::LastWriterWins => {
                            // Check timestamps to determine winner
                            if event.base.timestamp <= operation.event.base.timestamp {
                                return Err(NetworkError::ConflictDetected(
                                    "Later key rotation already in progress".to_string(),
                                ));
                            }
                        }
                        ConflictResolutionStrategy::ManualResolution => {
                            warn!("Manual conflict resolution required for key rotation");
                            // Continue but log the conflict for manual intervention
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute propagation for an operation
    async fn execute_propagation(&self, operation_id: Uuid) -> NetworkResult<()> {
        let batch_size = self.config.batch_size;

        loop {
            let (pending_nodes, retry_attempt) = {
                let operations = self.operations.read().await;
                if let Some(operation) = operations.get(&operation_id) {
                    if operation.is_complete() {
                        break;
                    }
                    (operation.pending_nodes.clone(), operation.retry_attempt)
                } else {
                    return Err(NetworkError::OperationNotFound(format!(
                        "Operation {} not found",
                        operation_id
                    )));
                }
            };

            if pending_nodes.is_empty() {
                // Update final status
                let mut operations = self.operations.write().await;
                if let Some(operation) = operations.get_mut(&operation_id) {
                    operation.update_status();
                }
                break;
            }

            // Process nodes in batches
            let nodes_batch: Vec<String> = pending_nodes.into_iter().take(batch_size).collect();
            debug!("Propagating to batch of {} nodes", nodes_batch.len());

            let results = self.propagate_to_batch(operation_id, &nodes_batch).await?;

            // Update operation status based on results
            {
                let mut operations = self.operations.write().await;
                if let Some(operation) = operations.get_mut(&operation_id) {
                    for (node_id, result) in results {
                        operation.pending_nodes.remove(&node_id);

                        match result {
                            Ok(_) => {
                                operation.confirmed_nodes.insert(node_id.clone());
                                info!("Successfully propagated to node: {}", node_id);
                            }
                            Err(error) => {
                                operation.failed_nodes.insert(node_id.clone(), error);
                                warn!(
                                    "Failed to propagate to node {}: {}",
                                    node_id,
                                    operation.failed_nodes.get(&node_id).unwrap()
                                );
                            }
                        }
                    }

                    operation.update_status();
                }
            }

            // Check if we should retry failed nodes
            if retry_attempt < self.config.max_retry_attempts {
                let retry_delay = Duration::from_millis(self.config.retry_delay_ms);
                tokio::time::sleep(retry_delay).await;

                // Add failed nodes back to pending for retry
                let mut operations = self.operations.write().await;
                if let Some(operation) = operations.get_mut(&operation_id) {
                    let failed_nodes: Vec<String> =
                        operation.failed_nodes.keys().cloned().collect();
                    if !failed_nodes.is_empty()
                        && operation.retry_attempt < self.config.max_retry_attempts
                    {
                        for node_id in failed_nodes {
                            operation.failed_nodes.remove(&node_id);
                            operation.pending_nodes.insert(node_id);
                        }
                        operation.retry_attempt += 1;
                        info!(
                            "Retrying propagation for operation {} (attempt {})",
                            operation_id, operation.retry_attempt
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Propagate to a batch of nodes
    async fn propagate_to_batch(
        &self,
        operation_id: Uuid,
        node_ids: &[String],
    ) -> NetworkResult<HashMap<String, Result<TransportResult, String>>> {
        let mut results = HashMap::new();

        // Get the operation event
        let event = {
            let operations = self.operations.read().await;
            operations
                .get(&operation_id)
                .ok_or_else(|| {
                    NetworkError::OperationNotFound(format!("Operation {} not found", operation_id))
                })?
                .event
                .clone()
        };

        // Create event envelope
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
            event: SecurityEvent::KeyRotation(event),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert(
                    "operation_id".to_string(),
                    serde_json::Value::String(operation_id.to_string()),
                );
                metadata.insert(
                    "propagation_batch".to_string(),
                    serde_json::Value::Bool(true),
                );
                metadata
            },
            envelope_timestamp: Utc::now(),
            envelope_id: Uuid::new_v4(),
        };

        // Send to each node with timeout
        let timeout_duration = Duration::from_secs(self.config.propagation_timeout_secs);
        let transport = self.transport.lock().await;

        for node_id in node_ids {
            let result =
                tokio::time::timeout(timeout_duration, transport.send_event(envelope.clone()))
                    .await;

            match result {
                Ok(Ok(transport_result)) => {
                    results.insert(node_id.clone(), Ok(transport_result));
                }
                Ok(Err(transport_error)) => {
                    results.insert(node_id.clone(), Err(transport_error.to_string()));
                }
                Err(_) => {
                    results.insert(
                        node_id.clone(),
                        Err("Timeout during propagation".to_string()),
                    );
                }
            }
        }

        Ok(results)
    }

    /// Background monitoring task
    async fn background_monitor(
        operations: Arc<RwLock<HashMap<Uuid, PropagationOperation>>>,
        config: KeyPropagationConfig,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(30));

        loop {
            interval.tick().await;

            // Clean up completed operations older than 1 hour
            let cutoff = Utc::now() - chrono::Duration::hours(1);
            let mut operations_guard = operations.write().await;

            let before_count = operations_guard.len();
            operations_guard.retain(|_, operation| {
                !operation.is_complete() || operation.completed_at.is_none_or(|t| t > cutoff)
            });
            let after_count = operations_guard.len();

            if before_count != after_count {
                info!(
                    "Cleaned up {} completed propagation operations",
                    before_count - after_count
                );
            }

            // Check for stale operations that need intervention
            let stale_cutoff =
                Utc::now() - chrono::Duration::seconds(config.propagation_timeout_secs as i64 * 2);
            for (operation_id, operation) in operations_guard.iter_mut() {
                if !operation.is_complete() && operation.last_activity < stale_cutoff {
                    warn!(
                        "Marking stale propagation operation {} as failed",
                        operation_id
                    );
                    operation.status = KeyPropagationStatus::Failed;
                    operation.completed_at = Some(Utc::now());
                }
            }
        }
    }
}

impl Drop for KeyPropagationManager {
    fn drop(&mut self) {
        if let Some(handle) = self.background_task.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{
        EventSeverity, OperationResult, PlatformSource, SecurityEventCategory, VerificationEvent,
    };
    use crate::events::KeyRotationEventType;
    use crate::events::transport::{InMemoryTransport, TransportConfig};
    use crate::network::NetworkConfig;

    #[tokio::test]
    async fn test_propagation_operation_lifecycle() {
        let base_event = VerificationEvent {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            category: SecurityEventCategory::KeyRotation,
            severity: EventSeverity::Info,
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

        let target_nodes = vec!["node1".to_string(), "node2".to_string()];
        let mut operation = PropagationOperation::new(key_rotation_event, target_nodes);

        assert_eq!(operation.status, KeyPropagationStatus::Pending);
        assert_eq!(operation.success_rate(), 0.0);
        assert!(!operation.is_complete());

        // Simulate successful propagation to one node
        operation.pending_nodes.remove("node1");
        operation.confirmed_nodes.insert("node1".to_string());
        operation.update_status();

        assert_eq!(operation.status, KeyPropagationStatus::InProgress);
        assert_eq!(operation.success_rate(), 0.5);

        // Simulate successful propagation to second node
        operation.pending_nodes.remove("node2");
        operation.confirmed_nodes.insert("node2".to_string());
        operation.update_status();

        assert_eq!(operation.status, KeyPropagationStatus::Completed);
        assert_eq!(operation.success_rate(), 1.0);
        assert!(operation.is_complete());
    }

    #[tokio::test]
    async fn test_key_propagation_manager() {
        let config = KeyPropagationConfig::default();
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

        let mut manager = KeyPropagationManager::new(config, network, transport);
        manager.start().await.unwrap();

        // Test that manager starts correctly
        assert!(manager.background_task.is_some());

        manager.stop().await;
    }
}
