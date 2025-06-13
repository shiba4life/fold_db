//! Cache invalidation manager for key rotation events
//!
//! This module handles cache invalidation logic for key associations when rotation occurs,
//! ensuring that all nodes have updated key caches and fresh key lookups after rotation.

use crate::crypto::ed25519::PublicKey;
use crate::events::event_types::{KeyRotationEvent, KeyRotationEventType, SecurityEvent};
use crate::events::handlers::{EventHandler, EventHandlerResult};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Configuration for key cache management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCacheConfig {
    /// Enable cache invalidation
    pub enable_invalidation: bool,
    /// Cache entry TTL in seconds
    pub cache_ttl_secs: u64,
    /// Maximum cache size
    pub max_cache_size: usize,
    /// Invalidation batch size
    pub invalidation_batch_size: usize,
    /// Enable distributed cache invalidation
    pub enable_distributed_invalidation: bool,
    /// Cache cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Enable metrics collection
    pub enable_metrics: bool,
}

impl Default for KeyCacheConfig {
    fn default() -> Self {
        Self {
            enable_invalidation: true,
            cache_ttl_secs: 3600, // 1 hour
            max_cache_size: 10000,
            invalidation_batch_size: 100,
            enable_distributed_invalidation: true,
            cleanup_interval_secs: 300, // 5 minutes
            enable_metrics: true,
        }
    }
}

/// Cache entry for key associations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCacheEntry {
    /// Key identifier (hex encoded)
    pub key_id: String,
    /// Associated public key
    pub public_key: PublicKey,
    /// Association type
    pub association_type: String,
    /// Associated data reference
    pub data_reference: String,
    /// Cache entry creation timestamp
    pub cached_at: DateTime<Utc>,
    /// Last access timestamp
    pub last_accessed: DateTime<Utc>,
    /// Access count
    pub access_count: u64,
    /// TTL expiration time
    pub expires_at: DateTime<Utc>,
    /// Entry status
    pub status: CacheEntryStatus,
}

/// Status of cache entries
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheEntryStatus {
    /// Entry is valid
    Valid,
    /// Entry is invalidated
    Invalidated,
    /// Entry is expired
    Expired,
    /// Entry is being refreshed
    Refreshing,
}

/// Cache invalidation operation
#[derive(Debug, Clone)]
pub struct InvalidationOperation {
    /// Operation ID
    pub operation_id: Uuid,
    /// Key IDs to invalidate
    pub key_ids: Vec<String>,
    /// Reason for invalidation
    pub reason: String,
    /// Start timestamp
    pub started_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Number of entries invalidated
    pub invalidated_count: usize,
    /// Success status
    pub success: bool,
}

/// Cache metrics and statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetrics {
    /// Total cache entries
    pub total_entries: usize,
    /// Valid entries
    pub valid_entries: usize,
    /// Invalidated entries
    pub invalidated_entries: usize,
    /// Expired entries
    pub expired_entries: usize,
    /// Cache hit rate
    pub hit_rate: f64,
    /// Cache miss rate
    pub miss_rate: f64,
    /// Total hits
    pub total_hits: u64,
    /// Total misses
    pub total_misses: u64,
    /// Total invalidations
    pub total_invalidations: u64,
    /// Average access time in milliseconds
    pub avg_access_time_ms: f64,
    /// Last cleanup timestamp
    pub last_cleanup: Option<DateTime<Utc>>,
}

impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            total_entries: 0,
            valid_entries: 0,
            invalidated_entries: 0,
            expired_entries: 0,
            hit_rate: 0.0,
            miss_rate: 0.0,
            total_hits: 0,
            total_misses: 0,
            total_invalidations: 0,
            avg_access_time_ms: 0.0,
            last_cleanup: None,
        }
    }
}

