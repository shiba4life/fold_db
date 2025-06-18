//! Cryptographic key operations and management for DataFold node
//!
//! This module provides comprehensive key management capabilities including
//! public key caching, key loading and validation, cache warmup operations,
//! and performance optimizations for cryptographic key operations.

use crate::datafold_node::error::NodeResult;
use crate::error::FoldDbError;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::sync::{atomic::Ordering, Arc, RwLock};
use std::time::{Duration, Instant};

use super::auth_types::{CacheStats, CacheWarmupResult, CachedPublicKey, PublicKeyCache};

impl LatencyHistogram {
    pub fn record(&mut self, latency_ms: u64) {
        // Add to measurements
        self.measurements.push_back(latency_ms);
        if self.measurements.len() > self.max_measurements {
            self.measurements.pop_front();
        }

        // Update histogram buckets
        for &bucket in &[1, 5, 10, 50, 100, 500, 1000, u64::MAX] {
            if latency_ms < bucket {
                *self.buckets.entry(bucket).or_insert(0) += 1;
                break;
            }
        }
    }

    pub fn percentile(&self, p: f64) -> Option<u64> {
        if self.measurements.is_empty() {
            return None;
        }

        let mut sorted: Vec<u64> = self.measurements.iter().copied().collect();
        sorted.sort_unstable();

        let index = ((sorted.len() as f64 * p / 100.0) as usize).saturating_sub(1);
        sorted.get(index).copied()
    }

    pub fn average(&self) -> f64 {
        if self.measurements.is_empty() {
            return 0.0;
        }

        let sum: u64 = self.measurements.iter().sum();
        sum as f64 / self.measurements.len() as f64
    }

    pub fn count(&self) -> usize {
        self.measurements.len()
    }
}

impl PublicKeyCache {
    pub fn get(&mut self, key_id: &str) -> Option<CachedPublicKey> {
        if let Some(cached_key) = self.keys.get_mut(key_id) {
            self.hit_count += 1;
            cached_key.access_count += 1;
            cached_key.last_accessed = Instant::now();
            Some(cached_key.clone())
        } else {
            self.miss_count += 1;
            None
        }
    }

    pub fn put(&mut self, key_id: String, key_bytes: [u8; 32], status: String) {
        // Enforce cache size limit
        if self.keys.len() >= self.max_size {
            self.evict_least_recently_used();
        }

        let cached_key = CachedPublicKey {
            key_bytes,
            cached_at: Instant::now(),
            access_count: 1,
            last_accessed: Instant::now(),
            status,
        };

        self.keys.insert(key_id, cached_key);
    }

    pub fn invalidate(&mut self, key_id: &str) {
        self.keys.remove(key_id);
    }

    pub fn clear(&mut self) {
        self.keys.clear();
        self.hit_count = 0;
        self.miss_count = 0;
        self.warmup_completed = false;
    }

    pub fn hit_rate(&self) -> f64 {
        let total = self.hit_count + self.miss_count;
        if total == 0 {
            0.0
        } else {
            self.hit_count as f64 / total as f64
        }
    }

    pub fn size(&self) -> usize {
        self.keys.len()
    }

    pub fn is_warmup_completed(&self) -> bool {
        self.warmup_completed
    }

    pub fn mark_warmup_completed(&mut self) {
        self.warmup_completed = true;
    }

