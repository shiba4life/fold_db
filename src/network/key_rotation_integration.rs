//! Integration module for key rotation network propagation
//!
//! This module demonstrates how to integrate key rotation events with network propagation,
//! cache invalidation, and consistency guarantees across a distributed DataFold network.

use crate::crypto::key_rotation::{
    KeyRotationRequest, RotationContext, RotationValidationResult,
};
use crate::datafold_node::key_cache_manager::{KeyCacheConfig, KeyCacheManager};
use crate::events::transport::{EventTransport, InMemoryTransport, PlatformInfo, TransportConfig};
use crate::events::{
    EventSeverity, KeyPropagationStatus, KeyRotationEvent, KeyRotationEventType,
    KeyRotationHandlerConfig, KeyRotationPropagationHandler, OperationResult, PlatformSource,
    SecurityEvent, SecurityEventCategory, VerificationEvent, VerificationEventBus,
};
use crate::network::{KeyPropagationConfig, KeyPropagationManager, NetworkCore};
use chrono::Utc;
use log::{info, warn};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Configuration for integrated key rotation network propagation
#[derive(Debug, Clone)]
pub struct KeyRotationNetworkConfig {
    /// Event bus configuration
    pub event_bus_config: crate::events::VerificationBusConfig,
    /// Network propagation configuration
    pub propagation_config: KeyPropagationConfig,
    /// Handler configuration
    pub handler_config: KeyRotationHandlerConfig,
    /// Cache configuration
    pub cache_config: KeyCacheConfig,
    /// Enable comprehensive logging
    pub enable_logging: bool,
}

impl Default for KeyRotationNetworkConfig {
    fn default() -> Self {
        Self {
            event_bus_config: crate::events::VerificationBusConfig::default(),
            propagation_config: KeyPropagationConfig::default(),
            handler_config: KeyRotationHandlerConfig::default(),
            cache_config: KeyCacheConfig::default(),
            enable_logging: true,
        }
    }
}

/// Integrated key rotation network propagation system
pub struct KeyRotationNetworkIntegration {
    /// Event bus for coordinating events
    event_bus: Arc<RwLock<VerificationEventBus>>,
    /// Network core for peer communication
    network: Arc<RwLock<NetworkCore>>,
    /// Propagation manager for network consistency
    propagation_manager: Arc<Mutex<KeyPropagationManager>>,
    /// Cache manager for invalidation
    cache_manager: Arc<Mutex<KeyCacheManager>>,
    /// Event transport
    transport: Arc<Mutex<dyn crate::events::transport::EventTransport + Send + Sync>>,
    /// Configuration
    config: KeyRotationNetworkConfig,
    /// Active rotations tracking
    active_rotations: Arc<RwLock<HashMap<Uuid, RotationContext>>>,
}

impl KeyRotationNetworkIntegration {
    /// Create a new integrated key rotation network system
    pub async fn new(
        config: KeyRotationNetworkConfig,
        network_config: crate::network::NetworkConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Initialize network core
        let network = Arc::new(RwLock::new(NetworkCore::new(network_config).await?));

        // Initialize event transport
        let transport_config = TransportConfig::default();
        let platform_info = PlatformInfo {
            platform_type: "datafold-node".to_string(),
            version: "1.0.0".to_string(),
            host: None,
            instance_id: Some(Uuid::new_v4().to_string()),
            metadata: HashMap::new(),
        };
        let transport: Arc<Mutex<dyn EventTransport + Send + Sync>> = Arc::new(Mutex::new(
            InMemoryTransport::new(transport_config, platform_info),
        ));

        // Initialize event bus
        let event_bus = Arc::new(RwLock::new(VerificationEventBus::new(
            config.event_bus_config.clone(),
        )));

        // Initialize propagation manager
        let propagation_manager = Arc::new(Mutex::new(KeyPropagationManager::new(
            config.propagation_config.clone(),
            Arc::clone(&network),
            Arc::clone(&transport),
        )));

        // Initialize cache manager
        let mut cache_manager = KeyCacheManager::new(config.cache_config.clone());
        cache_manager.start().await?;
        let cache_manager = Arc::new(Mutex::new(cache_manager));

        Ok(Self {
            event_bus,
            network,
            propagation_manager,
            cache_manager,
            transport,
            config,
            active_rotations: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Start the integrated system
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Start event bus
        {
            let mut event_bus = self.event_bus.write().await;
            event_bus.start().await?;
        }

        // Start network
        {
            let mut network = self.network.write().await;
            network.run("/ip4/0.0.0.0/tcp/0").await?;
        }

        // Start propagation manager
        {
            let mut propagation_manager = self.propagation_manager.lock().await;
            propagation_manager.start().await?;
        }

        // Register event handlers
        self.register_event_handlers().await?;

        info!("Key rotation network integration started");
        Ok(())
    }

    /// Stop the integrated system
    pub async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Stop propagation manager
        {
            let mut propagation_manager = self.propagation_manager.lock().await;
            propagation_manager.stop().await;
        }

        // Stop network
        {
            let mut network = self.network.write().await;
            network.stop();
        }

        // Stop cache manager
        {
            let mut cache_manager = self.cache_manager.lock().await;
            cache_manager.stop().await;
        }

        // Stop event bus
        {
            let mut event_bus = self.event_bus.write().await;
            event_bus.stop().await;
        }

        info!("Key rotation network integration stopped");
        Ok(())
    }

    /// Process a key rotation with full network propagation
    pub async fn process_key_rotation(
        &self,
        request: KeyRotationRequest,
        actor: Option<String>,
    ) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
        let correlation_id = Uuid::new_v4();

        if self.config.enable_logging {
            info!(
                "Starting key rotation process for correlation ID: {}",
                correlation_id
            );
        }

        // Create rotation context
        let validation = RotationValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        let context = RotationContext::new(request.clone(), validation, actor.clone());

        // Store active rotation
        {
            let mut active_rotations = self.active_rotations.write().await;
            active_rotations.insert(correlation_id, context.clone());
        }

        // Step 1: Publish rotation started event
        let started_event = self.create_key_rotation_event(
            &request,
            KeyRotationEventType::RotationStarted,
            KeyPropagationStatus::Pending,
            correlation_id,
            actor.clone(),
        )?;

        self.publish_event(SecurityEvent::KeyRotation(started_event))
            .await?;

        // Step 2: Simulate the actual key rotation (in real implementation, this would call db operations)
        tokio::time::sleep(Duration::from_millis(100)).await; // Simulate processing time

        // Step 3: Publish rotation completed event (this triggers propagation)
        let completed_event = self.create_key_rotation_event(
            &request,
            KeyRotationEventType::RotationCompleted,
            KeyPropagationStatus::Pending,
            correlation_id,
            actor,
        )?;

        self.publish_event(SecurityEvent::KeyRotation(completed_event))
            .await?;

        if self.config.enable_logging {
            info!(
                "Key rotation completed for correlation ID: {}",
                correlation_id
            );
        }

        Ok(correlation_id)
    }

