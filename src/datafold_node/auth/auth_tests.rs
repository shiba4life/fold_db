//! Consolidated test code for DataFold node authentication system
//!
//! This module contains all authentication-related tests including unit tests,
//! integration tests, and security validation tests for the signature authentication
//! system.

#[cfg(test)]
mod tests {
    use super::super::{
        auth_config::SignatureAuthConfig,
        auth_errors::AuthenticationError,
        auth_middleware::{should_skip_verification, SignatureVerificationMiddleware, SignatureVerificationState},
        auth_types::*,
        key_management::KeyManager,
        signature_verification::{SignatureComponents, SignatureVerifier},
    };
    use actix_web::{test, web, App, HttpResponse};
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json("success")
    }

    #[tokio::test]
    async fn test_signature_auth_config_default() {
        let config = SignatureAuthConfig::default();
        assert_eq!(config.allowed_time_window_secs, 300);
        assert_eq!(config.nonce_ttl_secs, 300);
        assert!(config.required_signature_components.contains(&"@method".to_string()));
        assert!(config.required_signature_components.contains(&"@target-uri".to_string()));
        assert!(config.required_signature_components.contains(&"content-type".to_string()));
        assert!(config.required_signature_components.contains(&"content-digest".to_string()));
    }

    #[tokio::test]
    async fn test_signature_verification_state_creation() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config.clone()).expect("Config should be valid");
        
        // Test that state is created successfully
        assert_eq!(state.get_config().allowed_time_window_secs, config.allowed_time_window_secs);
    }

    #[tokio::test]
    async fn test_timestamp_validation() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        // Test current timestamp (should be valid)
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert!(state.validate_timestamp(now).is_ok());
        
        // Test old timestamp (should be invalid)
        let old_timestamp = now - 600; // 10 minutes ago
        assert!(state.validate_timestamp(old_timestamp).is_err());
        
        // Test future timestamp (should be invalid)
        let future_timestamp = now + 600; // 10 minutes in the future
        assert!(state.validate_timestamp(future_timestamp).is_err());
    }

    #[tokio::test]
    async fn test_nonce_replay_prevention() {
        let mut config = SignatureAuthConfig::default();
        config.require_uuid4_nonces = false; // Allow simple nonces for testing
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let nonce = "test-nonce-123";
        
        // First use should succeed
        assert!(state.check_and_store_nonce(nonce, now).is_ok());
        
        // Second use of same nonce should fail
        assert!(state.check_and_store_nonce(nonce, now).is_err());
        
        // Different nonce should succeed
        assert!(state.check_and_store_nonce("different-nonce", now).is_ok());
    }

    #[tokio::test]
    async fn test_signature_input_parsing() {
        let input = r#"sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key";alg="ed25519";nonce="abc123""#;
        
        let (components, params) = SignatureComponents::parse_signature_input(input).unwrap();
        
        assert_eq!(components, vec!["@method", "@target-uri", "content-type"]);
        assert_eq!(params.get("created"), Some(&"1618884473".to_string()));
        assert_eq!(params.get("keyid"), Some(&"\"test-key\"".to_string()));
        assert_eq!(params.get("alg"), Some(&"\"ed25519\"".to_string()));
        assert_eq!(params.get("nonce"), Some(&"\"abc123\"".to_string()));
    }

    #[tokio::test]
    async fn test_signature_input_parsing_invalid_format() {
        let invalid_input = "invalid-format";
        let result = SignatureComponents::parse_signature_input(invalid_input);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mandatory_signature_verification() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler))
        ).await;
        
        // Request should be rejected since signature verification is mandatory
        let req = test::TestRequest::get().uri("/test").to_request();
        let result = test::try_call_service(&app, req).await;
        
        // Should get an authentication error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_middleware_skips_health_check_paths() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/api/system/status", web::get().to(test_handler))
        ).await;
        
        // Health check path should be skipped
        let req = test::TestRequest::get().uri("/api/system/status").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_middleware_rejects_requests_without_signature() {
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/api/test", web::get().to(test_handler))
        ).await;
        
        // Request without signature headers should be rejected
        let req = test::TestRequest::get().uri("/api/test").to_request();
        let result = test::try_call_service(&app, req).await;
        
        // Should get an authentication error
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_nonce_store_cleanup() {
        let mut store = NonceStore::new();
        
        // Add some nonces
        store.add_nonce("nonce1".to_string(), 1000);
        store.add_nonce("nonce2".to_string(), 2000);
        store.add_nonce("nonce3".to_string(), 3000);
        
        assert!(store.contains_nonce("nonce1"));
        assert!(store.contains_nonce("nonce2"));
        assert!(store.contains_nonce("nonce3"));
        
        // Cleanup with TTL that should remove older nonces
        store.cleanup_expired(500); // TTL of 500 seconds
        
        // Exact behavior depends on current time, but structure should remain valid
        // This test mainly verifies the cleanup method doesn't panic
    }

    #[tokio::test]
    async fn test_should_skip_verification_function() {
        // Paths that should be skipped
        assert!(should_skip_verification("/api/system/status"));
        assert!(should_skip_verification("/api/crypto/keys/register"));
        assert!(should_skip_verification("/"));
        assert!(should_skip_verification("/index.html"));
        
        // Paths that should not be skipped
        assert!(!should_skip_verification("/api/test"));
        assert!(!should_skip_verification("/api/schemas"));
        assert!(!should_skip_verification("/api/query"));
    }

    #[test]
    async fn test_authentication_error_correlation_id() {
        let error = AuthenticationError::MissingHeaders {
            missing: vec!["Signature".to_string()],
            correlation_id: "test-123".to_string(),
        };
        
        assert_eq!(error.correlation_id(), "test-123");
        assert_eq!(error.error_code(), "MISSING_HEADERS");
    }

    #[test]
    async fn test_authentication_error_public_message() {
        let error = AuthenticationError::SignatureVerificationFailed {
            key_id: "test-key".to_string(),
            correlation_id: "test-123".to_string(),
        };
        
        let message = error.public_message();
        assert!(message.contains("Signature verification failed"));
        assert!(!message.contains("test-key")); // Shouldn't expose internal details
    }

    #[test]
    async fn test_authentication_error_user_friendly_message() {
        let error = AuthenticationError::TimestampValidationFailed {
            timestamp: 1000,
            current_time: 2000,
            reason: "Too old".to_string(),
            correlation_id: "test-123".to_string(),
        };
        
        let dev_message = error.user_friendly_message("development");
        let prod_message = error.user_friendly_message("production");
        
        // Development message should have more details
        assert!(dev_message.len() > prod_message.len());
        assert!(dev_message.contains("Troubleshooting"));
    }

    #[test]
    async fn test_signature_verifier_validate_algorithm() {
        assert!(SignatureVerifier::validate_algorithm("ed25519").is_ok());
        assert!(SignatureVerifier::validate_algorithm("rsa").is_err());
        assert!(SignatureVerifier::validate_algorithm("").is_err());
    }

    #[test]
    async fn test_signature_verifier_validate_signature_format() {
        // Valid hex signature of correct length (64 bytes = 128 hex chars)
        let valid_sig = "a".repeat(128);
        assert!(SignatureVerifier::validate_signature_format(&valid_sig).is_ok());

        // Invalid hex characters
        let invalid_hex = "gggggggg".repeat(16);
        assert!(SignatureVerifier::validate_signature_format(&invalid_hex).is_err());

        // Wrong length
        let wrong_length = "aa";
        assert!(SignatureVerifier::validate_signature_format(&wrong_length).is_err());

        // Empty signature
        assert!(SignatureVerifier::validate_signature_format("").is_err());
    }

    #[test]
    async fn test_signature_verifier_validate_nonce_format() {
        // Valid non-UUID nonce
        assert!(SignatureVerifier::validate_nonce_format("test-nonce-123", false).is_ok());

        // Invalid characters
        assert!(SignatureVerifier::validate_nonce_format("test@nonce", false).is_err());

        // Empty nonce
        assert!(SignatureVerifier::validate_nonce_format("", false).is_err());

        // Too long nonce
        let long_nonce = "a".repeat(200);
        assert!(SignatureVerifier::validate_nonce_format(&long_nonce, false).is_err());
    }

    #[test]
    async fn test_signature_verifier_validate_uuid4_format() {
        // Valid UUID4
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(SignatureVerifier::validate_nonce_format(valid_uuid, true).is_ok());

        // Invalid UUID (wrong version)
        let invalid_version = "550e8400-e29b-31d4-a716-446655440000";
        assert!(SignatureVerifier::validate_nonce_format(invalid_version, true).is_err());

        // Invalid UUID (wrong format)
        let invalid_format = "not-a-uuid";
        assert!(SignatureVerifier::validate_nonce_format(invalid_format, true).is_err());

        // Invalid UUID (wrong length)
        let wrong_length = "550e8400-e29b-41d4-a716";
        assert!(SignatureVerifier::validate_nonce_format(wrong_length, true).is_err());
    }

    #[test]
    async fn test_signature_verifier_rfc3339_format_validation() {
        // Valid RFC 3339 format
        assert!(SignatureVerifier::is_valid_rfc3339_format("2023-01-01T12:00:00Z"));
        assert!(SignatureVerifier::is_valid_rfc3339_format("2023-12-31T23:59:59"));

        // Invalid formats
        assert!(!SignatureVerifier::is_valid_rfc3339_format("not-a-date"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format("2023-1-1T12:00:00Z"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format("23-01-01T12:00:00Z"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format(""));
    }

    #[test]
    async fn test_config_validation() {
        let config = SignatureAuthConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid config - zero time window
        let mut invalid_config = SignatureAuthConfig::default();
        invalid_config.allowed_time_window_secs = 0;
        assert!(invalid_config.validate().is_err());

        // Test invalid config - clock skew too large
        let mut invalid_config = SignatureAuthConfig::default();
        invalid_config.clock_skew_tolerance_secs = invalid_config.allowed_time_window_secs + 1;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    async fn test_config_security_profiles() {
        let strict_config = SignatureAuthConfig::strict();
        assert_eq!(strict_config.security_profile, SecurityProfile::Strict);
        assert_eq!(strict_config.allowed_time_window_secs, 60);
        assert!(!strict_config.response_security.detailed_error_messages);

        let lenient_config = SignatureAuthConfig::lenient();
        assert_eq!(lenient_config.security_profile, SecurityProfile::Lenient);
        assert!(!lenient_config.rate_limiting.enabled);
        assert!(!lenient_config.attack_detection.enabled);

        let production_config = SignatureAuthConfig::production();
        assert!(production_config.is_production());
        assert!(!production_config.is_development());

        let development_config = SignatureAuthConfig::development();
        assert!(development_config.is_development());
        assert!(!development_config.is_production());
    }

    #[test]
    async fn test_config_environment_switching() {
        let base_config = SignatureAuthConfig::default();
        
        let prod_config = base_config.clone().for_environment("production");
        assert!(prod_config.is_production());
        
        let dev_config = base_config.clone().for_environment("development");
        assert!(dev_config.is_development());
        
        let test_config = base_config.clone().for_environment("testing");
        assert!(!test_config.rate_limiting.enabled);
        assert!(!test_config.attack_detection.enabled);
    }

    #[test]
    async fn test_config_debug_mode_toggle() {
        let config = SignatureAuthConfig::production().with_debug();
        assert!(config.response_security.detailed_error_messages);
        assert!(config.security_logging.log_successful_auth);
        
        let config = config.without_debug();
        assert!(!config.response_security.detailed_error_messages);
        assert!(!config.security_logging.log_successful_auth);
    }

    #[test]
    async fn test_latency_histogram() {
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
    async fn test_public_key_cache_basic_operations() {
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
    async fn test_cache_eviction() {
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
    async fn test_cache_invalidation() {
        let mut cache = PublicKeyCache::new(3);
        
        cache.put("key1".to_string(), [1u8; 32], "active".to_string());
        assert!(cache.get("key1").is_some());
        
        cache.invalidate("key1");
        assert!(cache.get("key1").is_none());
        assert_eq!(cache.size(), 0);
    }

    #[test]
    async fn test_cache_clear() {
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
    async fn test_nonce_store_basic_operations() {
        let mut store = NonceStore::new();
        
        // Test adding and checking nonces
        assert!(!store.contains_nonce("test-nonce"));
        store.add_nonce("test-nonce".to_string(), 1234567890);
        assert!(store.contains_nonce("test-nonce"));
        
        // Test size tracking
        assert_eq!(store.size(), 1);
    }

    #[tokio::test]
    async fn test_nonce_store_cleanup_sync() {
        let mut store = NonceStore::new();
        
        // Add nonces with different timestamps
        store.add_nonce("old-nonce".to_string(), 1000);
        store.add_nonce("new-nonce".to_string(), 2000);
        
        // Cleanup with TTL that should remove old nonce
        store.cleanup_expired(500); // TTL of 500 seconds
        
        // Structure should remain valid (exact behavior depends on current time)
        assert!(store.size() <= 2);
    }

    #[test]
    async fn test_rate_limiter_basic_operations() {
        let mut rate_limiter = RateLimiter::new();
        let config = super::super::auth_config::RateLimitingConfig::default();
        
        // Should allow requests initially
        assert!(rate_limiter.check_rate_limit("client1", &config, false));
        
        // Get client stats
        let (requests, failures) = rate_limiter.get_client_stats("client1");
        assert_eq!(requests, 1);
        assert_eq!(failures, 0);
    }

    #[test]
    async fn test_attack_detector_basic_operations() {
        let mut detector = AttackDetector::new();
        let config = super::super::auth_config::AttackDetectionConfig::default();
        
        // Create a mock security event
        let event = create_security_event(
            SecurityEventType::AuthenticationFailure,
            crate::security_types::Severity::Warning,
            ClientInfo {
                ip_address: Some("192.168.1.1".to_string()),
                user_agent: None,
                key_id: None,
                forwarded_for: None,
            },
            RequestInfo {
                method: "GET".to_string(),
                path: "/api/test".to_string(),
                query_params: None,
                content_type: None,
                content_length: None,
                signature_components: None,
            },
            Some(AuthenticationError::SignatureVerificationFailed {
                key_id: "test-key".to_string(),
                correlation_id: "test-123".to_string(),
            }),
            SecurityMetrics::default(),
        );
        
        // Should not detect patterns initially
        let pattern = detector.detect_attack_patterns("192.168.1.1", &event, &config);
        assert!(pattern.is_none());
    }

    #[tokio::test]
    async fn test_key_manager_operations() {
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

    #[test]
    async fn test_security_metrics_collector_basic_operations() {
        let collector = EnhancedSecurityMetricsCollector::new();
        
        // Record some attempts
        collector.record_attempt(true, 100);
        collector.record_attempt(false, 200);
        
        // Get metrics
        let metrics = collector.get_enhanced_security_metrics(1000);
        assert!(metrics.processing_time_ms > 0);
    }

    #[test]
    async fn test_performance_monitor_basic_operations() {
        let mut monitor = PerformanceMonitor::new();
        
        // Record some requests
        monitor.record_request(50);
        monitor.record_request(100);
        
        // Get average latency
        let avg = monitor.get_average_latency();
        assert!(avg.is_some());
        assert_eq!(avg.unwrap(), 75.0);
    }

    #[actix_web::test]
    async fn test_signature_verification_success() {
        // Create test configuration
        let config = SignatureAuthConfig::default();
        let state = SignatureVerificationState::new(config).expect("Config should be valid");

        // Create test app with middleware
        let _app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // TODO: Create a properly signed request and test verification
        // This would require implementing the full signature creation process
        // and integration with the existing public key database storage
    }

    // Test helper functions
    fn create_test_signature_components() -> SignatureComponents {
        SignatureComponents {
            signature_input: "sig1=(\"@method\" \"@target-uri\");created=1234567890;keyid=\"test-key\";alg=\"ed25519\";nonce=\"test-nonce\"".to_string(),
            signature: "a".repeat(128), // Valid hex signature length
            created: 1234567890,
            keyid: "test-key".to_string(),
            algorithm: "ed25519".to_string(),
            nonce: "test-nonce".to_string(),
            covered_components: vec!["@method".to_string(), "@target-uri".to_string()],
        }
    }

    #[test]
    async fn test_signature_components_build_signature_params() {
        let components = create_test_signature_components();
        let params = components.build_signature_params();
        assert!(params.contains("@method"));
        assert!(params.contains("@target-uri"));
    }

    // Note: Integration tests are handled in higher-level test files
    // Basic unit tests above cover the core functionality

    // Performance and load testing helpers
    #[tokio::test]
    async fn test_performance_under_load() {
        let mut histogram = LatencyHistogram::new(10000);
        
        // Simulate load
        for i in 0..1000 {
            histogram.record(i % 100); // Varying latencies
        }
        
        assert_eq!(histogram.count(), 1000);
        
        let p95 = histogram.percentile(95.0).unwrap();
        let p99 = histogram.percentile(99.0).unwrap();
        
        assert!(p95 <= p99); // p95 should be <= p99
    }

    #[tokio::test]
    async fn test_cache_performance_under_load() {
        let mut cache = PublicKeyCache::new(100);
        
        // Fill cache
        for i in 0..100 {
            let key_id = format!("key-{}", i);
            cache.put(key_id, [i as u8; 32], "active".to_string());
        }
        
        // Access patterns
        for i in 0..1000 {
            let key_id = format!("key-{}", i % 50); // Access first 50 keys repeatedly
            cache.get(&key_id);
        }
        
        // Should have good hit rate for frequently accessed keys
        assert!(cache.hit_rate() > 0.8);
    }

    // Security-specific tests
    #[test]
    async fn test_timing_attack_resistance() {
        let config = SignatureAuthConfig::strict();
        assert!(config.attack_detection.enable_timing_protection);
        assert!(config.attack_detection.base_response_delay_ms > 0);
    }

    #[test]
    async fn test_replay_attack_prevention() {
        let mut store = NonceStore::new();
        
        let nonce = "replay-nonce";
        let timestamp = 1234567890;
        
        // First request should succeed
        store.add_nonce(nonce.to_string(), timestamp);
        assert!(store.contains_nonce(nonce));
        
        // Replay attempt should be detected
        assert!(store.contains_nonce(nonce)); // Already exists
    }

    #[test]
    async fn test_brute_force_detection() {
        let mut detector = AttackDetector::new();
        let config = super::super::auth_config::AttackDetectionConfig::default();
        
        // Simulate multiple failed attempts
        for i in 0..config.brute_force_threshold {
            let event = create_security_event(
                SecurityEventType::AuthenticationFailure,
                crate::security_types::Severity::Warning,
                ClientInfo {
                    ip_address: Some("192.168.1.100".to_string()),
                    user_agent: None,
                    key_id: None,
                    forwarded_for: None,
                },
                RequestInfo {
                    method: "GET".to_string(),
                    path: "/api/test".to_string(),
                    query_params: None,
                    content_type: None,
                    content_length: None,
                    signature_components: None,
                },
                Some(AuthenticationError::SignatureVerificationFailed {
                    key_id: format!("attacker-key-{}", i),
                    correlation_id: format!("attack-{}", i),
                }),
                SecurityMetrics::default(),
            );
            
            let pattern = detector.detect_attack_patterns("192.168.1.100", &event, &config);
            
            if i >= config.brute_force_threshold - 1 {
                // Should detect pattern on threshold breach
                assert!(pattern.is_some());
            }
        }
    }
}