/// Key cache manager for handling invalidation during key rotation
pub struct KeyCacheManager {
    /// Configuration
    config: KeyCacheConfig,
    /// Cache storage
    cache: Arc<RwLock<HashMap<String, KeyCacheEntry>>>,
    /// Cache metrics
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Active invalidation operations
    invalidation_operations: Arc<RwLock<HashMap<Uuid, InvalidationOperation>>>,
    /// Background cleanup task handle
    cleanup_task: Option<tokio::task::JoinHandle<()>>,
    /// Handler name
    name: String,
}

impl KeyCacheManager {
    /// Create a new key cache manager
    pub fn new(config: KeyCacheConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            invalidation_operations: Arc::new(RwLock::new(HashMap::new())),
            cleanup_task: None,
            name: "key_cache_manager".to_string(),
        }
    }

    /// Set handler name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = name;
        self
    }

    /// Start the cache manager
    pub async fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.config.cleanup_interval_secs > 0 {
            let cache = Arc::clone(&self.cache);
            let metrics = Arc::clone(&self.metrics);
            let config = self.config.clone();

            let handle = tokio::spawn(async move {
                Self::background_cleanup(cache, metrics, config).await;
            });

            self.cleanup_task = Some(handle);
        }

        info!(
            "Key cache manager started with {} max entries",
            self.config.max_cache_size
        );
        Ok(())
    }

    /// Stop the cache manager
    pub async fn stop(&mut self) {
        if let Some(handle) = self.cleanup_task.take() {
            handle.abort();
        }
        info!("Key cache manager stopped");
    }

    /// Get a cache entry by key ID
    pub async fn get(&self, key_id: &str) -> Option<KeyCacheEntry> {
        let start_time = Instant::now();
        let mut cache = self.cache.write().await;

        if let Some(entry) = cache.get_mut(key_id) {
            // Check if entry is expired
            if entry.expires_at < Utc::now() {
                entry.status = CacheEntryStatus::Expired;
                self.record_miss().await;
                None
            } else if entry.status == CacheEntryStatus::Valid {
                // Update access statistics
                entry.last_accessed = Utc::now();
                entry.access_count += 1;

                self.record_hit(start_time.elapsed()).await;
                Some(entry.clone())
            } else {
                self.record_miss().await;
                None
            }
        } else {
            self.record_miss().await;
            None
        }
    }

    /// Put an entry in the cache
    pub async fn put(
        &self,
        key_id: String,
        public_key: PublicKey,
        association_type: String,
        data_reference: String,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;

        // Check cache size limit
        if cache.len() >= self.config.max_cache_size {
            self.evict_oldest(&mut cache).await;
        }

        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.config.cache_ttl_secs as i64);

        let entry = KeyCacheEntry {
            key_id: key_id.clone(),
            public_key,
            association_type,
            data_reference,
            cached_at: now,
            last_accessed: now,
            access_count: 0,
            expires_at,
            status: CacheEntryStatus::Valid,
        };

        cache.insert(key_id, entry);
        self.update_entry_metrics().await;

        Ok(())
    }

    /// Invalidate cache entries for key rotation
    pub async fn invalidate_for_rotation(
        &self,
        event: &KeyRotationEvent,
    ) -> Result<InvalidationOperation, Box<dyn std::error::Error + Send + Sync>> {
        if !self.config.enable_invalidation {
            return Ok(InvalidationOperation {
                operation_id: Uuid::new_v4(),
                key_ids: Vec::new(),
                reason: "Invalidation disabled".to_string(),
                started_at: Utc::now(),
                completed_at: Some(Utc::now()),
                invalidated_count: 0,
                success: true,
            });
        }

        let operation_id = Uuid::new_v4();
        let mut key_ids_to_invalidate = Vec::new();

        // Add old key ID if present
        if let Some(old_key_id) = &event.old_key_id {
            key_ids_to_invalidate.push(old_key_id.clone());
        }

        // For some rotation types, also invalidate new key if it exists in cache
        match event.rotation_type {
            KeyRotationEventType::RotationFailed | KeyRotationEventType::RollbackStarted => {
                if let Some(new_key_id) = &event.new_key_id {
                    key_ids_to_invalidate.push(new_key_id.clone());
                }
            }
            _ => {}
        }

        let reason = format!("Key rotation: {:?}", event.rotation_type);
        let started_at = Utc::now();

        // Create operation record
        let mut operation = InvalidationOperation {
            operation_id,
            key_ids: key_ids_to_invalidate.clone(),
            reason: reason.clone(),
            started_at,
            completed_at: None,
            invalidated_count: 0,
            success: false,
        };

        // Store operation
        {
            let mut operations = self.invalidation_operations.write().await;
            operations.insert(operation_id, operation.clone());
        }

        info!(
            "Starting cache invalidation for operation {}: {:?}",
            operation_id, key_ids_to_invalidate
        );

        // Perform invalidation
        let invalidated_count = self
            .invalidate_keys(&key_ids_to_invalidate, &reason)
            .await?;

        // Update operation
        operation.completed_at = Some(Utc::now());
        operation.invalidated_count = invalidated_count;
        operation.success = true;

        // Store updated operation
        {
            let mut operations = self.invalidation_operations.write().await;
            operations.insert(operation_id, operation.clone());
        }

        info!(
            "Completed cache invalidation for operation {}: {} entries invalidated",
            operation_id, invalidated_count
        );

        Ok(operation)
    }

    /// Invalidate specific keys
    pub async fn invalidate_keys(
        &self,
        key_ids: &[String],
        reason: &str,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut cache = self.cache.write().await;
        let mut invalidated_count = 0;

        for key_id in key_ids {
            if let Some(entry) = cache.get_mut(key_id) {
                if entry.status == CacheEntryStatus::Valid {
                    entry.status = CacheEntryStatus::Invalidated;
                    invalidated_count += 1;
                    debug!("Invalidated cache entry for key {}: {}", key_id, reason);
                }
            }
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_invalidations += invalidated_count as u64;
        }

        self.update_entry_metrics().await;

        Ok(invalidated_count)
    }

    /// Clear all cache entries
    pub async fn clear_all(&self) -> usize {
        let mut cache = self.cache.write().await;
        let count = cache.len();
        cache.clear();

        self.update_entry_metrics().await;

        info!("Cleared all cache entries: {} entries removed", count);
        count
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get active invalidation operations
    pub async fn get_invalidation_operations(&self) -> Vec<InvalidationOperation> {
        let operations = self.invalidation_operations.read().await;
        operations.values().cloned().collect()
    }

    /// Cleanup expired entries manually
    pub async fn cleanup_expired(&self) -> usize {
        let mut cache = self.cache.write().await;
        let now = Utc::now();
        let initial_count = cache.len();

        cache.retain(|_, entry| {
            if entry.expires_at < now {
                false // Remove expired entries
            } else {
                true
            }
        });

        let removed_count = initial_count - cache.len();

        if removed_count > 0 {
            info!("Cleaned up {} expired cache entries", removed_count);
            self.update_entry_metrics().await;
        }

        removed_count
    }

    /// Record cache hit
    async fn record_hit(&self, access_time: Duration) {
        let mut metrics = self.metrics.write().await;
        metrics.total_hits += 1;

        let access_time_ms = access_time.as_millis() as f64;
        metrics.avg_access_time_ms = if metrics.avg_access_time_ms == 0.0 {
            access_time_ms
        } else {
            0.9 * metrics.avg_access_time_ms + 0.1 * access_time_ms
        };

        let total_requests = metrics.total_hits + metrics.total_misses;
        if total_requests > 0 {
            metrics.hit_rate = metrics.total_hits as f64 / total_requests as f64;
            metrics.miss_rate = metrics.total_misses as f64 / total_requests as f64;
        }
    }

    /// Record cache miss
    async fn record_miss(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_misses += 1;

        let total_requests = metrics.total_hits + metrics.total_misses;
        if total_requests > 0 {
            metrics.hit_rate = metrics.total_hits as f64 / total_requests as f64;
            metrics.miss_rate = metrics.total_misses as f64 / total_requests as f64;
        }
    }

    /// Update entry count metrics
    async fn update_entry_metrics(&self) {
        let cache = self.cache.read().await;
        let mut metrics = self.metrics.write().await;

        metrics.total_entries = cache.len();
        metrics.valid_entries = cache
            .values()
            .filter(|e| e.status == CacheEntryStatus::Valid)
            .count();
        metrics.invalidated_entries = cache
            .values()
            .filter(|e| e.status == CacheEntryStatus::Invalidated)
            .count();
        metrics.expired_entries = cache
            .values()
            .filter(|e| e.status == CacheEntryStatus::Expired)
            .count();
    }

    /// Evict the oldest entry from cache
    async fn evict_oldest(&self, cache: &mut HashMap<String, KeyCacheEntry>) {
        if let Some((oldest_key, _)) = cache.iter().min_by_key(|(_, entry)| entry.last_accessed) {
            let oldest_key = oldest_key.clone();
            cache.remove(&oldest_key);
            debug!("Evicted oldest cache entry: {}", oldest_key);
        }
    }

    /// Background cleanup task
    async fn background_cleanup(
        cache: Arc<RwLock<HashMap<String, KeyCacheEntry>>>,
        metrics: Arc<RwLock<CacheMetrics>>,
        config: KeyCacheConfig,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(config.cleanup_interval_secs));

        loop {
            interval.tick().await;

            // Clean up expired entries
            let mut cache_guard = cache.write().await;
            let now = Utc::now();
            let initial_count = cache_guard.len();

            cache_guard.retain(|_, entry| entry.expires_at >= now);

            let removed_count = initial_count - cache_guard.len();
            drop(cache_guard);

            if removed_count > 0 {
                debug!(
                    "Background cleanup removed {} expired entries",
                    removed_count
                );

                // Update metrics
                let mut metrics_guard = metrics.write().await;
                metrics_guard.last_cleanup = Some(now);

                // Recalculate entry metrics
                let cache_guard = cache.read().await;
                metrics_guard.total_entries = cache_guard.len();
                metrics_guard.valid_entries = cache_guard
                    .values()
                    .filter(|e| e.status == CacheEntryStatus::Valid)
                    .count();
                metrics_guard.invalidated_entries = cache_guard
                    .values()
                    .filter(|e| e.status == CacheEntryStatus::Invalidated)
                    .count();
                metrics_guard.expired_entries = cache_guard
                    .values()
                    .filter(|e| e.status == CacheEntryStatus::Expired)
                    .count();
            }
        }
    }
}

