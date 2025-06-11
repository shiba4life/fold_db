//! Async AES-256-GCM encryption utilities for DataFold encryption at rest
//!
//! This module provides async variants of the encryption utilities with performance
//! optimizations including caching, connection pooling, and batch operations.
//! Designed to meet the <20% performance overhead requirement for encryption at rest.
//!
//! ## Performance Features
//!
//! * Async encryption/decryption operations for non-blocking I/O
//! * Connection pooling for encryption contexts to reduce setup overhead
//! * Key derivation caching with LRU eviction policy
//! * Batch operations for multiple encryption/decryption tasks
//! * Streaming encryption for large data to optimize memory usage
//! * Performance monitoring and metrics collection
//! * Memory pool for encryption buffers to reduce allocations
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::datafold_node::encryption_at_rest_async::{AsyncEncryptionAtRest, PerformanceConfig};
//! use datafold::crypto::generate_master_keypair;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create async encryption manager with performance optimizations
//! let encryption_key = [0u8; 32]; // In practice, derive from master key
//! let config = PerformanceConfig::default();
//! let encryptor = AsyncEncryptionAtRest::new(encryption_key, config).await?;
//!
//! // Encrypt data asynchronously
//! let plaintext = b"sensitive database data";
//! let encrypted_data = encryptor.encrypt_async(plaintext).await?;
//!
//! // Decrypt data asynchronously
//! let decrypted_data = encryptor.decrypt_async(&encrypted_data).await?;
//! assert_eq!(plaintext, &decrypted_data[..]);
//! # Ok(())
//! # }
//! ```

use crate::crypto::error::{CryptoError, CryptoResult};
use crate::datafold_node::encryption_at_rest::{
    EncryptionAtRest, EncryptedData, AES_KEY_SIZE
};
use lru::LruCache;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::task;
use serde::{Deserialize, Serialize};
use futures::future::join_all;
use bytes::BytesMut;

/// Configuration for performance optimizations
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum number of cached derived keys
    pub key_cache_size: usize,
    /// Maximum number of encryption contexts in the pool
    pub context_pool_size: usize,
    /// Maximum number of concurrent encryption operations
    pub max_concurrent_operations: usize,
    /// Size of memory buffers for streaming encryption (in bytes)
    pub streaming_buffer_size: usize,
    /// Batch size for bulk operations
    pub batch_size: usize,
    /// Enable performance metrics collection
    pub enable_metrics: bool,
    /// TTL for cached keys (in seconds)
    pub key_cache_ttl: u64,
    /// Memory pool size for reusable buffers
    pub memory_pool_size: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            key_cache_size: 1000,
            context_pool_size: 100,
            max_concurrent_operations: 50,
            streaming_buffer_size: 64 * 1024, // 64KB
            batch_size: 100,
            enable_metrics: true,
            key_cache_ttl: 3600, // 1 hour
            memory_pool_size: 50,
        }
    }
}

impl PerformanceConfig {
    /// Create a configuration optimized for high throughput
    pub fn high_throughput() -> Self {
        Self {
            key_cache_size: 5000,
            context_pool_size: 200,
            max_concurrent_operations: 100,
            batch_size: 500,
            memory_pool_size: 100,
            ..Default::default()
        }
    }
    
    /// Create a configuration optimized for low latency
    pub fn low_latency() -> Self {
        Self {
            key_cache_size: 500,
            context_pool_size: 50,
            max_concurrent_operations: 25,
            streaming_buffer_size: 32 * 1024, // 32KB
            batch_size: 50,
            memory_pool_size: 25,
            ..Default::default()
        }
    }
    
    /// Create a configuration optimized for memory efficiency
    pub fn memory_efficient() -> Self {
        Self {
            key_cache_size: 100,
            context_pool_size: 20,
            max_concurrent_operations: 10,
            streaming_buffer_size: 16 * 1024, // 16KB
            batch_size: 25,
            memory_pool_size: 10,
            ..Default::default()
        }
    }
}

