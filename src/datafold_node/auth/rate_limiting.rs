//! Rate limiting functionality for DataFold authentication
//!
//! This module provides rate limiting capabilities to prevent abuse and protect
//! against brute force attacks. It tracks both general request rates and failure
//! rates per client, with configurable windows and thresholds.

use super::auth_config::RateLimitingConfig;
use super::auth_types::RateLimiter;

impl RateLimiter {
    /// Check if a client's request should be rate limited
    /// 
    /// # Arguments
    /// * `client_id` - Unique identifier for the client (IP, key_id, etc.)
    /// * `config` - Rate limiting configuration
    /// * `is_failure` - Whether this request represents a failure
    /// 
    /// # Returns
    /// * `true` if the request should be allowed
    /// * `false` if the request should be rate limited
    pub fn check_rate_limit(
        &mut self,
        client_id: &str,
        config: &RateLimitingConfig,
        is_failure: bool,
    ) -> bool {
        if !config.enabled {
            return true; // Rate limiting disabled
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Clean up old entries first
        self.cleanup_expired_entries(now, config.window_size_secs);

        // Check general request rate limit
        let requests = self
            .client_requests
            .entry(client_id.to_string())
            .or_default();
        requests.push(now);

        if requests.len() > config.max_requests_per_window {
            return false;
        }

        // Check failure rate limit separately if enabled
        if config.track_failures_separately && is_failure {
            let failures = self
                .client_failures
                .entry(client_id.to_string())
                .or_default();
            failures.push(now);

            if failures.len() > config.max_failures_per_window {
                return false;
            }
        }

        true
    }

    /// Clean up expired entries from rate limiting tracking
    /// 
    /// # Arguments
    /// * `now` - Current timestamp in seconds since UNIX epoch
    /// * `window_size` - Size of the tracking window in seconds
    fn cleanup_expired_entries(&mut self, now: u64, window_size: u64) {
        let cutoff = now.saturating_sub(window_size);

        // Clean up request tracking
        for requests in self.client_requests.values_mut() {
            requests.retain(|&timestamp| timestamp > cutoff);
        }
        self.client_requests
            .retain(|_, requests| !requests.is_empty());

        // Clean up failure tracking
        for failures in self.client_failures.values_mut() {
            failures.retain(|&timestamp| timestamp > cutoff);
        }
        self.client_failures
            .retain(|_, failures| !failures.is_empty());
    }

    /// Get statistics for a specific client
    /// 
    /// # Arguments
    /// * `client_id` - The client to get stats for
    /// 
    /// # Returns
    /// A tuple of (request_count, failure_count) for the current window
    pub fn get_client_stats(&self, client_id: &str) -> (usize, usize) {
        let requests = self
            .client_requests
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
        let failures = self
            .client_failures
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
        (requests, failures)
    }

    /// Get total number of tracked clients
    pub fn get_tracked_client_count(&self) -> usize {
        let mut clients = std::collections::HashSet::new();
        clients.extend(self.client_requests.keys());
        clients.extend(self.client_failures.keys());
        clients.len()
    }

    /// Get total request count across all clients
    pub fn get_total_request_count(&self) -> usize {
        self.client_requests.values().map(|v| v.len()).sum()
    }

    /// Get total failure count across all clients
    pub fn get_total_failure_count(&self) -> usize {
        self.client_failures.values().map(|v| v.len()).sum()
    }

    /// Clear all rate limiting data (useful for testing or resets)
    pub fn clear_all_data(&mut self) {
        self.client_requests.clear();
        self.client_failures.clear();
    }

    /// Get the most active clients by request count
    pub fn get_top_clients_by_requests(&self, limit: usize) -> Vec<(String, usize)> {
        let mut clients: Vec<(String, usize)> = self
            .client_requests
            .iter()
            .map(|(id, requests)| (id.clone(), requests.len()))
            .collect();
        
        clients.sort_by(|a, b| b.1.cmp(&a.1));
        clients.into_iter().take(limit).collect()
    }

    /// Get the clients with the most failures
    pub fn get_top_clients_by_failures(&self, limit: usize) -> Vec<(String, usize)> {
        let mut clients: Vec<(String, usize)> = self
            .client_failures
            .iter()
            .map(|(id, failures)| (id.clone(), failures.len()))
            .collect();
        
        clients.sort_by(|a, b| b.1.cmp(&a.1));
        clients.into_iter().take(limit).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::auth::auth_config::RateLimitingConfig;

    fn create_test_config() -> RateLimitingConfig {
        RateLimitingConfig {
            enabled: true,
            max_requests_per_window: 10,
            max_failures_per_window: 5,
            window_size_secs: 60,
            track_failures_separately: true,
        }
    }

    #[test]
    fn test_rate_limiting_basic_functionality() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Should allow requests within limit (using same client)
        for _i in 0..10 {
            assert!(limiter.check_rate_limit("client_0", &config, false));
        }

        // Should rate limit when exceeded
        assert!(!limiter.check_rate_limit("client_0", &config, false));
    }

    #[test]
    fn test_rate_limiting_disabled() {
        let mut limiter = RateLimiter::new();
        let mut config = create_test_config();
        config.enabled = false;

        // Should always allow when disabled
        for _ in 0..100 {
            assert!(limiter.check_rate_limit("client", &config, false));
        }
    }

    #[test]
    fn test_failure_tracking() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Add some failures
        for _ in 0..5 {
            assert!(limiter.check_rate_limit("client", &config, true));
        }

        // Should rate limit on next failure
        assert!(!limiter.check_rate_limit("client", &config, true));

        // But should still allow regular requests
        assert!(limiter.check_rate_limit("client", &config, false));
    }