    fn evict_least_recently_used(&mut self) {
        if let Some((lru_key, _)) = self
            .keys
            .iter()
            .min_by_key(|(_, cached)| cached.last_accessed)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.keys.remove(&lru_key);
        }
    }

    pub fn cleanup_expired(&mut self, ttl: Duration) {
        let now = Instant::now();
        let cutoff = now - ttl;

        self.keys.retain(|_, cached| cached.cached_at > cutoff);
        self.last_cleanup = now;
    }

    /// Get keys that are about to expire (for proactive refresh)
    pub fn get_expiring_keys(&self, ttl: Duration, warning_threshold: Duration) -> Vec<String> {
        let now = Instant::now();
        let expiry_cutoff = now - ttl + warning_threshold;

        self.keys
            .iter()
            .filter(|(_, cached)| cached.cached_at < expiry_cutoff)
            .map(|(key_id, _)| key_id.clone())
            .collect()
    }

    /// Get cache statistics for monitoring
    pub fn get_detailed_stats(&self) -> DetailedCacheStats {
        let total_accesses = self.keys.values().map(|cached| cached.access_count).sum();
        let avg_age = if self.keys.is_empty() {
            Duration::from_secs(0)
        } else {
            let total_age: Duration = self
                .keys
                .values()
                .map(|cached| cached.cached_at.elapsed())
                .sum();
            total_age / self.keys.len() as u32
        };

        DetailedCacheStats {
            size: self.keys.len(),
            max_size: self.max_size,
            hit_count: self.hit_count,
            miss_count: self.miss_count,
            hit_rate: self.hit_rate(),
            total_accesses,
            average_age_secs: avg_age.as_secs(),
            warmup_completed: self.warmup_completed,
            last_cleanup: self.last_cleanup,
        }
    }

    /// Preload keys based on usage patterns
    pub fn preload_frequent_keys(&mut self, usage_threshold: u64) -> Vec<String> {
        self.keys
            .iter()
            .filter(|(_, cached)| cached.access_count >= usage_threshold)
            .map(|(key_id, _)| key_id.clone())
            .collect()
    }
}

/// Detailed cache statistics for monitoring and optimization
#[derive(Debug, Clone)]
pub struct DetailedCacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub total_accesses: u64,
    pub average_age_secs: u64,
    pub warmup_completed: bool,
    pub last_cleanup: Instant,
}

/// Key management operations and utilities
pub struct KeyManager {
    cache: Arc<RwLock<PublicKeyCache>>,
}

