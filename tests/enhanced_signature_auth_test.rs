//! Enhanced tests for timestamp and nonce validation in signature authentication middleware

use datafold::datafold_node::signature_auth::{
    NonceStoreStats, SecurityProfile, SignatureAuthConfig, SignatureVerificationState,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_security_profiles() {
    // Test strict profile
    let strict_config = SignatureAuthConfig::strict();
    assert_eq!(strict_config.security_profile, SecurityProfile::Strict);
    assert_eq!(strict_config.allowed_time_window_secs, 60);
    assert_eq!(strict_config.clock_skew_tolerance_secs, 5);
    assert_eq!(strict_config.max_future_timestamp_secs, 10);
    assert!(strict_config.enforce_rfc3339_timestamps);
    assert!(strict_config.require_uuid4_nonces);

    // Test lenient profile
    let lenient_config = SignatureAuthConfig::lenient();
    assert_eq!(lenient_config.security_profile, SecurityProfile::Lenient);
    assert_eq!(lenient_config.allowed_time_window_secs, 600);
    assert_eq!(lenient_config.clock_skew_tolerance_secs, 120);
    assert_eq!(lenient_config.max_future_timestamp_secs, 300);
    assert!(!lenient_config.enforce_rfc3339_timestamps);
    assert!(!lenient_config.require_uuid4_nonces);
}

#[tokio::test]
async fn test_config_validation() {
    // Test valid configuration
    let valid_config = SignatureAuthConfig::default();
    assert!(valid_config.validate().is_ok());

    // Test invalid configurations
    let mut invalid_config = SignatureAuthConfig::default();

    // Zero time window should fail
    invalid_config.allowed_time_window_secs = 0;
    assert!(invalid_config.validate().is_err());

    // Reset and test zero nonce TTL
    invalid_config = SignatureAuthConfig::default();
    invalid_config.nonce_ttl_secs = 0;
    assert!(invalid_config.validate().is_err());

    // Reset and test zero store size
    invalid_config = SignatureAuthConfig::default();
    invalid_config.max_nonce_store_size = 0;
    assert!(invalid_config.validate().is_err());

    // Reset and test clock skew > time window
    invalid_config = SignatureAuthConfig::default();
    invalid_config.clock_skew_tolerance_secs = 400; // > 300 default window
    assert!(invalid_config.validate().is_err());
}

#[tokio::test]
async fn test_enhanced_timestamp_validation() {
    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test current timestamp (should be valid)
    assert!(state.validate_timestamp(now).is_ok());

    // Test small future timestamp within clock skew tolerance (should be valid)
    let small_future = now + 15; // Within 30s default tolerance
    assert!(state.validate_timestamp(small_future).is_ok());

    // Test large future timestamp beyond tolerance (should be invalid)
    let large_future = now + 120; // Beyond 60s max future + 30s tolerance
    assert!(state.validate_timestamp(large_future).is_err());

    // Test old timestamp beyond effective window (should be invalid)
    let old_timestamp = now - 400; // Beyond 300s window + 30s tolerance
    assert!(state.validate_timestamp(old_timestamp).is_err());

    // Test timestamp within effective window (should be valid)
    let within_window = now - 320; // Within 300s + 30s effective window
    assert!(state.validate_timestamp(within_window).is_ok());
}

#[tokio::test]
async fn test_nonce_format_validation() {
    let mut config = SignatureAuthConfig::default();
    config.require_uuid4_nonces = true;
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test valid UUID4
    let valid_uuid4 = "550e8400-e29b-41d4-a716-446655440000";
    assert!(state.check_and_store_nonce(valid_uuid4, now).is_ok());

    // Test invalid UUID format
    let invalid_uuid = "not-a-uuid";
    assert!(state.check_and_store_nonce(invalid_uuid, now).is_err());

    // Test wrong UUID version (version 1)
    let uuid_v1 = "550e8400-e29b-11d4-a716-446655440000"; // version 1
    assert!(state.check_and_store_nonce(uuid_v1, now).is_err());

    // Test valid UUID4 with different format
    let another_uuid4 = "123e4567-e89b-42d3-a456-426614174000";
    assert!(state.check_and_store_nonce(another_uuid4, now).is_ok());
}

#[tokio::test]
async fn test_nonce_format_validation_disabled() {
    let mut config = SignatureAuthConfig::default();
    config.require_uuid4_nonces = false;
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test alphanumeric nonces (should be valid)
    assert!(state.check_and_store_nonce("simple-nonce-123", now).is_ok());
    assert!(state
        .check_and_store_nonce("another_nonce_456", now)
        .is_ok());

    // Test empty nonce (should be invalid)
    assert!(state.check_and_store_nonce("", now).is_err());

    // Test nonce with invalid characters (should be invalid)
    assert!(state
        .check_and_store_nonce("nonce@with#special$chars", now)
        .is_err());

    // Test overly long nonce (should be invalid)
    let long_nonce = "a".repeat(129);
    assert!(state.check_and_store_nonce(&long_nonce, now).is_err());
}

#[tokio::test]
async fn test_nonce_store_size_limits() {
    let mut config = SignatureAuthConfig::default();
    config.max_nonce_store_size = 3; // Very small for testing
    config.require_uuid4_nonces = false; // Use simple nonces for testing
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Add nonces up to the limit
    assert!(state.check_and_store_nonce("nonce1", now).is_ok());
    assert!(state.check_and_store_nonce("nonce2", now).is_ok());
    assert!(state.check_and_store_nonce("nonce3", now).is_ok());

    // Add one more nonce (should trigger size limit enforcement)
    assert!(state.check_and_store_nonce("nonce4", now).is_ok());

    // Verify stats
    let stats = state.get_nonce_store_stats().expect("Should get stats");
    assert_eq!(stats.max_capacity, 3);
    assert!(stats.total_nonces <= 3); // Should be at or below limit due to cleanup
}

#[tokio::test]
async fn test_nonce_store_statistics() {
    let mut config = SignatureAuthConfig::default();
    config.require_uuid4_nonces = false; // Allow simple nonces for this test
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Initially empty
    let initial_stats = state.get_nonce_store_stats().expect("Should get stats");
    assert_eq!(initial_stats.total_nonces, 0);
    assert_eq!(initial_stats.oldest_nonce_age_secs, None);

    // Add some nonces
    assert!(state
        .check_and_store_nonce("test-nonce-1", now - 10)
        .is_ok());
    assert!(state.check_and_store_nonce("test-nonce-2", now - 5).is_ok());
    assert!(state.check_and_store_nonce("test-nonce-3", now).is_ok());

    // Check updated stats
    let updated_stats = state.get_nonce_store_stats().expect("Should get stats");
    assert_eq!(updated_stats.total_nonces, 3);
    assert_eq!(updated_stats.max_capacity, 10000); // Default capacity
    assert!(updated_stats.oldest_nonce_age_secs.is_some());
    assert!(updated_stats.oldest_nonce_age_secs.unwrap() >= 10); // At least 10 seconds old
}

#[tokio::test]
async fn test_replay_attack_prevention() {
    let mut config = SignatureAuthConfig::default();
    config.require_uuid4_nonces = false; // Allow simple nonces for this test
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let nonce = "replay-test-nonce";

    // First use should succeed
    assert!(state.check_and_store_nonce(nonce, now).is_ok());

    // Immediate replay should fail
    assert!(state.check_and_store_nonce(nonce, now).is_err());

    // Replay with different timestamp should also fail
    assert!(state.check_and_store_nonce(nonce, now + 10).is_err());

    // Different nonce should succeed
    assert!(state.check_and_store_nonce("different-nonce", now).is_ok());
}

#[tokio::test]
async fn test_rfc3339_timestamp_format_validation() {
    let mut config = SignatureAuthConfig::default();
    config.enforce_rfc3339_timestamps = false; // Test basic mode first
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    // Test simple unix timestamp parsing
    assert!(state.validate_rfc3339_timestamp("1618884473").is_ok());
    assert!(state.validate_rfc3339_timestamp("invalid").is_err());

    // Test with RFC 3339 enforcement enabled
    let mut strict_config = SignatureAuthConfig::default();
    strict_config.enforce_rfc3339_timestamps = true;
    let strict_state =
        SignatureVerificationState::new(strict_config).expect("Config should be valid");

    // Valid RFC 3339 format should pass basic validation but fail parsing (not implemented)
    let result = strict_state.validate_rfc3339_timestamp("2021-04-20T14:27:53Z");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not implemented"));

    // Invalid format should fail validation
    assert!(strict_state
        .validate_rfc3339_timestamp("invalid-format")
        .is_err());
}

#[tokio::test]
async fn test_clock_skew_tolerance() {
    let mut config = SignatureAuthConfig::default();
    config.allowed_time_window_secs = 60; // 1 minute
    config.clock_skew_tolerance_secs = 30; // 30 seconds tolerance
    let state = SignatureVerificationState::new(config).expect("Config should be valid");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // Test timestamp exactly at the edge of window + tolerance
    let edge_timestamp = now - 90; // 60s window + 30s tolerance = 90s total
    assert!(state.validate_timestamp(edge_timestamp).is_ok());

    // Test timestamp just beyond window + tolerance
    let beyond_edge = now - 91;
    assert!(state.validate_timestamp(beyond_edge).is_err());

    // Test future timestamp within tolerance
    let future_within_tolerance = now + 25; // Within 30s tolerance
    assert!(state.validate_timestamp(future_within_tolerance).is_ok());
}