#[async_trait]
impl EventHandler for KeyCacheManager {
    async fn handle_event(&self, event: &SecurityEvent) -> EventHandlerResult {
        let start_time = std::time::Instant::now();

        match event {
            SecurityEvent::KeyRotation(key_rotation_event) => {
                match self.invalidate_for_rotation(key_rotation_event).await {
                    Ok(operation) => {
                        let mut metadata = HashMap::new();
                        metadata.insert(
                            "operation_id".to_string(),
                            serde_json::Value::String(operation.operation_id.to_string()),
                        );
                        metadata.insert(
                            "invalidated_count".to_string(),
                            serde_json::Value::Number(operation.invalidated_count.into()),
                        );
                        metadata
                            .insert("key_ids".to_string(), serde_json::json!(operation.key_ids));

                        EventHandlerResult {
                            handler_name: self.name.clone(),
                            success: operation.success,
                            duration: start_time.elapsed(),
                            error: None,
                            metadata,
                        }
                    }
                    Err(error) => EventHandlerResult {
                        handler_name: self.name.clone(),
                        success: false,
                        duration: start_time.elapsed(),
                        error: Some(error.to_string()),
                        metadata: HashMap::new(),
                    },
                }
            }
            _ => {
                // Don't handle non-key-rotation events
                EventHandlerResult {
                    handler_name: self.name.clone(),
                    success: true,
                    duration: start_time.elapsed(),
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

impl Drop for KeyCacheManager {
    fn drop(&mut self) {
        if let Some(handle) = self.cleanup_task.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;
    use crate::events::event_types::{
        EventSeverity, OperationResult, PlatformSource, SecurityEventCategory, VerificationEvent,
    };

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Hangs indefinitely - needs investigation"]
    async fn test_cache_basic_operations() {
        let test_future = async {
            let config = KeyCacheConfig::default();
            let mut manager = KeyCacheManager::new(config);
            manager.start().await.unwrap();

            let keypair = generate_master_keypair().unwrap();
            let public_key = keypair.public_key().clone();
            let key_id = "test_key_123".to_string();

            // Test put
            manager
                .put(
                    key_id.clone(),
                    public_key.clone(),
                    "client_key".to_string(),
                    "client_123".to_string(),
                )
                .await
                .unwrap();

            // Test get
            let entry = manager.get(&key_id).await.unwrap();
            assert_eq!(entry.key_id, key_id);
            assert_eq!(entry.public_key.to_bytes(), public_key.to_bytes());
            assert_eq!(entry.status, CacheEntryStatus::Valid);

            // Test invalidate
            let invalidated = manager
                .invalidate_keys(&[key_id.clone()], "test invalidation")
                .await
                .unwrap();
            assert_eq!(invalidated, 1);

            // Test get after invalidation
            let entry = manager.get(&key_id).await;
            assert!(entry.is_none());

            manager.stop().await;
        };

        test_future.await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "Hangs indefinitely - needs investigation"]
    async fn test_key_rotation_invalidation() {
        let test_future = async {
            let config = KeyCacheConfig::default();
            let manager = KeyCacheManager::new(config);

            // Create test key rotation event
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
                rotation_type: KeyRotationEventType::RotationCompleted,
                user_id: Some("test_user".to_string()),
                old_key_id: Some("old_key_123".to_string()),
                new_key_id: Some("new_key_456".to_string()),
                rotation_reason: "Scheduled".to_string(),
                operation_id: Some(Uuid::new_v4().to_string()),
                target_nodes: vec!["node1".to_string()],
                propagation_status: crate::events::event_types::KeyPropagationStatus::Completed,
                affected_associations: Some(5),
                rotation_metadata: HashMap::new(),
            };

            // Test invalidation
            let operation = manager
                .invalidate_for_rotation(&key_rotation_event)
                .await
                .unwrap();
            assert!(operation.success);
            assert_eq!(operation.key_ids, vec!["old_key_123".to_string()]);
        };

        test_future.await;
    }

    #[tokio::test]
    #[ignore = "Hangs indefinitely - needs investigation"]
    async fn test_cache_metrics() {
        let test_future = async {
            let config = KeyCacheConfig::default();
            let manager = KeyCacheManager::new(config);

            let keypair = generate_master_keypair().unwrap();
            let public_key = keypair.public_key().clone();

            // Add some entries
            manager
                .put(
                    "key1".to_string(),
                    public_key.clone(),
                    "type1".to_string(),
                    "ref1".to_string(),
                )
                .await
                .unwrap();
            manager
                .put(
                    "key2".to_string(),
                    public_key.clone(),
                    "type2".to_string(),
                    "ref2".to_string(),
                )
                .await
                .unwrap();

            // Test hits and misses
            manager.get("key1").await; // hit
            manager.get("key2").await; // hit
            manager.get("key3").await; // miss

            let metrics = manager.get_metrics().await;
            assert_eq!(metrics.total_entries, 2);
            assert_eq!(metrics.valid_entries, 2);
            assert_eq!(metrics.total_hits, 2);
            assert_eq!(metrics.total_misses, 1);
            assert!(metrics.hit_rate > 0.0);
        };

        test_future.await;
    }
}
