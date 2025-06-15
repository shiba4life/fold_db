//! Tests for enhanced authentication failure handling and security logging

use actix_web::http::StatusCode;
use datafold::datafold_node::signature_auth::{
    AuthenticationError, SignatureAuthConfig, SignatureVerificationState,
};
use datafold::security_types::Severity;
use uuid::Uuid;

#[tokio::test]
async fn test_authentication_error_types() {
    let correlation_id = Uuid::new_v4().to_string();

    // Test missing headers error
    let missing_headers_error = AuthenticationError::MissingHeaders {
        missing: vec!["Signature-Input".to_string(), "Signature".to_string()],
        correlation_id: correlation_id.clone(),
    };

    assert_eq!(
        missing_headers_error.http_status_code(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(missing_headers_error.severity(), Severity::Info);
    assert_eq!(missing_headers_error.correlation_id(), &correlation_id);
    assert_eq!(missing_headers_error.public_message(), "Missing required authentication headers. Please include Signature-Input and Signature headers.");

    // Test signature verification failed error
    let sig_failed_error = AuthenticationError::SignatureVerificationFailed {
        key_id: "test-key".to_string(),
        correlation_id: correlation_id.clone(),
    };

    assert_eq!(
        sig_failed_error.http_status_code(),
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(sig_failed_error.severity(), Severity::Warning);
    assert_eq!(
        sig_failed_error.public_message(),
        "Signature verification failed. Please check your signature calculation and key."
    );

    // Test nonce replay error (should be critical)
    let replay_error = AuthenticationError::NonceValidationFailed {
        nonce: "test-nonce".to_string(),
        reason: "Nonce replay detected".to_string(),
        correlation_id: correlation_id.clone(),
    };

    assert_eq!(replay_error.http_status_code(), StatusCode::UNAUTHORIZED);
    assert_eq!(replay_error.severity(), Severity::Critical);
    assert_eq!(
        replay_error.public_message(),
        "Request validation failed. Please use a unique nonce for each request."
    );

    // Test rate limit error
    let rate_limit_error = AuthenticationError::RateLimitExceeded {
        client_id: "192.168.1.100".to_string(),
        correlation_id: correlation_id.clone(),
    };

    assert_eq!(
        rate_limit_error.http_status_code(),
        StatusCode::TOO_MANY_REQUESTS
    );
    assert_eq!(rate_limit_error.severity(), Severity::Critical);
    assert_eq!(
        rate_limit_error.public_message(),
        "Rate limit exceeded. Please reduce request frequency and try again later."
    );
}

#[tokio::test]
async fn test_security_configuration_validation() {
    // Test default configuration
    let default_config = SignatureAuthConfig::default();
    assert!(default_config.validate().is_ok());
    assert!(default_config.security_logging.enabled);
    assert!(default_config.rate_limiting.enabled);
    assert!(default_config.attack_detection.enabled);

    // Test strict configuration
    let strict_config = SignatureAuthConfig::strict();
    assert!(strict_config.validate().is_ok());
    assert_eq!(strict_config.allowed_time_window_secs, 60);
    assert_eq!(strict_config.clock_skew_tolerance_secs, 5);
    assert!(!strict_config.response_security.detailed_error_messages);
    assert_eq!(strict_config.rate_limiting.max_requests_per_window, 50);
    assert_eq!(strict_config.rate_limiting.max_failures_per_window, 3);
    assert_eq!(strict_config.attack_detection.brute_force_threshold, 3);

    // Test lenient configuration
    let lenient_config = SignatureAuthConfig::lenient();
    assert!(lenient_config.validate().is_ok());
    assert!(!lenient_config.rate_limiting.enabled);
    assert!(!lenient_config.attack_detection.enabled);
    assert!(lenient_config.response_security.detailed_error_messages);
    assert_eq!(lenient_config.allowed_time_window_secs, 600);
    assert_eq!(lenient_config.clock_skew_tolerance_secs, 120);

    // Test invalid configuration
    let mut invalid_config = SignatureAuthConfig::default();
    invalid_config.allowed_time_window_secs = 0;
    assert!(invalid_config.validate().is_err());

    invalid_config.allowed_time_window_secs = 300;
    invalid_config.clock_skew_tolerance_secs = 400; // Greater than time window
    assert!(invalid_config.validate().is_err());
}

#[tokio::test]
async fn test_configuration_structures() {
    let config = SignatureAuthConfig::default();

    // Test security logging configuration
    assert!(config.security_logging.enabled);
    assert!(config.security_logging.include_correlation_ids);
    assert!(config.security_logging.include_client_info);
    assert!(config.security_logging.include_performance_metrics);
    assert!(!config.security_logging.log_successful_auth);
    assert_eq!(config.security_logging.min_severity, Severity::Info);
    assert_eq!(config.security_logging.max_log_entry_size, 8192);

    // Test rate limiting configuration
    assert!(config.rate_limiting.enabled);
    assert_eq!(config.rate_limiting.max_requests_per_window, 100);
    assert_eq!(config.rate_limiting.window_size_secs, 60);
    assert!(config.rate_limiting.track_failures_separately);
    assert_eq!(config.rate_limiting.max_failures_per_window, 10);

    // Test attack detection configuration
    assert!(config.attack_detection.enabled);
    assert_eq!(config.attack_detection.brute_force_threshold, 5);
    assert_eq!(config.attack_detection.brute_force_window_secs, 300);
    assert_eq!(config.attack_detection.replay_threshold, 3);
    assert!(config.attack_detection.enable_timing_protection);
    assert_eq!(config.attack_detection.base_response_delay_ms, 100);

    // Test response security configuration
    assert!(config.response_security.include_security_headers);
    assert!(config.response_security.consistent_timing);
    assert!(!config.response_security.detailed_error_messages);
    assert!(config.response_security.include_correlation_id);
}

#[tokio::test]
async fn test_signature_verification_state_creation() {
    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config);
    assert!(state.is_ok());

    // Test with invalid configuration
    let mut invalid_config = SignatureAuthConfig::default();
    invalid_config.nonce_ttl_secs = 0;
    let invalid_state = SignatureVerificationState::new(invalid_config);
    assert!(invalid_state.is_err());
}

#[tokio::test]
async fn test_authentication_error_display() {
    let correlation_id = Uuid::new_v4().to_string();

    let error = AuthenticationError::SignatureVerificationFailed {
        key_id: "test-key-123".to_string(),
        correlation_id: correlation_id.clone(),
    };

    let error_string = format!("{}", error);
    assert!(error_string.contains("test-key-123"));
    assert!(error_string.contains(&correlation_id));
    assert!(error_string.contains("Signature verification failed"));

    let timestamp_error = AuthenticationError::TimestampValidationFailed {
        timestamp: 1234567890,
        current_time: 1234567950,
        reason: "Too old".to_string(),
        correlation_id: correlation_id.clone(),
    };

    let timestamp_string = format!("{}", timestamp_error);
    assert!(timestamp_string.contains("1234567890"));
    assert!(timestamp_string.contains("1234567950"));
    assert!(timestamp_string.contains("Too old"));
}

#[tokio::test]
async fn test_basic_nonce_validation() {
    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config).expect("Valid config");

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test valid UUID4 nonce using the legacy method
    let valid_nonce = Uuid::new_v4().to_string();
    assert!(state.check_and_store_nonce(&valid_nonce, timestamp).is_ok());

    // Test nonce replay (should fail)
    let replay_result = state.check_and_store_nonce(&valid_nonce, timestamp);
    assert!(replay_result.is_err());
}

#[tokio::test]
async fn test_timestamp_validation() {
    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config).expect("Valid config");

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test valid timestamp (current time)
    assert!(state.validate_timestamp(now).is_ok());

    // Test slightly old timestamp (within window)
    let old_timestamp = now - 100; // 100 seconds ago
    assert!(state.validate_timestamp(old_timestamp).is_ok());

    // Test very old timestamp (outside window)
    let very_old_timestamp = now - 1000; // 1000 seconds ago
    assert!(state.validate_timestamp(very_old_timestamp).is_err());
}
