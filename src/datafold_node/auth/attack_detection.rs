//! Attack pattern detection for DataFold authentication
//!
//! This module provides sophisticated attack detection capabilities including
//! brute force detection, replay attack monitoring, and suspicious pattern analysis.
//! It helps identify potential security threats and enables proactive response.

use super::auth_config::AttackDetectionConfig;
use super::auth_errors::AuthenticationError;
use super::auth_types::{
    AttackDetector, AttackPatternType, SecurityEvent, SuspiciousPattern,
};

impl AttackDetector {
    /// Detect attack patterns based on security events
    /// 
    /// # Arguments
    /// * `client_id` - Unique identifier for the client
    /// * `event` - Security event to analyze
    /// * `config` - Attack detection configuration
    /// 
    /// # Returns
    /// * `Some(SuspiciousPattern)` if an attack pattern is detected
    /// * `None` if no suspicious activity is found
    pub fn detect_attack_patterns(
        &mut self,
        client_id: &str,
        event: &SecurityEvent,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        if !config.enabled {
            return None;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Clean up old entries
        self.cleanup_expired_patterns(now, config.brute_force_window_secs);

        match &event.error_details {
            Some(auth_error) => match auth_error {
                AuthenticationError::SignatureVerificationFailed { .. }
                | AuthenticationError::TimestampValidationFailed { .. }
                | AuthenticationError::PublicKeyLookupFailed { .. } => {
                    self.track_brute_force_attempt(client_id, now, config)
                }
                AuthenticationError::NonceValidationFailed { .. } => {
                    self.track_replay_attempt(client_id, now, config)
                }
                _ => None,
            },
            None => None,
        }
    }

    /// Track and analyze brute force attempts
    /// 
    /// # Arguments
    /// * `client_id` - Client making the attempts
    /// * `now` - Current timestamp
    /// * `config` - Attack detection configuration
    /// 
    /// # Returns
    /// * `Some(SuspiciousPattern)` if brute force threshold is exceeded
    fn track_brute_force_attempt(
        &mut self,
        client_id: &str,
        now: u64,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        let attempts = self
            .brute_force_attempts
            .entry(client_id.to_string())
            .or_default();
        attempts.push(now);

        if attempts.len() >= config.brute_force_threshold {
            Some(SuspiciousPattern {
                pattern_type: AttackPatternType::BruteForce,
                detection_time: now,
                severity_score: (attempts.len() as f64 / config.brute_force_threshold as f64)
                    * 10.0,
                client_id: client_id.to_string(),
            })
        } else {
            None
        }
    }

    /// Track and analyze replay attempts
    /// 
    /// # Arguments
    /// * `client_id` - Client making the attempts
    /// * `now` - Current timestamp
    /// * `config` - Attack detection configuration
    /// 
    /// # Returns
    /// * `Some(SuspiciousPattern)` if replay threshold is exceeded
    fn track_replay_attempt(
        &mut self,
        client_id: &str,
        now: u64,
        config: &AttackDetectionConfig,
    ) -> Option<SuspiciousPattern> {
        let attempts = self
            .replay_attempts
            .entry(client_id.to_string())
            .or_default();
        attempts.push(now);

        if attempts.len() >= config.replay_threshold {
            Some(SuspiciousPattern {
                pattern_type: AttackPatternType::ReplayAttack,
                detection_time: now,
                severity_score: (attempts.len() as f64 / config.replay_threshold as f64) * 15.0, // Higher severity
                client_id: client_id.to_string(),
            })
        } else {
            None
        }
    }

    /// Clean up expired attack pattern data
    /// 
    /// # Arguments
    /// * `now` - Current timestamp
    /// * `window_size` - Size of the tracking window in seconds
    fn cleanup_expired_patterns(&mut self, now: u64, window_size: u64) {
        let cutoff = now.saturating_sub(window_size);

        for attempts in self.brute_force_attempts.values_mut() {
            attempts.retain(|&timestamp| timestamp > cutoff);
        }
        self.brute_force_attempts
            .retain(|_, attempts| !attempts.is_empty());

        for attempts in self.replay_attempts.values_mut() {
            attempts.retain(|&timestamp| timestamp > cutoff);
        }
        self.replay_attempts
            .retain(|_, attempts| !attempts.is_empty());
    }

    /// Get attack statistics for a specific client
    /// 
    /// # Arguments
    /// * `client_id` - Client to get stats for
    /// 
    /// # Returns
    /// A tuple of (brute_force_attempts, replay_attempts) in current window
    pub fn get_client_attack_stats(&self, client_id: &str) -> (usize, usize) {
        let brute_force = self
            .brute_force_attempts
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
        let replay = self
            .replay_attempts
            .get(client_id)
            .map(|v| v.len())
            .unwrap_or(0);
        (brute_force, replay)
    }

    /// Get total attack attempt counts across all clients
    pub fn get_total_attack_stats(&self) -> (usize, usize) {
        let total_brute_force = self.brute_force_attempts.values().map(|v| v.len()).sum();
        let total_replay = self.replay_attempts.values().map(|v| v.len()).sum();
        (total_brute_force, total_replay)
    }

    /// Get the most aggressive attackers by brute force attempts
    pub fn get_top_brute_force_attackers(&self, limit: usize) -> Vec<(String, usize)> {
        let mut attackers: Vec<(String, usize)> = self
            .brute_force_attempts
            .iter()
            .map(|(id, attempts)| (id.clone(), attempts.len()))
            .collect();
        
        attackers.sort_by(|a, b| b.1.cmp(&a.1));
        attackers.into_iter().take(limit).collect()
    }

    /// Get the most aggressive attackers by replay attempts
    pub fn get_top_replay_attackers(&self, limit: usize) -> Vec<(String, usize)> {
        let mut attackers: Vec<(String, usize)> = self
            .replay_attempts
            .iter()
            .map(|(id, attempts)| (id.clone(), attempts.len()))
            .collect();
        
        attackers.sort_by(|a, b| b.1.cmp(&a.1));
        attackers.into_iter().take(limit).collect()
    }

    /// Check if a client is currently under suspicion
    pub fn is_client_suspicious(
        &self,
        client_id: &str,
        config: &AttackDetectionConfig,
    ) -> bool {
        let (brute_force, replay) = self.get_client_attack_stats(client_id);
        
        brute_force >= config.brute_force_threshold || replay >= config.replay_threshold
    }

    /// Calculate a risk score for a client (0.0 to 100.0)
    pub fn calculate_client_risk_score(
        &self,
        client_id: &str,
        config: &AttackDetectionConfig,
    ) -> f64 {
        let (brute_force, replay) = self.get_client_attack_stats(client_id);
        
        let brute_force_score = if config.brute_force_threshold > 0 {
            (brute_force as f64 / config.brute_force_threshold as f64) * 50.0
        } else {
            0.0
        };
        
        let replay_score = if config.replay_threshold > 0 {
            (replay as f64 / config.replay_threshold as f64) * 50.0
        } else {
            0.0
        };
        
        (brute_force_score + replay_score).min(100.0)
    }

    /// Clear all attack detection data (useful for testing or resets)
    pub fn clear_all_data(&mut self) {
        self.brute_force_attempts.clear();
        self.replay_attempts.clear();
        self._suspicious_patterns.clear();
    }

    /// Get total number of clients being tracked for attacks
    pub fn get_tracked_attacker_count(&self) -> usize {
        let mut clients = std::collections::HashSet::new();
        clients.extend(self.brute_force_attempts.keys());
        clients.extend(self.replay_attempts.keys());
        clients.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::auth::auth_config::AttackDetectionConfig;
    use crate::datafold_node::auth::auth_types::{SecurityEvent, SecurityEventType, ClientInfo, RequestInfo, SecurityMetrics};
    use crate::security_types::Severity;

    fn create_test_config() -> AttackDetectionConfig {
        AttackDetectionConfig {
            enabled: true,
            brute_force_threshold: 5,
            replay_threshold: 3,
            brute_force_window_secs: 300,
            enable_timing_protection: false,
            base_response_delay_ms: 0,
        }
    }

    #[test]
    fn test_attack_detection_basic() {
        let mut detector = AttackDetector::new();
        let config = create_test_config();

        // Test basic functionality without requiring complex event creation
        let (brute_force, replay) = detector.get_client_attack_stats("test_client");
        assert_eq!(brute_force, 0);
        assert_eq!(replay, 0);

        // Test total stats
        let (total_brute_force, total_replay) = detector.get_total_attack_stats();
        assert_eq!(total_brute_force, 0);
        assert_eq!(total_replay, 0);

        // Test risk score calculation
        let risk_score = detector.calculate_client_risk_score("test_client", &config);
        assert_eq!(risk_score, 0.0);

        // Test suspicion check
        assert!(!detector.is_client_suspicious("test_client", &config));

        // Test tracked attacker count
        assert_eq!(detector.get_tracked_attacker_count(), 0);
    }

    #[test]
    fn test_clear_all_data() {
        let mut detector = AttackDetector::new();

        // Add some test data manually
        detector.brute_force_attempts.insert("test".to_string(), vec![1000, 2000]);
        detector.replay_attempts.insert("test".to_string(), vec![1500]);

        assert_eq!(detector.get_tracked_attacker_count(), 1);

        // Clear and verify
        detector.clear_all_data();
        assert_eq!(detector.get_tracked_attacker_count(), 0);
        let (total_brute_force, total_replay) = detector.get_total_attack_stats();
        assert_eq!(total_brute_force, 0);
        assert_eq!(total_replay, 0);
    }

    #[test]
    fn test_top_attackers() {
        let mut detector = AttackDetector::new();

        // Add different levels of attack activity manually
        detector.brute_force_attempts.insert("high_volume".to_string(), vec![1, 2, 3, 4, 5]);
        detector.brute_force_attempts.insert("low_volume".to_string(), vec![1, 2]);

        let top_attackers = detector.get_top_brute_force_attackers(2);
        assert_eq!(top_attackers[0].0, "high_volume");
        assert_eq!(top_attackers[0].1, 5);
        assert_eq!(top_attackers[1].0, "low_volume");
        assert_eq!(top_attackers[1].1, 2);
    }
}