impl KeyManager {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(PublicKeyCache::new(max_cache_size))),
        }
    }

    /// Warm up the public key cache by preloading frequently used keys
    pub async fn warm_cache(
        &self,
        db_ops: Arc<crate::db_operations::core::DbOperations>,
    ) -> NodeResult<CacheWarmupResult> {
        let warmup_start = Instant::now();
        let mut keys_loaded = 0;
        let mut errors = 0;

        info!("Starting public key cache warmup...");

        // Get list of active key registrations from database
        match self.load_active_keys_from_database(db_ops).await {
            Ok(loaded_keys) => {
                keys_loaded = loaded_keys.len() as u64;
                
                // Add keys to cache
                if let Ok(mut cache) = self.cache.write() {
                    for (key_id, key_bytes) in loaded_keys {
                        cache.put(key_id, key_bytes, "active".to_string());
                    }
                    cache.mark_warmup_completed();
                }
            }
            Err(e) => {
                warn!("Failed to load keys for cache warmup: {}", e);
                errors = 1;
            }
        }

        let warmup_duration = warmup_start.elapsed();
        let cache_size_after = if let Ok(cache) = self.cache.read() {
            cache.size() as u64
        } else {
            0
        };

        info!(
            "Cache warmup completed: {} keys loaded, {} errors, took {:?}",
            keys_loaded, errors, warmup_duration
        );

        Ok(CacheWarmupResult {
            keys_loaded,
            errors,
            duration_ms: warmup_duration.as_millis() as u64,
            cache_size_after,
        })
    }

    /// Load active keys from database for cache warmup
    async fn load_active_keys_from_database(
        &self,
        db_ops: Arc<crate::db_operations::core::DbOperations>,
    ) -> NodeResult<Vec<(String, [u8; 32])>> {
        use crate::datafold_node::crypto::crypto_routes::{
            CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE,
        };

        let mut keys = Vec::new();

        // This is a simplified version - in a real implementation, you might want to
        // scan the database for active registrations or maintain a separate index
        // For now, we'll just mark the cache as warmed up

        // TODO: Implement actual database scanning for active keys
        // This would involve:
        // 1. Scanning the CLIENT_KEY_INDEX_TREE to get all client IDs
        // 2. For each client ID, checking if the registration is active
        // 3. Loading the public key bytes for active registrations

        debug!("Key warmup: would scan {} and {}", CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE);

        Ok(keys)
    }

    /// Get a public key from cache or database
    pub async fn get_public_key(
        &self,
        key_id: &str,
        db_ops: Arc<crate::db_operations::core::DbOperations>,
    ) -> NodeResult<[u8; 32]> {
        // Try cache first
        if let Ok(mut cache) = self.cache.write() {
            if let Some(cached_key) = cache.get(key_id) {
                if cached_key.status == "active" {
                    debug!("Cache hit for key_id: {}", key_id);
                    return Ok(cached_key.key_bytes);
                }
            }
        }

        // Cache miss - load from database
        debug!("Cache miss for key_id: {}, loading from database", key_id);
        let key_bytes = self.load_key_from_database(key_id, db_ops).await?;

        // Cache the result
        if let Ok(mut cache) = self.cache.write() {
            cache.put(key_id.to_string(), key_bytes, "active".to_string());
        }

        Ok(key_bytes)
    }

    /// Load key from database (extracted from signature verification logic)
    async fn load_key_from_database(
        &self,
        key_id: &str,
        db_ops: Arc<crate::db_operations::core::DbOperations>,
    ) -> NodeResult<[u8; 32]> {
        use crate::datafold_node::crypto::crypto_routes::{
            CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE,
        };

        // Look up registration ID by client ID
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, key_id);
        let registration_id_str = match db_ops.get_item::<String>(&client_index_key) {
            Ok(Some(reg_id)) => reg_id,
            Ok(None) => {
                debug!("No registration found for client_id: {}", key_id);
                return Err(FoldDbError::Permission(format!(
                    "Public key not found for key_id: {}",
                    key_id
                )));
            }
            Err(e) => {
                warn!("Failed to lookup client key index: {}", e);
                return Err(FoldDbError::Permission(format!(
                    "Database error looking up key_id: {}",
                    key_id
                )));
            }
        };

        // Get registration record
        let registration_key =
            format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
        let registration: crate::datafold_node::crypto::crypto_routes::PublicKeyRegistration =
            match db_ops.get_item(&registration_key) {
                Ok(Some(reg)) => reg,
                Ok(None) => {
                    debug!("Registration record not found: {}", registration_id_str);
                    return Err(FoldDbError::Permission(format!(
                        "Registration not found for key_id: {}",
                        key_id
                    )));
                }
                Err(e) => {
                    warn!("Failed to get registration record: {}", e);
                    return Err(FoldDbError::Permission(format!(
                        "Database error retrieving registration for key_id: {}",
                        key_id
                    )));
                }
            };

        // Check if the key is active
        if registration.status != "active" {
            debug!(
                "Public key for {} is not active: {}",
                key_id, registration.status
            );
            return Err(FoldDbError::Permission(format!(
                "Key {} is not active (status: {})",
                key_id, registration.status
            )));
        }

        debug!(
            "Successfully loaded active public key for client_id: {}",
            key_id
        );
        Ok(registration.public_key_bytes)
    }

    /// Invalidate a key in the cache (e.g., when key is revoked)
    pub fn invalidate_key(&self, key_id: &str) -> NodeResult<bool> {
        if let Ok(mut cache) = self.cache.write() {
            let was_present = cache.keys.contains_key(key_id);
            cache.invalidate(key_id);
            debug!("Invalidated key {} from cache", key_id);
            Ok(was_present)
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for invalidation".to_string(),
            ))
        }
    }

    /// Clear the entire cache
    pub fn clear_cache(&self) -> NodeResult<()> {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
            info!("Public key cache cleared");
            Ok(())
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for clearing".to_string(),
            ))
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> NodeResult<CacheStats> {
        if let Ok(cache) = self.cache.read() {
            Ok(CacheStats {
                size: cache.size(),
                hit_rate: cache.hit_rate(),
                warmup_completed: cache.is_warmup_completed(),
            })
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for stats".to_string(),
            ))
        }
    }

    /// Get detailed cache statistics
    pub fn get_detailed_cache_stats(&self) -> NodeResult<DetailedCacheStats> {
        if let Ok(cache) = self.cache.read() {
            Ok(cache.get_detailed_stats())
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for detailed stats".to_string(),
            ))
        }
    }

    /// Perform cache maintenance (cleanup expired entries)
    pub fn maintain_cache(&self, ttl: Duration) -> NodeResult<u64> {
        if let Ok(mut cache) = self.cache.write() {
            let initial_size = cache.size();
            cache.cleanup_expired(ttl);
            let final_size = cache.size();
            let cleaned = initial_size.saturating_sub(final_size) as u64;

            if cleaned > 0 {
                debug!("Cache maintenance: removed {} expired entries", cleaned);
            }

            Ok(cleaned)
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for maintenance".to_string(),
            ))
        }
    }

    /// Check cache health and identify potential issues
    pub fn check_cache_health(&self) -> NodeResult<CacheHealthReport> {
        if let Ok(cache) = self.cache.read() {
            let stats = cache.get_detailed_stats();
            let mut issues = Vec::new();
            let mut recommendations = Vec::new();

            // Check hit rate
            if stats.hit_rate < 0.5 {
                issues.push("Low cache hit rate".to_string());
                recommendations.push("Consider increasing cache size or reviewing key access patterns".to_string());
            }

            // Check cache utilization
            let utilization = stats.size as f64 / stats.max_size as f64;
            if utilization > 0.9 {
                issues.push("High cache utilization".to_string());
                recommendations.push("Consider increasing cache size to prevent frequent evictions".to_string());
            }

            // Check if warmup was completed
            if !stats.warmup_completed {
                issues.push("Cache warmup not completed".to_string());
                recommendations.push("Run cache warmup to improve initial performance".to_string());
            }

            // Check average key age
            if stats.average_age_secs > 3600 {
                // Keys older than 1 hour
                recommendations.push("Consider refreshing old cache entries".to_string());
            }

            let health_score = calculate_cache_health_score(&stats, &issues);

            Ok(CacheHealthReport {
                health_score,
                issues,
                recommendations,
                stats,
            })
        } else {
            Err(FoldDbError::Permission(
                "Failed to acquire cache lock for health check".to_string(),
            ))
        }
    }

    /// Refresh keys that are about to expire
    pub async fn refresh_expiring_keys(
        &self,
        db_ops: Arc<crate::db_operations::core::DbOperations>,
        ttl: Duration,
        warning_threshold: Duration,
    ) -> NodeResult<u32> {
        let expiring_keys = if let Ok(cache) = self.cache.read() {
            cache.get_expiring_keys(ttl, warning_threshold)
        } else {
            return Err(FoldDbError::Permission(
                "Failed to acquire cache lock".to_string(),
            ));
        };

        let mut refreshed_count = 0;
        for key_id in expiring_keys {
            match self.load_key_from_database(&key_id, db_ops.clone()).await {
                Ok(key_bytes) => {
                    if let Ok(mut cache) = self.cache.write() {
                        cache.put(key_id.clone(), key_bytes, "active".to_string());
                        refreshed_count += 1;
                        debug!("Refreshed expiring key: {}", key_id);
                    }
                }
                Err(e) => {
                    warn!("Failed to refresh key {}: {}", key_id, e);
                    // Remove invalid key from cache
                    if let Ok(mut cache) = self.cache.write() {
                        cache.invalidate(&key_id);
                    }
                }
            }
        }

        if refreshed_count > 0 {
            info!("Refreshed {} expiring keys", refreshed_count);
        }

        Ok(refreshed_count)
    }
}

