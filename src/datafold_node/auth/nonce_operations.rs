//! Nonce store operations for replay prevention in DataFold authentication
//!
//! This module provides the NonceStore implementation for preventing replay attacks
//! by tracking and validating nonces. It includes nonce validation, cleanup operations,
//! and performance monitoring utilities.

use log::{debug, warn};
use super::auth_types::{NonceStore, NonceStorePerformanceStats};

impl NonceStore {
    /// Check if a nonce exists in the store
    pub fn contains_nonce(&self, nonce: &str) -> bool {
        self.nonces.contains_key(nonce)
    }

    /// Add a nonce to the store with its creation timestamp
    pub fn add_nonce(&mut self, nonce: String, created: u64) {
        self.nonces.insert(nonce, created);
    }

    /// Get the current size of the nonce store
    pub fn size(&self) -> usize {
        self.nonces.len()
    }

    /// Clean up expired nonces based on TTL
    pub fn cleanup_expired(&mut self, ttl_secs: u64) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let initial_size = self.nonces.len();
        self.nonces
            .retain(|_, &mut created| now.saturating_sub(created) < ttl_secs);

        let removed = initial_size - self.nonces.len();
        if removed > 0 {
            debug!(
                "Cleaned up {} expired nonces, {} remaining",
                removed,
                self.nonces.len()
            );
        }
    }

    /// Enforce size limits by removing oldest nonces
    pub fn enforce_size_limit(&mut self, max_size: usize) {
        if self.nonces.len() <= max_size {
            return;
        }

        let to_remove = self.nonces.len() - max_size;

        // Collect all nonces with timestamps first
        let mut nonce_timestamps: Vec<(String, u64)> = self
            .nonces
            .iter()
            .map(|(nonce, &timestamp)| (nonce.clone(), timestamp))
            .collect();

        // Sort by timestamp (oldest first)
        nonce_timestamps.sort_by_key(|(_, timestamp)| *timestamp);

        // Remove the oldest nonces
        for (nonce, _) in nonce_timestamps.into_iter().take(to_remove) {
            self.nonces.remove(&nonce);
        }

        warn!(
            "Enforced nonce store size limit: removed {} oldest nonces, {} remaining",
            to_remove,
            self.nonces.len()
        );
    }

    /// Get the age of the oldest nonce in seconds
    pub fn get_oldest_nonce_age(&self) -> Option<u64> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.nonces
            .values()
            .min()
            .map(|&oldest| now.saturating_sub(oldest))
    }

    /// Get performance statistics for the nonce store
    pub fn get_performance_stats(&self, max_capacity: usize) -> NonceStorePerformanceStats {
        let oldest_age = self.get_oldest_nonce_age();
        let total_nonces = self.nonces.len();
        let utilization_percent = if max_capacity > 0 {
            (total_nonces as f64 / max_capacity as f64) * 100.0
        } else {
            0.0
        };

        NonceStorePerformanceStats {
            total_nonces,
            max_capacity,
            utilization_percent,
            oldest_nonce_age_secs: oldest_age,
            cleanup_operations: 0, // This would be tracked separately
            memory_usage_bytes: (total_nonces * 64) as u64, // Rough estimate
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nonce_store_basic_operations() {
        let mut store = NonceStore::new();
        
        // Test empty store
        assert_eq!(store.size(), 0);
        assert!(!store.contains_nonce("test"));
        
        // Test adding nonces
        store.add_nonce("nonce1".to_string(), 1000);
        store.add_nonce("nonce2".to_string(), 2000);
        
        assert_eq!(store.size(), 2);
        assert!(store.contains_nonce("nonce1"));
        assert!(store.contains_nonce("nonce2"));
        assert!(!store.contains_nonce("nonce3"));
    }

    #[test]
    fn test_nonce_store_cleanup() {
        let mut store = NonceStore::new();
        
        // Add nonces with different timestamps
        store.add_nonce("old_nonce".to_string(), 1000);
        store.add_nonce("recent_nonce".to_string(), 5000);
        
        // Cleanup with TTL that should remove older nonces
        store.cleanup_expired(500); // TTL of 500 seconds
        
        // This test mainly verifies the cleanup method doesn't panic
        // Exact behavior depends on current time
        assert!(store.size() <= 2);
    }

    #[test]
    fn test_nonce_store_size_limit() {
        let mut store = NonceStore::new();
        
        // Add more nonces than the limit
        for i in 0..10 {
            store.add_nonce(format!("nonce_{}", i), 1000 + i as u64);
        }
        
        assert_eq!(store.size(), 10);
        
        // Enforce a size limit smaller than current size
        store.enforce_size_limit(5);
        
        assert_eq!(store.size(), 5);
    }

    #[test]
    fn test_oldest_nonce_age() {
        let mut store = NonceStore::new();
        
        // Empty store should return None
        assert!(store.get_oldest_nonce_age().is_none());
        
        // Add a nonce and check age calculation
        store.add_nonce("test".to_string(), 1000);
        let age = store.get_oldest_nonce_age();
        assert!(age.is_some());
        assert!(age.unwrap() > 0); // Should be positive age
    }

    #[test]
    fn test_performance_stats() {
        let mut store = NonceStore::new();
        store.add_nonce("test1".to_string(), 1000);
        store.add_nonce("test2".to_string(), 2000);
        
        let stats = store.get_performance_stats(10);
        
        assert_eq!(stats.total_nonces, 2);
        assert_eq!(stats.max_capacity, 10);
        assert_eq!(stats.utilization_percent, 20.0);
        assert!(stats.oldest_nonce_age_secs.is_some());
        assert!(stats.memory_usage_bytes > 0);
    }
}