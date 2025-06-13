//! Async operations for the AtomManager with performance optimizations
//!
//! This module provides async variants of atom management operations
//! with performance enhancements including batch processing, caching,
//! and streaming capabilities for large atom operations.

use crate::atom::{Atom, AtomStatus};
use crate::crypto::{CryptoResult, MasterKeyPair};
use crate::db_operations::encryption_wrapper::contexts;
use crate::db_operations::encryption_wrapper_async::{AsyncEncryptionWrapper, AsyncWrapperConfig};
use crate::db_operations::DbOperations;
use crate::schema::SchemaError;
use futures::future::join_all;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task;

/// Configuration for async atom operations
#[derive(Debug, Clone)]
pub struct AsyncAtomConfig {
    /// Maximum number of concurrent atom operations
    pub max_concurrent_operations: usize,
    /// Batch size for bulk atom operations
    pub batch_size: usize,
    /// Enable caching for frequently accessed atoms
    pub enable_atom_cache: bool,
    /// Size of atom cache
    pub atom_cache_size: usize,
    /// TTL for cached atoms in seconds
    pub atom_cache_ttl: u64,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
}

impl Default for AsyncAtomConfig {
    fn default() -> Self {
        Self {
            max_concurrent_operations: 100,
            batch_size: 50,
            enable_atom_cache: true,
            atom_cache_size: 1000,
            atom_cache_ttl: 1800, // 30 minutes
            enable_metrics: true,
        }
    }
}

/// Performance metrics for async atom operations
#[derive(Debug, Clone, Default)]
pub struct AsyncAtomMetrics {
    /// Number of async atom creations
    pub async_atom_creates: u64,
    /// Number of async atom retrievals
    pub async_atom_gets: u64,
    /// Number of async atom updates
    pub async_atom_updates: u64,
    /// Number of batch operations
    pub batch_operations: u64,
    /// Average operation time in microseconds
    pub avg_operation_time_micros: u64,
    /// Cache hit ratio
    pub cache_hit_ratio: f64,
    /// Total bytes processed
    pub total_bytes_processed: u64,
}

/// Async AtomManager with performance optimizations
pub struct AsyncAtomManager {
    /// The underlying database operations
    db_ops: Arc<DbOperations>,
    /// Async encryption wrapper for encrypted operations
    async_encryption: Option<AsyncEncryptionWrapper>,
    /// Configuration for async operations
    config: AsyncAtomConfig,
    /// Semaphore to limit concurrent operations
    operation_semaphore: Arc<Semaphore>,
    /// Performance metrics
    metrics: Arc<RwLock<AsyncAtomMetrics>>,
    /// Cache for frequently accessed atoms
    atom_cache: Arc<RwLock<lru::LruCache<String, (Atom, Instant)>>>,
}

