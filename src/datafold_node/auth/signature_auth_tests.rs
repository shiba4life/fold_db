//! Comprehensive test suite for DataFold authentication modules
//!
//! This module contains integration and unit tests for all authentication
//! components including nonce operations, rate limiting, attack detection,
//! and security metrics collection.

#[cfg(test)]
mod tests {
    use super::super::{
        auth_config::{AttackDetectionConfig, RateLimitingConfig, SignatureAuthConfig},
        auth_errors::AuthenticationError,
        auth_middleware::{SignatureVerificationMiddleware, SignatureVerificationState, should_skip_verification},
        auth_types::*,
        signature_auth::*,
    };
    use actix_web::{test, web, App, HttpResponse};
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json("success")
    }

    // === Configuration Tests ===

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

    // === Timestamp Validation Tests ===

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

    // === Nonce Tests ===

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
    async fn test_nonce_store_operations() {
        let mut store = NonceStore::new();
        
        // Test basic operations
        assert_eq!(store.size(), 0);
        assert!(!store.contains_nonce("test"));
        
        // Add nonces
        store.add_nonce("nonce1".to_string(), 1000);
        store.add_nonce("nonce2".to_string(), 2000);
        store.add_nonce("nonce3".to_string(), 3000);
        
        assert_eq!(store.size(), 3);
        assert!(store.contains_nonce("nonce1"));
        assert!(store.contains_nonce("nonce2"));
        assert!(store.contains_nonce("nonce3"));
        
        // Test cleanup (structure should remain valid)
        store.cleanup_expired(500);
        
        // Test size enforcement
        store.enforce_size_limit(2);
        assert!(store.size() <= 2);
    }

    // === Rate Limiting Tests ===

    #[tokio::test]
    async fn test_rate_limiting_functionality() {
        let mut limiter = RateLimiter::new();
        let config = RateLimitingConfig {
            enabled: true,
            max_requests_per_window: 5,
            max_failures_per_window: 3,
            window_size_secs: 60,
            track_failures_separately: true,
        };

        // Should allow requests within limit
        for i in 0..5 {
            assert!(limiter.check_rate_limit(&format!("client_{}", i % 2), &config, false));
        }

        // Should rate limit when exceeded for the same client
        assert!(!limiter.check_rate_limit("client_0", &config, false));

        // Test failure tracking
        for _ in 0..3 {
            assert!(limiter.check_rate_limit("failure_client", &config, true));
        }
        assert!(!limiter.check_rate_limit("failure_client", &config, true));
    }

    #[tokio::test]
    async fn test_rate_limiting_disabled() {
        let mut limiter = RateLimiter::new();
        let config = RateLimitingConfig {
            enabled: false,
            max_requests_per_window: 1,
            max_failures_per_window: 1,
            window_size_secs: 60,
            track_failures_separately: true,
        };

        // Should always allow when disabled
        for _ in 0..10 {
            assert!(limiter.check_rate_limit("client", &config, false));
        }
    }

    // === Attack Detection Tests ===

    #[tokio::test]
    async fn test_attack_detection() {
        let mut detector = AttackDetector::new();
        let config = AttackDetectionConfig {
            enabled: true,
            brute_force_threshold: 3,
            replay_threshold: 2,
            brute_force_window_secs: 300,
            enable_timing_protection: false,
            base_response_delay_ms: 0,
        };

        let client_info = ClientInfo {
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: None,
            key_id: None,
            forwarded_for: None,
        };

        let request_info = RequestInfo {
            method: "GET".to_string(),
            path: "/test".to_string(),
            query_params: None,
            content_type: None,
            content_length: None,
            signature_components: None,
        };

        // Test brute force detection
        for i in 0..config.brute_force_threshold {
            let event = SecurityEvent {
                event_id: format!("test_{}", i),
                correlation_id: format!("corr_{}", i),
                timestamp: 1000 + i as u64,
                event_type: SecurityEventType::AuthenticationFailure,
                severity: crate::security_types::Severity::Warning,
                client_info: client_info.clone(),
                request_info: request_info.clone(),
                error_details: Some(AuthenticationError::SignatureVerificationFailed {
                    key_id: "test_key".to_string(),
                    correlation_id: format!("corr_{}", i),
                }),
                metrics: SecurityMetrics::default(),
            };

            let result = detector.detect_attack_patterns("attacker", &event, &config);
            
            if i < config.brute_force_threshold - 1 {
                assert!(result.is_none());
            } else {
                assert!(result.is_some());
                let pattern = result.unwrap();
                assert!(matches!(pattern.pattern_type, AttackPatternType::BruteForce));
            }
        }
    }

    // === Security Metrics Tests ===

    #[tokio::test]
    async fn test_security_metrics_collection() {
        let collector = EnhancedSecurityMetricsCollector::new();

        // Record some attempts
        collector.record_attempt(true, 50);
        collector.record_attempt(false, 100);
        collector.record_attempt(true, 75);

        // Test metrics retrieval
        let metrics = collector.get_enhanced_security_metrics(1000);
        assert!(metrics.processing_time_ms >= 0);
        assert!(metrics.cache_hit_rate >= 0.0);
    }

    #[tokio::test]
    async fn test_performance_monitor() {
        let mut monitor = PerformanceMonitor::new();

        // Record some requests
        monitor.record_request(50);
        monitor.record_request(100);
        monitor.record_request(75);

        let avg = monitor.get_average_latency().unwrap();
        assert_eq!(avg, 75.0);
    }

    // === Signature Input Parsing Tests ===

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

    // === Middleware Integration Tests ===

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
        let error = result.unwrap_err();
        
        // Verify it's our custom auth error (missing headers)
        let custom_error = error.as_error::<CustomAuthError>();
        assert!(custom_error.is_some());
        assert!(error.to_string().contains("Missing required authentication headers"));
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

    // === Integration Tests ===

    #[tokio::test]
    async fn test_comprehensive_security_workflow() {
        let mut config = SignatureAuthConfig::default();
        config.require_uuid4_nonces = false;
        
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        let mut rate_limiter = RateLimiter::new();
        let mut attack_detector = AttackDetector::new();
        let metrics_collector = EnhancedSecurityMetricsCollector::new();
        
        let rate_config = RateLimitingConfig {
            enabled: true,
            max_requests_per_window: 10,
            max_failures_per_window: 5,
            window_size_secs: 60,
            track_failures_separately: true,
        };
        
        let attack_config = AttackDetectionConfig {
            enabled: true,
            brute_force_threshold: 5,
            replay_threshold: 3,
            brute_force_window_secs: 300,
            enable_timing_protection: false,
            base_response_delay_ms: 0,
        };

        // Simulate normal requests
        for i in 0..3 {
            let client_id = format!("client_{}", i);
            
            // Check rate limit
            assert!(rate_limiter.check_rate_limit(&client_id, &rate_config, false));
            
            // Record metrics
            metrics_collector.record_attempt(true, 50 + i * 10);
        }

        // Simulate attack scenario
        let attacker_id = "potential_attacker";
        
        // Multiple failed attempts should trigger attack detection
        for i in 0..6 {
            let client_info = ClientInfo {
                ip_address: Some(attacker_id.to_string()),
                user_agent: None,
                key_id: None,
                forwarded_for: None,
            };

            let request_info = RequestInfo {
                method: "POST".to_string(),
                path: "/api/test".to_string(),
                query_params: None,
                content_type: Some("application/json".to_string()),
                content_length: Some(100),
                signature_components: None,
            };

            let event = SecurityEvent {
                event_id: format!("attack_{}", i),
                correlation_id: format!("corr_{}", i),
                timestamp: 2000 + i as u64,
                event_type: SecurityEventType::AuthenticationFailure,
                severity: crate::security_types::Severity::Error,
                client_info,
                request_info,
                error_details: Some(AuthenticationError::SignatureVerificationFailed {
                    key_id: "attacker_key".to_string(),
                    correlation_id: format!("corr_{}", i),
                }),
                metrics: SecurityMetrics::default(),
            };

            // Check rate limit (should start failing after threshold)
            let _rate_ok = rate_limiter.check_rate_limit(attacker_id, &rate_config, true);
            
            // Check for attack patterns
            let attack_pattern = attack_detector.detect_attack_patterns(attacker_id, &event, &attack_config);
            
            // Record failed attempt
            metrics_collector.record_attempt(false, 200);
            
            if i >= attack_config.brute_force_threshold - 1 {
                assert!(attack_pattern.is_some());
                let pattern = attack_pattern.unwrap();
                assert!(matches!(pattern.pattern_type, AttackPatternType::BruteForce));
                assert_eq!(pattern.client_id, attacker_id);
            }
        }

        // Verify metrics
        let metrics = metrics_collector.get_enhanced_security_metrics(1000);
        assert!(metrics.processing_time_ms >= 0);

        // Verify rate limiting
        let (requests, failures) = rate_limiter.get_client_stats(attacker_id);
        assert!(requests > 0);
        assert!(failures > 0);
    }
}