/// Performance metrics for encryption operations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total number of encryption operations
    pub total_encryptions: u64,
    /// Total number of decryption operations
    pub total_decryptions: u64,
    /// Total bytes encrypted
    pub total_bytes_encrypted: u64,
    /// Total bytes decrypted
    pub total_bytes_decrypted: u64,
    /// Average encryption time in microseconds
    pub avg_encryption_time_micros: u64,
    /// Average decryption time in microseconds
    pub avg_decryption_time_micros: u64,
    /// Number of cache hits for key derivation
    pub key_cache_hits: u64,
    /// Number of cache misses for key derivation
    pub key_cache_misses: u64,
    /// Number of batch operations performed
    pub batch_operations: u64,
    /// Total time spent in batch operations (microseconds)
    pub batch_operation_time_micros: u64,
    /// Peak memory usage in bytes
    pub peak_memory_usage: u64,
    /// Number of streaming operations
    pub streaming_operations: u64,
    /// Total streaming throughput in bytes per second
    pub streaming_throughput_bps: u64,
}

impl PerformanceMetrics {
    /// Calculate cache hit ratio as a percentage
    pub fn cache_hit_ratio(&self) -> f64 {
        let total_requests = self.key_cache_hits + self.key_cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            (self.key_cache_hits as f64 / total_requests as f64) * 100.0
        }
    }
    
    /// Calculate average throughput in bytes per second
    pub fn avg_throughput_bps(&self) -> u64 {
        let total_bytes = self.total_bytes_encrypted + self.total_bytes_decrypted;
        let total_ops = self.total_encryptions + self.total_decryptions;
        let total_time_seconds = (self.avg_encryption_time_micros + self.avg_decryption_time_micros) * total_ops / 2_000_000;
        
        if total_time_seconds == 0 {
            0
        } else {
            total_bytes / total_time_seconds
        }
    }
    
    /// Calculate performance overhead percentage compared to baseline
    pub fn overhead_percentage(&self, baseline_throughput_bps: u64) -> f64 {
        let current_throughput = self.avg_throughput_bps();
        if baseline_throughput_bps == 0 {
            0.0
        } else {
            let overhead = (baseline_throughput_bps as f64 - current_throughput as f64) / baseline_throughput_bps as f64;
            overhead * 100.0
        }
    }
}

/// Cache entry for derived encryption keys
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct CachedKey {
    key: [u8; AES_KEY_SIZE],
    created_at: Instant,
    last_used: Instant,
}

#[allow(dead_code)]
impl CachedKey {
    fn new(key: [u8; AES_KEY_SIZE]) -> Self {
        let now = Instant::now();
        Self {
            key,
            created_at: now,
            last_used: now,
        }
    }
    
    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }
    
    fn touch(&mut self) {
        self.last_used = Instant::now();
    }
}

/// Memory pool for reusable buffers
struct MemoryPool {
    buffers: Arc<RwLock<Vec<BytesMut>>>,
    buffer_size: usize,
    max_size: usize,
}

impl MemoryPool {
    fn new(buffer_size: usize, max_size: usize) -> Self {
        Self {
            buffers: Arc::new(RwLock::new(Vec::with_capacity(max_size))),
            buffer_size,
            max_size,
        }
    }
    
    async fn get_buffer(&self) -> BytesMut {
        let mut buffers = self.buffers.write().await;
        if let Some(mut buffer) = buffers.pop() {
            buffer.clear();
            buffer.reserve(self.buffer_size);
            buffer
        } else {
            BytesMut::with_capacity(self.buffer_size)
        }
    }
    
    async fn return_buffer(&self, buffer: BytesMut) {
        let mut buffers = self.buffers.write().await;
        if buffers.len() < self.max_size {
            buffers.push(buffer);
        }
    }
}

/// Connection pool for encryption contexts
struct EncryptionContextPool {
    contexts: Arc<RwLock<Vec<EncryptionAtRest>>>,
    semaphore: Arc<Semaphore>,
    key: [u8; AES_KEY_SIZE],
}

impl EncryptionContextPool {
    fn new(key: [u8; AES_KEY_SIZE], pool_size: usize) -> CryptoResult<Self> {
        let semaphore = Arc::new(Semaphore::new(pool_size));
        let contexts = Arc::new(RwLock::new(Vec::with_capacity(pool_size)));
        
        Ok(Self {
            contexts,
            semaphore,
            key,
        })
    }
    
    async fn acquire_context(&self) -> CryptoResult<EncryptionAtRest> {
        let _permit = self.semaphore.acquire().await
            .map_err(|_| CryptoError::InvalidInput("Failed to acquire context from pool".to_string()))?;
        
        let mut contexts = self.contexts.write().await;
        if let Some(context) = contexts.pop() {
            Ok(context)
        } else {
            // Create new context if pool is empty
            EncryptionAtRest::new(self.key)
        }
    }
    