    /// Wait for propagation to complete
    pub async fn wait_for_propagation_completion(
        &self,
        correlation_id: Uuid,
        timeout: Duration,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > timeout {
                warn!(
                    "Timeout waiting for propagation completion: {}",
                    correlation_id
                );
                return Ok(false);
            }

            // Check if propagation is complete by looking at active operations
            let propagation_manager = self.propagation_manager.lock().await;
            let active_ops = propagation_manager.get_active_operations().await;

            let has_active = active_ops.iter().any(|op| {
                op.event
                    .operation_id
                    .as_ref()
                    .and_then(|id| Uuid::parse_str(id).ok()) == Some(correlation_id)
            });

            if !has_active {
                info!(
                    "Propagation completed for correlation ID: {}",
                    correlation_id
                );
                return Ok(true);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Get propagation status for a rotation
    pub async fn get_rotation_status(&self, correlation_id: &Uuid) -> Option<RotationContext> {
        let active_rotations = self.active_rotations.read().await;
        active_rotations.get(correlation_id).cloned()
    }

    /// Get cache metrics
    pub async fn get_cache_metrics(&self) -> crate::datafold_node::key_cache_manager::CacheMetrics {
        let cache_manager = self.cache_manager.lock().await;
        cache_manager.get_metrics().await
    }

    /// Get network statistics
    pub async fn get_network_statistics(
        &self,
    ) -> crate::events::verification_bus::EventBusStatistics {
        let event_bus = self.event_bus.read().await;
        event_bus.get_statistics().await
    }

    /// Add a network node
    pub async fn add_network_node(&self, node_id: String, address: String) {
        let mut network = self.network.write().await;
        let peer_id = libp2p::PeerId::random(); // In real implementation, this would be the actual peer ID
        network.register_node_id(&node_id, peer_id);
        network.register_node_address(&node_id, address);
        network.add_known_peer(peer_id);

        info!("Added network node: {} ({})", node_id, peer_id);
    }

    /// Register event handlers with the event bus
    async fn register_event_handlers(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let _event_bus = self.event_bus.clone();

        // Register propagation handler
        let propagation_handler = KeyRotationPropagationHandler::new(
            self.config.handler_config.clone(),
            Arc::clone(&self.network),
            Arc::clone(&self.transport),
            Arc::clone(&self.event_bus),
        );

        {
            let event_bus = self.event_bus.read().await;
            event_bus
                .register_handler(Box::new(propagation_handler))
                .await?;
        }

        // TODO: Register cache invalidation handler
        // Note: KeyCacheManager doesn't implement EventHandler
        // This may need to be implemented if cache invalidation events are required

        info!("Registered event handlers for key rotation");
        Ok(())
    }

    /// Create a key rotation event
    fn create_key_rotation_event(
        &self,
        request: &KeyRotationRequest,
        rotation_type: KeyRotationEventType,
        propagation_status: KeyPropagationStatus,
        correlation_id: Uuid,
        actor: Option<String>,
    ) -> Result<KeyRotationEvent, Box<dyn std::error::Error + Send + Sync>> {
        let base_event =
            VerificationEvent {
                event_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                category: SecurityEventCategory::KeyRotation,
                severity: match rotation_type {
                    KeyRotationEventType::RotationFailed
                    | KeyRotationEventType::PropagationFailed => EventSeverity::Error,
                    KeyRotationEventType::RotationStarted
                    | KeyRotationEventType::PropagationStarted => EventSeverity::Warning,
                    _ => EventSeverity::Info,
                },
                platform: PlatformSource::DataFoldNode,
                component: "key_rotation_network".to_string(),
                operation: "network_propagation".to_string(),
                actor,
                result: match rotation_type {
                    KeyRotationEventType::RotationCompleted
                    | KeyRotationEventType::PropagationCompleted => OperationResult::Success,
                    KeyRotationEventType::RotationFailed
                    | KeyRotationEventType::PropagationFailed => OperationResult::Failure {
                        error_type: "RotationError".to_string(),
                        error_message: "Key rotation failed".to_string(),
                        error_code: Some("KEY_ROTATION_FAILED".to_string()),
                    },
                    _ => OperationResult::InProgress,
                },
                duration: None,
                metadata: HashMap::new(),
                correlation_id: Some(correlation_id),
                trace_id: None,
                session_id: None,
                environment: Some("production".to_string()),
            };

        let old_key_id = hex::encode(&request.old_public_key.to_bytes()[..8]);
        let new_key_id = hex::encode(&request.new_public_key.to_bytes()[..8]);

        Ok(KeyRotationEvent {
            base: base_event,
            rotation_type,
            user_id: request.client_id.clone(),
            old_key_id: Some(old_key_id),
            new_key_id: Some(new_key_id),
            rotation_reason: format!("{:?}", request.reason),
            operation_id: Some(correlation_id.to_string()),
            target_nodes: Vec::new(), // Will be populated by propagation handler
            propagation_status,
            affected_associations: None,
            rotation_metadata: request
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect(),
        })
    }

    /// Publish an event to the event bus
    async fn publish_event(
        &self,
        event: SecurityEvent,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let event_bus = self.event_bus.read().await;
        event_bus.publish_event(event).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{ed25519::generate_master_keypair, RotationReason};
    use crate::network::NetworkConfig;

    #[tokio::test]
    async fn test_key_rotation_network_integration() {
        let config = KeyRotationNetworkConfig::default();
        let network_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/0");

        let mut integration = KeyRotationNetworkIntegration::new(config, network_config)
            .await
            .unwrap();
        integration.start().await.unwrap();

        // Add some network nodes
        integration
            .add_network_node("node1".to_string(), "127.0.0.1:8001".to_string())
            .await;
        integration
            .add_network_node("node2".to_string(), "127.0.0.1:8002".to_string())
            .await;

        // Create a test key rotation request
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();

        let request = KeyRotationRequest::new(
            &old_keypair.private_key(),
            new_keypair.public_key().clone(),
            RotationReason::Scheduled,
            Some("test_client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        // Process the rotation
        let correlation_id = integration
            .process_key_rotation(request, Some("test_actor".to_string()))
            .await
            .unwrap();

        // Wait for propagation to complete (with timeout)
        let completed = integration
            .wait_for_propagation_completion(correlation_id, Duration::from_secs(10))
            .await
            .unwrap();

        assert!(completed, "Propagation should complete within timeout");

        // Check metrics
        let _cache_metrics = integration.get_cache_metrics().await;
        let network_stats = integration.get_network_statistics().await;

        assert!(
            network_stats.total_events > 0,
            "Should have processed events"
        );

        integration.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_concurrent_rotations() {
        let config = KeyRotationNetworkConfig::default();
        let network_config = NetworkConfig::new("/ip4/127.0.0.1/tcp/0");

        let mut integration = KeyRotationNetworkIntegration::new(config, network_config)
            .await
            .unwrap();
        integration.start().await.unwrap();

        // Create multiple rotation requests
        let mut correlation_ids = Vec::new();

        for i in 0..3 {
            let old_keypair = generate_master_keypair().unwrap();
            let new_keypair = generate_master_keypair().unwrap();

            let request = KeyRotationRequest::new(
                &old_keypair.private_key(),
                new_keypair.public_key().clone(),
                RotationReason::Scheduled,
                Some(format!("test_client_{}", i)),
                HashMap::new(),
            )
            .unwrap();

            let correlation_id = integration
                .process_key_rotation(request, Some(format!("test_actor_{}", i)))
                .await
                .unwrap();

            correlation_ids.push(correlation_id);
        }

        // Wait for all to complete
        for correlation_id in correlation_ids {
            let completed = integration
                .wait_for_propagation_completion(correlation_id, Duration::from_secs(10))
                .await
                .unwrap();

            assert!(completed, "All rotations should complete");
        }

        integration.stop().await.unwrap();
    }
}