/// Cache health report for monitoring and diagnostics
#[derive(Debug, Clone)]
pub struct CacheHealthReport {
    pub health_score: f64,      // 0.0 to 100.0
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub stats: DetailedCacheStats,
}

/// Calculate cache health score based on various metrics
fn calculate_cache_health_score(stats: &DetailedCacheStats, issues: &[String]) -> f64 {
    let mut score = 100.0;

    // Penalize for low hit rate
    if stats.hit_rate < 0.8 {
        score -= (0.8 - stats.hit_rate) * 50.0;
    }

    // Penalize for high utilization
    let utilization = stats.size as f64 / stats.max_size as f64;
    if utilization > 0.8 {
        score -= (utilization - 0.8) * 50.0;
    }

    // Penalize for not completing warmup
    if !stats.warmup_completed {
        score -= 20.0;
    }

    // Penalize for each issue
    score -= issues.len() as f64 * 10.0;

    // Ensure score is between 0 and 100
    score.max(0.0).min(100.0)
}

/// Import the LatencyHistogram implementation
use super::auth_types::LatencyHistogram;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_key_cache_basic_operations() {
        let mut cache = PublicKeyCache::new(3);
        
        let key_bytes = [1u8; 32];
        let key_id = "test-key".to_string();
        
        // Test put and get
        cache.put(key_id.clone(), key_bytes, "active".to_string());
        assert_eq!(cache.size(), 1);
        
        let retrieved = cache.get(&key_id).unwrap();
        assert_eq!(retrieved.key_bytes, key_bytes);
        assert_eq!(retrieved.status, "active");
        
        // Test hit rate calculation
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = PublicKeyCache::new(2);
        
        // Fill cache to capacity
        cache.put("key1".to_string(), [1u8; 32], "active".to_string());
        cache.put("key2".to_string(), [2u8; 32], "active".to_string());
        assert_eq!(cache.size(), 2);
        
        // Add one more - should evict least recently used
        cache.put("key3".to_string(), [3u8; 32], "active".to_string());
        assert_eq!(cache.size(), 2);
        
        // key1 should be evicted (least recently used)
        assert!(cache.get("key1").is_none());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_cache_invalidation() {
        let mut cache = PublicKeyCache::new(3);
        
        cache.put("key1".to_string(), [1u8; 32], "active".to_string());
        assert!(cache.get("key1").is_some());
        
        cache.invalidate("key1");
        assert!(cache.get("key1").is_none());
        assert_eq!(cache.size(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = PublicKeyCache::new(3);
        
        cache.put("key1".to_string(), [1u8; 32], "active".to_string());
        cache.put("key2".to_string(), [2u8; 32], "active".to_string());
        assert_eq!(cache.size(), 2);
        
        cache.clear();
        assert_eq!(cache.size(), 0);
        assert_eq!(cache.hit_rate(), 0.0);
        assert!(!cache.is_warmup_completed());
    }

    #[test]
    fn test_latency_histogram() {
        let mut histogram = LatencyHistogram::new(1000);
        
        histogram.record(5);
        histogram.record(10);
        histogram.record(15);
        
        assert_eq!(histogram.count(), 3);
        assert_eq!(histogram.average(), 10.0);
        
        // Test percentile calculation
        let p50 = histogram.percentile(50.0);
        assert!(p50.is_some());
        assert_eq!(p50.unwrap(), 10);
    }

    #[test]
    fn test_cache_health_calculation() {
        let stats = DetailedCacheStats {
            size: 500,
            max_size: 1000,
            hit_count: 800,
            miss_count: 200,
            hit_rate: 0.8,
            total_accesses: 1000,
            average_age_secs: 300,
            warmup_completed: true,
            last_cleanup: Instant::now(),
        };
        
        let issues = vec![];
        let score = calculate_cache_health_score(&stats, &issues);
        
        // Should get full score with good stats and no issues
        assert_eq!(score, 100.0);
        
        // Test with issues
        let issues = vec!["Low hit rate".to_string()];
        let score_with_issues = calculate_cache_health_score(&stats, &issues);
        assert!(score_with_issues < 100.0);
    }

    #[tokio::test]
    async fn test_key_manager_cache_operations() {
        let manager = KeyManager::new(10);
        
        // Test cache stats
        let stats = manager.get_cache_stats().unwrap();
        assert_eq!(stats.size, 0);
        assert!(!stats.warmup_completed);
        
        // Test cache clearing
        assert!(manager.clear_cache().is_ok());
        
        // Test cache health check
        let health = manager.check_cache_health().unwrap();
        assert!(health.health_score >= 0.0 && health.health_score <= 100.0);
    }
}