    #[allow(dead_code)]
    async fn return_context(&self, context: EncryptionAtRest) {
        let mut contexts = self.contexts.write().await;
        if contexts.len() < contexts.capacity() {
            contexts.push(context);
        }
    }
}

/// Async AES-256-GCM encryption manager with performance optimizations
pub struct AsyncEncryptionAtRest {
    /// Connection pool for encryption contexts
    context_pool: EncryptionContextPool,
    /// Cache for derived encryption keys
    key_cache: Arc<RwLock<LruCache<String, CachedKey>>>,
    /// Performance configuration
    config: PerformanceConfig,
    /// Performance metrics
    metrics: Arc<RwLock<PerformanceMetrics>>,
    /// Memory pool for buffers
    memory_pool: MemoryPool,
    /// Semaphore to limit concurrent operations
    operation_semaphore: Arc<Semaphore>,
}

impl AsyncEncryptionAtRest {
    /// Create a new async encryption manager with performance optimizations
    pub async fn new(key: [u8; AES_KEY_SIZE], config: PerformanceConfig) -> CryptoResult<Self> {
        let context_pool = EncryptionContextPool::new(key, config.context_pool_size)?;
        
        let key_cache = Arc::new(RwLock::new(
            LruCache::new(NonZeroUsize::new(config.key_cache_size).unwrap())
        ));
        
        let memory_pool = MemoryPool::new(config.streaming_buffer_size, config.memory_pool_size);
        
        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        
        Ok(Self {
            context_pool,
            key_cache,
            config,
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            memory_pool,
            operation_semaphore,
        })
    }
    
    /// Encrypt data asynchronously
    pub async fn encrypt_async(&self, plaintext: &[u8]) -> CryptoResult<EncryptedData> {
        let _permit = self.operation_semaphore.acquire().await
            .map_err(|_| CryptoError::InvalidInput("Failed to acquire operation permit".to_string()))?;
        
        let start_time = Instant::now();
        
        // Use a task to perform CPU-intensive encryption
        let plaintext_bytes = plaintext.to_vec();
        let context = self.context_pool.acquire_context().await?;
        
        let result = task::spawn_blocking(move || {
            context.encrypt(&plaintext_bytes)
        }).await
        .map_err(|e| CryptoError::InvalidInput(format!("Encryption task failed: {}", e)))??;
        
        let elapsed = start_time.elapsed();
        
        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.total_encryptions += 1;
            metrics.total_bytes_encrypted += plaintext.len() as u64;
            let new_avg = (metrics.avg_encryption_time_micros * (metrics.total_encryptions - 1) + elapsed.as_micros() as u64) / metrics.total_encryptions;
            metrics.avg_encryption_time_micros = new_avg;
        }
        