impl AsyncAtomManager {
    /// Create a new async atom manager
    pub async fn new(db_ops: DbOperations, config: AsyncAtomConfig) -> Result<Self, SchemaError> {
        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let atom_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.atom_cache_size).unwrap(),
        )));

        Ok(Self {
            db_ops: Arc::new(db_ops),
            async_encryption: None,
            config,
            operation_semaphore,
            metrics: Arc::new(RwLock::new(AsyncAtomMetrics::default())),
            atom_cache,
        })
    }

    /// Create with async encryption support
    pub async fn with_encryption(
        db_ops: DbOperations,
        master_keypair: &MasterKeyPair,
        config: AsyncAtomConfig,
    ) -> CryptoResult<Self> {
        let async_wrapper_config = AsyncWrapperConfig::default();
        let async_encryption =
            AsyncEncryptionWrapper::new(db_ops.clone(), master_keypair, async_wrapper_config)
                .await?;

        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        let atom_cache = Arc::new(RwLock::new(lru::LruCache::new(
            std::num::NonZeroUsize::new(config.atom_cache_size).unwrap(),
        )));

        Ok(Self {
            db_ops: Arc::new(db_ops),
            async_encryption: Some(async_encryption),
            config,
            operation_semaphore,
            metrics: Arc::new(RwLock::new(AsyncAtomMetrics::default())),
            atom_cache,
        })
    }

    /// Create atom asynchronously
    pub async fn create_atom_async(
        &self,
        schema_name: &str,
        source_pub_key: String,
        content: Value,
    ) -> Result<Atom, SchemaError> {
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            SchemaError::InvalidData("Failed to acquire operation permit".to_string())
        })?;

        let start_time = Instant::now();

        if let Some(encryption) = &self.async_encryption {
            // Use async encryption
            let atom = Atom::new(schema_name.to_string(), source_pub_key, content);
            let key = format!("atom:{}", atom.uuid());

            encryption
                .store_encrypted_item_async(&key, &atom, contexts::ATOM_DATA)
                .await?;

            // Update cache
            if self.config.enable_atom_cache {
                let mut cache = self.atom_cache.write().await;
                cache.put(atom.uuid().to_string(), (atom.clone(), Instant::now()));
            }

            self.update_metrics_async(start_time, "create").await;
            Ok(atom)
        } else {
            // Use sync operations in a blocking task
            let db_ops = Arc::clone(&self.db_ops);
            let schema_name = schema_name.to_string();

            let atom = task::spawn_blocking(move || {
                db_ops.create_atom(
                    &schema_name,
                    source_pub_key,
                    None,
                    content,
                    Some(AtomStatus::Active),
                )
            })
            .await
            .map_err(|e| SchemaError::InvalidData(format!("Async task failed: {}", e)))??;

            // Update cache
            if self.config.enable_atom_cache {
                let mut cache = self.atom_cache.write().await;
                cache.put(atom.uuid().to_string(), (atom.clone(), Instant::now()));
            }

            self.update_metrics_async(start_time, "create").await;
            Ok(atom)
        }
    }

    /// Get atom asynchronously with caching
    pub async fn get_atom_async(&self, atom_uuid: &str) -> Result<Option<Atom>, SchemaError> {
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            SchemaError::InvalidData("Failed to acquire operation permit".to_string())
        })?;

        let start_time = Instant::now();

        // Check cache first
        if self.config.enable_atom_cache {
            let mut cache = self.atom_cache.write().await;
            if let Some((cached_atom, cached_time)) = cache.get_mut(atom_uuid) {
                if cached_time.elapsed() < Duration::from_secs(self.config.atom_cache_ttl) {
                    self.update_metrics_async(start_time, "get_cached").await;
                    return Ok(Some(cached_atom.clone()));
                } else {
                    // Remove expired entry
                    cache.pop(atom_uuid);
                }
            }
        }

        let result = if let Some(encryption) = &self.async_encryption {
            // Use async encryption
            let key = format!("atom:{}", atom_uuid);
            encryption
                .get_encrypted_item_async(&key, contexts::ATOM_DATA)
                .await?
        } else {
            // Use sync operations in a blocking task
            let db_ops = Arc::clone(&self.db_ops);
            let key = format!("atom:{}", atom_uuid);

            task::spawn_blocking(move || db_ops.get_item::<Atom>(&key))
                .await
                .map_err(|e| SchemaError::InvalidData(format!("Async task failed: {}", e)))??
        };

        // Update cache if atom was found
        if let Some(ref atom) = result {
            if self.config.enable_atom_cache {
                let mut cache = self.atom_cache.write().await;
                cache.put(atom_uuid.to_string(), (atom.clone(), Instant::now()));
            }
        }

        self.update_metrics_async(start_time, "get").await;
        Ok(result)
    }

    /// Create multiple atoms in a batch operation
    pub async fn create_atoms_batch_async(
        &self,
        atom_specs: Vec<(String, String, Value)>, // (schema_name, source_pub_key, content)
    ) -> Result<Vec<Atom>, SchemaError> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        // Process in batches for better resource management
        for chunk in atom_specs.chunks(self.config.batch_size) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|(schema_name, source_pub_key, content)| {
                    self.create_atom_async(schema_name, source_pub_key.clone(), content.clone())
                })
                .collect();

            let chunk_results = join_all(futures).await;
            for result in chunk_results {
                results.push(result?);
            }
        }

        // Update batch metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.batch_operations += 1;
        }

        self.update_metrics_async(start_time, "batch_create").await;
        Ok(results)
    }

    /// Get multiple atoms in a batch operation
    pub async fn get_atoms_batch_async(
        &self,
        atom_uuids: Vec<String>,
    ) -> Result<HashMap<String, Atom>, SchemaError> {
        let start_time = Instant::now();
        let mut results = HashMap::new();

        // Process in batches for better resource management
        for chunk in atom_uuids.chunks(self.config.batch_size) {
            let futures: Vec<_> = chunk
                .iter()
                .map(|uuid| {
                    let uuid_clone = uuid.clone();
                    async move {
                        let atom = self.get_atom_async(&uuid_clone).await?;
                        Ok::<_, SchemaError>((uuid_clone, atom))
                    }
                })
                .collect();

            let chunk_results = join_all(futures).await;
            for result in chunk_results {
                let (uuid, atom_opt) = result?;
                if let Some(atom) = atom_opt {
                    results.insert(uuid, atom);
                }
            }
        }

        // Update batch metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.batch_operations += 1;
        }

        self.update_metrics_async(start_time, "batch_get").await;
        Ok(results)
    }

    /// Stream atom processing for very large datasets
    pub async fn stream_atoms_async<F, Fut>(
        &self,
        atom_uuids: Vec<String>,
        mut processor: F,
    ) -> Result<u64, SchemaError>
    where
        F: FnMut(Atom) -> Fut + Send,
        Fut: std::future::Future<Output = Result<(), SchemaError>> + Send,
    {
        let start_time = Instant::now();
        let mut processed_count = 0u64;

        // Process atoms in smaller chunks to manage memory
        for chunk in atom_uuids.chunks(self.config.batch_size / 2) {
            for uuid in chunk {
                if let Some(atom) = self.get_atom_async(uuid).await? {
                    processor(atom).await?;
                    processed_count += 1;
                }
            }
        }

        self.update_metrics_async(start_time, "stream").await;
        Ok(processed_count)
    }

    /// Update atom asynchronously
    pub async fn update_atom_async(
        &self,
        atom_uuid: &str,
        new_content: Value,
        source_pub_key: String,
    ) -> Result<Atom, SchemaError> {
        let _permit = self.operation_semaphore.acquire().await.map_err(|_| {
            SchemaError::InvalidData("Failed to acquire operation permit".to_string())
        })?;

        let start_time = Instant::now();

        // Get the existing atom first
        let existing_atom = self
            .get_atom_async(atom_uuid)
            .await?
            .ok_or_else(|| SchemaError::InvalidData(format!("Atom not found: {}", atom_uuid)))?;

        // Create new atom with updated content
        let new_atom = Atom::new(
            existing_atom.source_schema_name().to_string(),
            source_pub_key,
            new_content,
        )
        .with_prev_version(atom_uuid.to_string())
        .with_status(AtomStatus::Active);

        if let Some(encryption) = &self.async_encryption {
            // Store with async encryption
            let key = format!("atom:{}", new_atom.uuid());
            encryption
                .store_encrypted_item_async(&key, &new_atom, contexts::ATOM_DATA)
                .await?;
        } else {
            // Use sync operations in a blocking task
            let db_ops = Arc::clone(&self.db_ops);
            let key = format!("atom:{}", new_atom.uuid());
            let atom_clone = new_atom.clone();

            task::spawn_blocking(move || db_ops.store_item(&key, &atom_clone))
                .await
                .map_err(|e| SchemaError::InvalidData(format!("Async task failed: {}", e)))??;
        }

        // Update cache
        if self.config.enable_atom_cache {
            let mut cache = self.atom_cache.write().await;
            cache.put(
                new_atom.uuid().to_string(),
                (new_atom.clone(), Instant::now()),
            );
            // Remove old atom from cache
            cache.pop(atom_uuid);
        }

        self.update_metrics_async(start_time, "update").await;
        Ok(new_atom)
    }

    /// Clear atom cache
    pub async fn clear_atom_cache(&self) {
        if self.config.enable_atom_cache {
            let mut cache = self.atom_cache.write().await;
            cache.clear();
        }
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize, f64) {
        if self.config.enable_atom_cache {
            let cache = self.atom_cache.read().await;
            let utilization = (cache.len() as f64 / cache.cap().get() as f64) * 100.0;
            (cache.len(), cache.cap().get(), utilization)
        } else {
            (0, 0, 0.0)
        }
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> AsyncAtomMetrics {
        self.metrics.read().await.clone()
    }

    /// Reset performance metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = AsyncAtomMetrics::default();
    }

    /// Benchmark atom operations performance
    pub async fn benchmark_performance(
        &self,
        num_atoms: usize,
        content_size: usize,
    ) -> Result<HashMap<String, f64>, SchemaError> {
        let mut results = HashMap::new();

        // Create test content
        let test_content = serde_json::json!({
            "data": "x".repeat(content_size),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        // Benchmark creation
        let create_start = Instant::now();
        let mut created_atoms = Vec::new();
        for i in 0..num_atoms {
            let atom = self
                .create_atom_async(
                    "benchmark_schema",
                    format!("pub_key_{}", i),
                    test_content.clone(),
                )
                .await?;
            created_atoms.push(atom.uuid().to_string());
        }
        let create_duration = create_start.elapsed();
        let create_ops_per_sec = num_atoms as f64 / create_duration.as_secs_f64();
        results.insert("create_ops_per_sec".to_string(), create_ops_per_sec);

        // Benchmark retrieval
        let get_start = Instant::now();
        for uuid in &created_atoms {
            let _atom = self.get_atom_async(uuid).await?;
        }
        let get_duration = get_start.elapsed();
        let get_ops_per_sec = num_atoms as f64 / get_duration.as_secs_f64();
        results.insert("get_ops_per_sec".to_string(), get_ops_per_sec);

        // Benchmark batch operations
        let batch_start = Instant::now();
        let _batch_result = self.get_atoms_batch_async(created_atoms).await?;
        let batch_duration = batch_start.elapsed();
        let batch_ops_per_sec = num_atoms as f64 / batch_duration.as_secs_f64();
        results.insert("batch_get_ops_per_sec".to_string(), batch_ops_per_sec);

        Ok(results)
    }

    /// Validate performance overhead against baseline
    pub async fn validate_performance_overhead(
        &self,
        baseline_ops_per_sec: f64,
        max_overhead_percent: f64,
    ) -> Result<bool, SchemaError> {
        let benchmark_results = self.benchmark_performance(100, 1024).await?;

        if let Some(&current_ops_per_sec) = benchmark_results.get("create_ops_per_sec") {
            let overhead =
                ((baseline_ops_per_sec - current_ops_per_sec) / baseline_ops_per_sec) * 100.0;
            return Ok(overhead <= max_overhead_percent);
        }

        Ok(false)
    }

    /// Update performance metrics
    async fn update_metrics_async(&self, start_time: Instant, operation_type: &str) {
        if !self.config.enable_metrics {
            return;
        }

        let elapsed = start_time.elapsed();
        let mut metrics = self.metrics.write().await;

        match operation_type {
            "create" | "batch_create" => {
                metrics.async_atom_creates += 1;
            }
            "get" | "get_cached" | "batch_get" => {
                metrics.async_atom_gets += 1;
            }
            "update" => {
                metrics.async_atom_updates += 1;
            }
            _ => {}
        }

        let total_ops =
            metrics.async_atom_creates + metrics.async_atom_gets + metrics.async_atom_updates;
        if total_ops > 0 {
            metrics.avg_operation_time_micros =
                (metrics.avg_operation_time_micros * (total_ops - 1) + elapsed.as_micros() as u64)
                    / total_ops;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;
    use tempfile::tempdir;

    async fn create_test_async_atom_manager() -> AsyncAtomManager {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        let config = AsyncAtomConfig {
            max_concurrent_operations: 10,
            batch_size: 5,
            atom_cache_size: 100,
            ..Default::default()
        };

        AsyncAtomManager::new(db_ops, config).await.unwrap()
    }

    async fn create_test_encrypted_async_atom_manager() -> AsyncAtomManager {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        let master_keypair = generate_master_keypair().unwrap();
        let config = AsyncAtomConfig {
            max_concurrent_operations: 10,
            batch_size: 5,
            atom_cache_size: 100,
            ..Default::default()
        };

        AsyncAtomManager::with_encryption(db_ops, &master_keypair, config)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_async_create_and_get_atom() {
        let manager = create_test_async_atom_manager().await;

        let test_content = serde_json::json!({"test": "data"});
        let atom = manager
            .create_atom_async(
                "test_schema",
                "test_pub_key".to_string(),
                test_content.clone(),
            )
            .await
            .unwrap();

        let retrieved = manager.get_atom_async(atom.uuid()).await.unwrap().unwrap();
        assert_eq!(atom.uuid(), retrieved.uuid());
        assert_eq!(atom.content(), retrieved.content());
    }

    #[tokio::test]
    async fn test_async_encrypted_operations() {
        let manager = create_test_encrypted_async_atom_manager().await;

        let test_content = serde_json::json!({"encrypted": "data"});
        let atom = manager
            .create_atom_async(
                "encrypted_schema",
                "encrypted_pub_key".to_string(),
                test_content.clone(),
            )
            .await
            .unwrap();

        let retrieved = manager.get_atom_async(atom.uuid()).await.unwrap().unwrap();
        assert_eq!(atom.content(), retrieved.content());
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let manager = create_test_async_atom_manager().await;

        let atom_specs = vec![
            (
                "schema1".to_string(),
                "key1".to_string(),
                serde_json::json!({"data": 1}),
            ),
            (
                "schema2".to_string(),
                "key2".to_string(),
                serde_json::json!({"data": 2}),
            ),
            (
                "schema3".to_string(),
                "key3".to_string(),
                serde_json::json!({"data": 3}),
            ),
        ];

        let created_atoms = manager.create_atoms_batch_async(atom_specs).await.unwrap();
        assert_eq!(created_atoms.len(), 3);

        let uuids: Vec<String> = created_atoms.iter().map(|a| a.uuid().to_string()).collect();
        let retrieved_atoms = manager.get_atoms_batch_async(uuids).await.unwrap();
        assert_eq!(retrieved_atoms.len(), 3);
    }

    #[tokio::test]
    async fn test_atom_caching() {
        let manager = create_test_async_atom_manager().await;

        let test_content = serde_json::json!({"cached": "data"});
        let atom = manager
            .create_atom_async("cache_schema", "cache_pub_key".to_string(), test_content)
            .await
            .unwrap();

        // First retrieval (cache miss)
        let _retrieved1 = manager.get_atom_async(atom.uuid()).await.unwrap();

        // Second retrieval (should be cache hit)
        let _retrieved2 = manager.get_atom_async(atom.uuid()).await.unwrap();

        let (cache_size, cache_capacity, _utilization) = manager.get_cache_stats().await;
        assert!(cache_size > 0);
        assert!(cache_capacity > 0);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let manager = create_test_async_atom_manager().await;

        let test_content = serde_json::json!({"metrics": "test"});
        let _atom = manager
            .create_atom_async(
                "metrics_schema",
                "metrics_pub_key".to_string(),
                test_content,
            )
            .await
            .unwrap();

        let metrics = manager.get_performance_metrics().await;
        assert!(metrics.async_atom_creates > 0);
    }

    #[tokio::test]
    async fn test_benchmark_performance() {
        let manager = create_test_async_atom_manager().await;

        let results = manager.benchmark_performance(5, 100).await.unwrap();

        assert!(results.contains_key("create_ops_per_sec"));
        assert!(results.contains_key("get_ops_per_sec"));
        assert!(results.contains_key("batch_get_ops_per_sec"));

        for (_metric, value) in results {
            assert!(value > 0.0);
        }
    }

    #[tokio::test]
    async fn test_stream_processing() {
        let manager = create_test_async_atom_manager().await;

        // Create some test atoms
        let atom_specs = vec![
            (
                "stream_schema".to_string(),
                "key1".to_string(),
                serde_json::json!({"stream": 1}),
            ),
            (
                "stream_schema".to_string(),
                "key2".to_string(),
                serde_json::json!({"stream": 2}),
            ),
        ];

        let created_atoms = manager.create_atoms_batch_async(atom_specs).await.unwrap();
        let uuids: Vec<String> = created_atoms.iter().map(|a| a.uuid().to_string()).collect();

        let processor = |_atom: Atom| async move { Ok::<_, SchemaError>(()) };

        let result = manager.stream_atoms_async(uuids, processor).await.unwrap();
        assert_eq!(result, 2);
    }
}
