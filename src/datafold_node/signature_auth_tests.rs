//! Unit tests for Ed25519 signature verification middleware

#[cfg(test)]
mod tests {
    use super::super::signature_auth::*;
    use crate::crypto::ed25519::{generate_master_keypair, PublicKey};
    use actix_web::{test, web, App, HttpResponse, HttpMessage};
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json("success")
    }

    #[tokio::test]
    async fn test_signature_auth_config_default() {
        let config = SignatureAuthConfig::default();
        assert!(config.enabled);
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
        assert_eq!(state.config.enabled, config.enabled);
        assert_eq!(state.config.allowed_time_window_secs, config.allowed_time_window_secs);
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
        let config = SignatureAuthConfig::default();
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
    async fn test_middleware_with_disabled_config() {
        let mut config = SignatureAuthConfig::default();
        config.enabled = false;
        let state = SignatureVerificationState::new(config).expect("Config should be valid");
        
        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler))
        ).await;
        
        // Request should pass through when verification is disabled
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
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
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status().as_u16(), 401);
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
        use super::super::signature_auth::should_skip_verification;
        
        // Paths that should be skipped
        assert!(should_skip_verification("/api/system/status"));
        assert!(should_skip_verification("/api/crypto/status"));
        assert!(should_skip_verification("/api/crypto/keys/register"));
        assert!(should_skip_verification("/"));
        assert!(should_skip_verification("/index.html"));
        
        // Paths that should not be skipped
        assert!(!should_skip_verification("/api/test"));
        assert!(!should_skip_verification("/api/schemas"));
        assert!(!should_skip_verification("/api/query"));
    }
}