    #[test]
    fn test_client_stats() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Make some requests and failures
        for _ in 0..3 {
            limiter.check_rate_limit("client", &config, false);
        }
        for _ in 0..2 {
            limiter.check_rate_limit("client", &config, true);
        }

        let (requests, failures) = limiter.get_client_stats("client");
        assert_eq!(requests, 5); // 3 regular + 2 failures (failures count as requests too)
        assert_eq!(failures, 2);
    }

    #[test]
    fn test_cleanup_expired_entries() {
        let mut limiter = RateLimiter::new();
        
        // Manually add old entries
        limiter.client_requests.insert("client".to_string(), vec![1000, 2000, 3000]);
        limiter.client_failures.insert("client".to_string(), vec![1500, 2500]);

        // Clean up with a cutoff that should remove old entries
        limiter.cleanup_expired_entries(4000, 1000); // Now=4000, window=1000, cutoff=3000

        let (requests, failures) = limiter.get_client_stats("client");
        assert_eq!(requests, 0); // Entry at 3000 should be removed (4000-1000=3000, so cutoff exactly at 3000)
        assert_eq!(failures, 0); // All failures should be removed
    }

    #[test]
    fn test_total_counts() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Add requests for multiple clients
        limiter.check_rate_limit("client1", &config, false);
        limiter.check_rate_limit("client1", &config, true);
        limiter.check_rate_limit("client2", &config, false);

        assert_eq!(limiter.get_tracked_client_count(), 2);
        assert_eq!(limiter.get_total_request_count(), 3);
        assert_eq!(limiter.get_total_failure_count(), 1);
    }

    #[test]
    fn test_top_clients() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Create different activity levels
        for _ in 0..5 {
            limiter.check_rate_limit("high_volume", &config, false);
        }
        for _ in 0..2 {
            limiter.check_rate_limit("medium_volume", &config, false);
        }
        limiter.check_rate_limit("low_volume", &config, false);

        let top_clients = limiter.get_top_clients_by_requests(3);
        assert_eq!(top_clients[0].0, "high_volume");
        assert_eq!(top_clients[0].1, 5);
        assert_eq!(top_clients[1].0, "medium_volume");
        assert_eq!(top_clients[1].1, 2);
    }

    #[test]
    fn test_clear_all_data() {
        let mut limiter = RateLimiter::new();
        let config = create_test_config();

        // Add some data
        limiter.check_rate_limit("client", &config, false);
        limiter.check_rate_limit("client", &config, true);

        assert_eq!(limiter.get_total_request_count(), 2);

        // Clear and verify
        limiter.clear_all_data();
        assert_eq!(limiter.get_total_request_count(), 0);
        assert_eq!(limiter.get_tracked_client_count(), 0);
    }
}