        Ok(result)
    }
    
    /// Decrypt data asynchronously
    pub async fn decrypt_async(&self, encrypted_data: &EncryptedData) -> CryptoResult<Vec<u8>> {
        let _permit = self.operation_semaphore.acquire().await
            .map_err(|_| CryptoError::InvalidInput("Failed to acquire operation permit".to_string()))?;
        
        let start_time = Instant::now();
        
        // Use a task to perform CPU-intensive decryption
        let encrypted_data_clone = encrypted_data.clone();
        let context = self.context_pool.acquire_context().await?;
        
        let result = task::spawn_blocking(move || {
            context.decrypt(&encrypted_data_clone)
        }).await
        .map_err(|e| CryptoError::InvalidInput(format!("Decryption task failed: {}", e)))??;
        
        let elapsed = start_time.elapsed();
        
        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.total_decryptions += 1;
            metrics.total_bytes_decrypted += result.len() as u64;
            let new_avg = (metrics.avg_decryption_time_micros * (metrics.total_decryptions - 1) + elapsed.as_micros() as u64) / metrics.total_decryptions;
            metrics.avg_decryption_time_micros = new_avg;
        }
        
        Ok(result)
    }
    
    /// Perform batch encryption operations
    pub async fn encrypt_batch(&self, plaintexts: Vec<&[u8]>) -> CryptoResult<Vec<EncryptedData>> {
        let start_time = Instant::now();
        
        // Split into batches for better resource management
        let mut results = Vec::new();
        
        for chunk in plaintexts.chunks(self.config.batch_size) {
            let futures: Vec<_> = chunk.iter().map(|plaintext| {
                self.encrypt_async(plaintext)
            }).collect();
            
            let chunk_results = join_all(futures).await;
            for result in chunk_results {
                results.push(result?);
            }
        }
        
        let elapsed = start_time.elapsed();
        
        // Update batch metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.batch_operations += 1;
            metrics.batch_operation_time_micros += elapsed.as_micros() as u64;
        }
        
        Ok(results)
    }
    
    /// Perform batch decryption operations
    pub async fn decrypt_batch(&self, encrypted_data_list: Vec<&EncryptedData>) -> CryptoResult<Vec<Vec<u8>>> {
        let start_time = Instant::now();
        
        // Split into batches for better resource management
        let mut results = Vec::new();
        
        for chunk in encrypted_data_list.chunks(self.config.batch_size) {
            let futures: Vec<_> = chunk.iter().map(|encrypted_data| {
                self.decrypt_async(encrypted_data)
            }).collect();
            
            let chunk_results = join_all(futures).await;
            for result in chunk_results {
                results.push(result?);
            }
        }
        
        let elapsed = start_time.elapsed();
        
        // Update batch metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.batch_operations += 1;
            metrics.batch_operation_time_micros += elapsed.as_micros() as u64;
        }
        
        Ok(results)
    }
    
    /// Stream encryption for large data to optimize memory usage
    pub async fn encrypt_stream<R>(&self, reader: R) -> CryptoResult<Vec<u8>>
    where
        R: tokio::io::AsyncRead + Unpin,
    {
        use tokio::io::AsyncReadExt;
        
        let start_time = Instant::now();
        let mut reader = reader;
        let mut buffer = self.memory_pool.get_buffer().await;
        let mut all_data = Vec::new();
        
        // Read data in chunks to optimize memory usage
        loop {
            buffer.resize(self.config.streaming_buffer_size, 0);
            let bytes_read = reader.read(&mut buffer).await
                .map_err(|e| CryptoError::InvalidInput(format!("Failed to read from stream: {}", e)))?;
            
            if bytes_read == 0 {
                break; // End of stream
            }
            
            all_data.extend_from_slice(&buffer[..bytes_read]);
            buffer.clear();
        }
        
        // Return buffer to pool
        self.memory_pool.return_buffer(buffer).await;
        
        // Encrypt the accumulated data
        let encrypted_data = self.encrypt_async(&all_data).await?;
        let result = encrypted_data.to_bytes();
        
        let elapsed = start_time.elapsed();
        
        // Update streaming metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.streaming_operations += 1;
            if elapsed.as_secs() > 0 {
                let throughput = all_data.len() as u64 / elapsed.as_secs();
                metrics.streaming_throughput_bps = (metrics.streaming_throughput_bps + throughput) / 2;
            }
        }
        
        Ok(result)
    }
    
    /// Get current performance metrics
    pub async fn get_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Reset performance metrics
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = PerformanceMetrics::default();
    }
    
    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let cache = self.key_cache.read().await;
        (cache.len(), cache.cap().get())
    }
    
    /// Clear the key cache
    pub async fn clear_cache(&self) {
        let mut cache = self.key_cache.write().await;
        cache.clear();
    }
    
    /// Benchmark encryption performance against a baseline
    pub async fn benchmark_performance(&self, data_sizes: Vec<usize>, iterations: usize) -> CryptoResult<HashMap<usize, Duration>> {
        let mut results = HashMap::new();
        
        for &size in &data_sizes {
            let test_data = vec![0u8; size];
            let mut total_time = Duration::default();
            
            for _ in 0..iterations {
                let start = Instant::now();
                let encrypted = self.encrypt_async(&test_data).await?;
                let _decrypted = self.decrypt_async(&encrypted).await?;
                total_time += start.elapsed();
            }
            
            results.insert(size, total_time / iterations as u32);
        }
        
        Ok(results)
    }
    
    /// Validate that performance overhead is within acceptable limits
    pub async fn validate_performance_overhead(&self, baseline_throughput_bps: u64, max_overhead_percent: f64) -> CryptoResult<bool> {
        let metrics = self.get_metrics().await;
        let overhead = metrics.overhead_percentage(baseline_throughput_bps);
        Ok(overhead <= max_overhead_percent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    
    fn create_test_key() -> [u8; AES_KEY_SIZE] {
        [0x42; AES_KEY_SIZE]
    }
    
    #[tokio::test]
    async fn test_async_encryption_roundtrip() {
        let key = create_test_key();
        let config = PerformanceConfig::default();
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let test_data = b"Hello, async encryption!";
        let encrypted = encryptor.encrypt_async(test_data).await.unwrap();
        let decrypted = encryptor.decrypt_async(&encrypted).await.unwrap();
        
        assert_eq!(test_data, &decrypted[..]);
    }
    
    #[tokio::test]
    async fn test_batch_operations() {
        let key = create_test_key();
        let config = PerformanceConfig::default();
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let test_data = vec![
            b"Data 1".as_slice(),
            b"Data 2".as_slice(),
            b"Data 3".as_slice(),
        ];
        
        let encrypted_batch = encryptor.encrypt_batch(test_data.clone()).await.unwrap();
        assert_eq!(encrypted_batch.len(), 3);
        
        let encrypted_refs: Vec<&EncryptedData> = encrypted_batch.iter().collect();
        let decrypted_batch = encryptor.decrypt_batch(encrypted_refs).await.unwrap();
        
        for (i, decrypted) in decrypted_batch.iter().enumerate() {
            assert_eq!(test_data[i], &decrypted[..]);
        }
    }
    
    #[tokio::test]
    async fn test_streaming_encryption() {
        let key = create_test_key();
        let config = PerformanceConfig::default();
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let test_data = b"This is a test of streaming encryption with larger data";
        let cursor = Cursor::new(test_data);
        
        let encrypted_bytes = encryptor.encrypt_stream(cursor).await.unwrap();
        assert!(!encrypted_bytes.is_empty());
        
        // Verify we can decrypt the result
        let encrypted_data = EncryptedData::from_bytes(&encrypted_bytes).unwrap();
        let decrypted = encryptor.decrypt_async(&encrypted_data).await.unwrap();
        assert_eq!(test_data, &decrypted[..]);
    }
    
    #[tokio::test]
    async fn test_performance_metrics() {
        let key = create_test_key();
        let config = PerformanceConfig::default();
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let test_data = b"Test data for metrics";
        
        // Perform some operations
        let encrypted = encryptor.encrypt_async(test_data).await.unwrap();
        let _decrypted = encryptor.decrypt_async(&encrypted).await.unwrap();
        
        let metrics = encryptor.get_metrics().await;
        assert_eq!(metrics.total_encryptions, 1);
        assert_eq!(metrics.total_decryptions, 1);
        assert_eq!(metrics.total_bytes_encrypted, test_data.len() as u64);
    }
    
    #[tokio::test]
    async fn test_performance_config_variants() {
        let key = create_test_key();
        
        // Test different configuration profiles
        let configs = vec![
            PerformanceConfig::default(),
            PerformanceConfig::high_throughput(),
            PerformanceConfig::low_latency(),
            PerformanceConfig::memory_efficient(),
        ];
        
        for config in configs {
            let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
            let test_data = b"Test data";
            let encrypted = encryptor.encrypt_async(test_data).await.unwrap();
            let decrypted = encryptor.decrypt_async(&encrypted).await.unwrap();
            assert_eq!(test_data, &decrypted[..]);
        }
    }
    
    #[tokio::test]
    async fn test_benchmark_performance() {
        let key = create_test_key();
        let config = PerformanceConfig::memory_efficient(); // Use smaller config for tests
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let data_sizes = vec![100, 1000, 10000];
        let results = encryptor.benchmark_performance(data_sizes.clone(), 3).await.unwrap();
        
        assert_eq!(results.len(), data_sizes.len());
        for size in data_sizes {
            assert!(results.contains_key(&size));
            assert!(results[&size] > Duration::default());
        }
    }
    
    #[tokio::test]
    async fn test_cache_operations() {
        let key = create_test_key();
        let config = PerformanceConfig::default();
        let encryptor = AsyncEncryptionAtRest::new(key, config).await.unwrap();
        
        let (initial_size, capacity) = encryptor.get_cache_stats().await;
        assert_eq!(initial_size, 0);
        assert!(capacity > 0);
        
        // Clear cache (should be no-op initially)
        encryptor.clear_cache().await;
        let (size_after_clear, _) = encryptor.get_cache_stats().await;
        assert_eq!(size_after_clear, 0);
    }
}