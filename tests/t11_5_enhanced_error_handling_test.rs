//! Tests for T11.5: Enhanced Error Handling & User Experience for PBI-11

use datafold::datafold_node::signature_auth::{
    AuthenticationError, SecurityEventSeverity, SignatureAuthConfig, SignatureVerificationState,
};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_error_message_improvements() {
    let error = AuthenticationError::MissingHeaders {
        missing: vec!["Signature-Input".to_string(), "Signature".to_string()],
        correlation_id: "test-123".to_string(),
    };

    // Test public message with guidance
    let public_msg = error.public_message();
    assert!(public_msg.contains("Missing required authentication headers"));
    assert!(public_msg.contains("Please include"));

    // Test troubleshooting guidance
    let guidance = error.get_troubleshooting_guidance();
    assert!(guidance.contains("Signature-Input"));
    assert!(guidance.contains("lowercase"));
    assert!(guidance.contains("HTTP client"));

    // Test suggested actions
    let actions = error.get_suggested_actions();
    assert!(!actions.is_empty());
    assert!(actions.iter().any(|action| action.contains("Include both")));

    // Test documentation link
    let doc_link = error.get_documentation_link();
    assert!(doc_link.contains("docs.datafold.dev"));
    assert!(doc_link.contains("signature-auth"));
}

#[tokio::test]
async fn test_environment_aware_error_messages() {
    let error = AuthenticationError::SignatureVerificationFailed {
        key_id: "test-key".to_string(),
        correlation_id: "test-456".to_string(),
    };

    // Test development environment (detailed)
    let dev_message = error.user_friendly_message("development");
    assert!(dev_message.contains("Troubleshooting:"));
    assert!(dev_message.contains("For more help:"));
    assert!(dev_message.contains("canonical message"));

    // Test production environment (concise)
    let prod_message = error.user_friendly_message("production");
    assert!(prod_message.contains("reference error ID"));
    assert!(prod_message.contains("test-456"));
    assert!(!prod_message.contains("Troubleshooting:"));
}

#[tokio::test]
async fn test_error_response_format() {
    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config).expect("Valid config");

    let error = AuthenticationError::TimestampValidationFailed {
        timestamp: 1234567890,
        current_time: 1234567900,
        reason: "Too old".to_string(),
        correlation_id: "test-789".to_string(),
    };

    // Test error response creation
    let response = state.create_error_response(&error);

    assert!(response.error);
    assert_eq!(response.error_code, "TIMESTAMP_VALIDATION_FAILED");
    assert!(response.message.contains("timestamp"));
    assert_eq!(response.correlation_id, Some("test-789".to_string()));
    assert!(response.timestamp > 0);

    // In production mode (detailed_error_messages = false), details should be None
    assert!(response.details.is_none());
}

#[tokio::test]
async fn test_error_response_with_details() {
    let mut config = SignatureAuthConfig::default();
    config.response_security.detailed_error_messages = true; // Development mode
    let state = SignatureVerificationState::new(config).expect("Valid config");

    let error = AuthenticationError::InvalidSignatureFormat {
        reason: "Invalid hex encoding".to_string(),
        correlation_id: "test-details".to_string(),
    };

    let response = state.create_error_response(&error);

    assert!(response.error);
    assert_eq!(response.error_code, "INVALID_SIGNATURE_FORMAT");

    // In development mode, details should be present
    assert!(response.details.is_some());
    let details = response.details.unwrap();
    assert!(details.error_type.contains("InvalidSignatureFormat"));
    assert!(details.troubleshooting.contains("hex-encoded"));
    assert!(!details.suggested_actions.is_empty());
    assert!(details.documentation_link.contains("signature-auth"));
}

#[tokio::test]
async fn test_all_error_types_have_guidance() {
    let correlation_id = "test-all".to_string();

    let errors = vec![
        AuthenticationError::MissingHeaders {
            missing: vec!["test".to_string()],
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::InvalidSignatureFormat {
            reason: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::SignatureVerificationFailed {
            key_id: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::TimestampValidationFailed {
            timestamp: 123,
            current_time: 456,
            reason: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::NonceValidationFailed {
            nonce: "test".to_string(),
            reason: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::PublicKeyLookupFailed {
            key_id: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::ConfigurationError {
            reason: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::UnsupportedAlgorithm {
            algorithm: "rsa".to_string(),
            correlation_id: correlation_id.clone(),
        },
        AuthenticationError::RateLimitExceeded {
            client_id: "test".to_string(),
            correlation_id: correlation_id.clone(),
        },
    ];

    for error in errors {
        // Every error should have guidance
        let guidance = error.get_troubleshooting_guidance();
        assert!(
            !guidance.is_empty(),
            "Error {:?} should have troubleshooting guidance",
            error
        );

        // Every error should have suggested actions
        let actions = error.get_suggested_actions();
        assert!(
            !actions.is_empty(),
            "Error {:?} should have suggested actions",
            error
        );

        // Every error should have a documentation link
        let doc_link = error.get_documentation_link();
        assert!(
            doc_link.contains("docs.datafold.dev"),
            "Error {:?} should have valid doc link",
            error
        );

        // Every error should have an error code
        let error_code = error.error_code();
        assert!(
            !error_code.is_empty(),
            "Error {:?} should have error code",
            error
        );

        // Every error should have proper HTTP status codes
        let status = error.http_status_code();
        assert!(
            status.is_client_error() || status.is_server_error(),
            "Error {:?} should have appropriate HTTP status",
            error
        );
    }
}

#[tokio::test]
async fn test_error_severity_mapping() {
    let correlation_id = "test-severity".to_string();

    // Test critical severity errors
    let critical_error = AuthenticationError::NonceValidationFailed {
        nonce: "replay-nonce".to_string(),
        reason: "Nonce replay detected".to_string(),
        correlation_id: correlation_id.clone(),
    };
    assert_eq!(critical_error.severity(), SecurityEventSeverity::Critical);

    // Test warning severity errors
    let warn_error = AuthenticationError::SignatureVerificationFailed {
        key_id: "test-key".to_string(),
        correlation_id: correlation_id.clone(),
    };
    assert_eq!(warn_error.severity(), SecurityEventSeverity::Warn);

    // Test info severity errors
    let info_error = AuthenticationError::MissingHeaders {
        missing: vec!["Signature".to_string()],
        correlation_id: correlation_id.clone(),
    };
    assert_eq!(info_error.severity(), SecurityEventSeverity::Info);
}

#[tokio::test]
async fn test_correlation_id_consistency() {
    let correlation_id = "test-correlation-123".to_string();

    let error = AuthenticationError::ConfigurationError {
        reason: "Database connection failed".to_string(),
        correlation_id: correlation_id.clone(),
    };

    // Correlation ID should be consistent across different methods
    assert_eq!(error.correlation_id(), &correlation_id);

    let config = SignatureAuthConfig::default();
    let state = SignatureVerificationState::new(config).expect("Valid config");
    let response = state.create_error_response(&error);

    assert_eq!(response.correlation_id, Some(correlation_id));
}

#[tokio::test]
async fn test_error_message_security() {
    let config = SignatureAuthConfig::default(); // Production mode by default
    let state = SignatureVerificationState::new(config).expect("Valid config");

    let sensitive_error = AuthenticationError::ConfigurationError {
        reason: "Database password incorrect: secret123".to_string(),
        correlation_id: "test-security".to_string(),
    };

    // In production mode, sensitive details should not be exposed
    let error_message = state.get_error_message(&sensitive_error);
    assert!(!error_message.contains("secret123"));
    assert!(!error_message.contains("password"));
    assert!(error_message.contains("system administrator"));
}

#[tokio::test]
async fn test_timestamp_error_details() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let old_timestamp = now - 1000; // 1000 seconds ago

    let error = AuthenticationError::TimestampValidationFailed {
        timestamp: old_timestamp,
        current_time: now,
        reason: format!(
            "Timestamp outside allowed window: {} seconds (max: 300)",
            1000
        ),
        correlation_id: "test-timestamp".to_string(),
    };

    let guidance = error.get_troubleshooting_guidance();
    assert!(guidance.contains("clock synchronization"));
    assert!(guidance.contains("NTP"));
    assert!(guidance.contains(&old_timestamp.to_string()));
    assert!(guidance.contains(&now.to_string()));

    let actions = error.get_suggested_actions();
    assert!(actions
        .iter()
        .any(|action| action.contains("Synchronize system clock")));
    assert!(actions
        .iter()
        .any(|action| action.contains("current Unix timestamp")));
}

#[tokio::test]
async fn test_nonce_error_details() {
    let invalid_nonce = "invalid-nonce-with-special-chars!@#";

    let error = AuthenticationError::NonceValidationFailed {
        nonce: invalid_nonce.to_string(),
        reason: "Nonce contains invalid characters".to_string(),
        correlation_id: "test-nonce".to_string(),
    };

    let guidance = error.get_troubleshooting_guidance();
    assert!(guidance.contains("unique nonce"));
    assert!(guidance.contains("UUID4"));
    assert!(guidance.contains("length restrictions"));
    assert!(guidance.contains(&invalid_nonce));

    let actions = error.get_suggested_actions();
    assert!(actions
        .iter()
        .any(|action| action.contains("Generate a new unique nonce")));
    assert!(actions
        .iter()
        .any(|action| action.contains("format requirements")));
}

#[cfg(test)]
mod integration_tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};
    use datafold::datafold_node::signature_auth::SignatureVerificationMiddleware;

    async fn test_handler() -> HttpResponse {
        HttpResponse::Ok().json("success")
    }

    #[actix_web::test]
    async fn test_error_response_in_middleware() {
        let mut config = SignatureAuthConfig::default();
        config.response_security.detailed_error_messages = true; // Development mode
        config.response_security.include_correlation_id = true;

        let state = SignatureVerificationState::new(config).expect("Valid config");

        let app = test::init_service(
            App::new()
                .wrap(SignatureVerificationMiddleware::new(state))
                .route("/test", web::get().to(test_handler)),
        )
        .await;

        // Request without signature headers should return enhanced error response
        let req = test::TestRequest::get().uri("/test").to_request();

        // The middleware returns an actix-web error for authentication failures
        // We need to handle this properly in the test
        let result = test::try_call_service(&app, req).await;

        // Should be an error due to missing signature
        assert!(result.is_err());

        // Verify the error contains helpful information
        let error = result.unwrap_err();
        let error_message = format!("{}", error);
        assert!(error_message.contains("Missing required authentication headers"));
        assert!(error_message.contains("Troubleshooting"));
        assert!(error_message.contains("https://docs.datafold.dev/signature-auth/setup"));
    }
}
