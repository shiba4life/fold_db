//! Test suite for verification logic

#[cfg(test)]
mod tests {
    use crate::cli::verification::signature_data::*;
    use crate::cli::verification::verification_config::*;
    use crate::cli::verification::verification_engine::*;
    use crate::cli::verification::verification_inspector::*;
    use crate::cli::verification::verification_types::*;
    use crate::crypto::ed25519::PUBLIC_KEY_LENGTH;
    use std::collections::HashMap;

    #[test]
    fn test_verification_config_default() {
        let config = CliVerificationConfig::default();
        assert!(config.policies.contains_key("default"));
        assert!(config.policies.contains_key("strict"));
        assert!(config.policies.contains_key("permissive"));
        assert_eq!(config.default_policy, "default");
    }

    #[test]
    fn test_verifier_creation() {
        let verifier = CliSignatureVerifier::with_default_config();
        assert!(verifier.config().policies.len() >= 3);
    }

    #[test]
    fn test_add_public_key() {
        let mut verifier = CliSignatureVerifier::with_default_config();
        let key_bytes = vec![0u8; PUBLIC_KEY_LENGTH];

        let result = verifier.add_public_key("test-key".to_string(), key_bytes);
        assert!(result.is_ok());
        assert!(verifier.config().public_keys.contains_key("test-key"));
    }

    #[test]
    fn test_add_invalid_public_key() {
        let mut verifier = CliSignatureVerifier::with_default_config();
        let key_bytes = vec![0u8; 16]; // Invalid length

        let result = verifier.add_public_key("test-key".to_string(), key_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_signature_inspector() {
        let inspector = SignatureInspector::new(true);
        let mut headers = HashMap::new();
        headers.insert(
            "signature-input".to_string(),
            "sig1=(\"@method\" \"@target-uri\");alg=\"ed25519\"".to_string(),
        );
        headers.insert("signature".to_string(), "base64signature".to_string());

        let analysis = inspector.inspect_signature_format(&headers);
        assert!(analysis.is_valid_rfc9421);
        assert!(analysis.issues.is_empty());
        assert_eq!(analysis.signature_headers.len(), 2);
    }

    #[test]
    fn test_verification_status_display() {
        assert_eq!(VerificationStatus::Valid.to_string(), "VALID");
        assert_eq!(VerificationStatus::Invalid.to_string(), "INVALID");
        assert_eq!(VerificationStatus::Unknown.to_string(), "UNKNOWN");
        assert_eq!(VerificationStatus::Error.to_string(), "ERROR");
    }

    #[test]
    fn test_verification_policy_default() {
        let policy = VerificationPolicy::default();
        assert_eq!(policy.name, "default");
        assert!(policy.verify_timestamp);
        assert!(policy.verify_nonce);
        assert!(policy.verify_content_digest);
        assert_eq!(policy.max_timestamp_age, Some(300));
        assert!(policy.allowed_algorithms.contains(&"ed25519".to_string()));
    }

    #[test]
    fn test_verification_policy_strict() {
        let config = CliVerificationConfig::default();
        let strict_policy = config.policies.get("strict").unwrap();
        assert_eq!(strict_policy.name, "strict");
        assert_eq!(strict_policy.max_timestamp_age, Some(60));
        assert!(strict_policy.required_components.len() >= 3);
    }

    #[test]
    fn test_verification_policy_permissive() {
        let config = CliVerificationConfig::default();
        let permissive_policy = config.policies.get("permissive").unwrap();
        assert_eq!(permissive_policy.name, "permissive");
        assert!(!permissive_policy.verify_timestamp);
        assert!(!permissive_policy.verify_nonce);
        assert!(!permissive_policy.require_all_headers);
    }

    #[test]
    fn test_signature_inspector_missing_headers() {
        let inspector = SignatureInspector::new(true);
        let headers = HashMap::new(); // Empty headers

        let analysis = inspector.inspect_signature_format(&headers);
        assert!(!analysis.is_valid_rfc9421);
        assert_eq!(analysis.issues.len(), 2); // Missing both headers
        assert!(analysis.signature_headers.is_empty());
    }

    #[test]
    fn test_content_digest_creation() {
        let digest = ContentDigest {
            algorithm: "sha-256".to_string(),
            value: "test-hash".to_string(),
        };
        assert_eq!(digest.algorithm, "sha-256");
        assert_eq!(digest.value, "test-hash");
    }

    #[test]
    fn test_verification_result_data_creation() {
        let checks = VerificationChecks {
            format_valid: true,
            cryptographic_valid: true,
            timestamp_valid: true,
            nonce_valid: true,
            content_digest_valid: true,
            component_coverage_valid: true,
            policy_compliance_valid: true,
        };

        let diagnostics = VerificationDiagnostics {
            signature_analysis: SignatureAnalysis {
                algorithm: "ed25519".to_string(),
                key_id: "test-key".to_string(),
                created: None,
                age_seconds: None,
                nonce: None,
                covered_components: vec!["@method".to_string()],
            },
            content_analysis: ContentAnalysis {
                has_content_digest: false,
                digest_algorithm: None,
                content_size: 0,
                content_type: None,
            },
            policy_compliance: PolicyCompliance {
                policy_name: "default".to_string(),
                missing_required_components: vec![],
                extra_components: vec![],
                all_rules_passed: true,
            },
            security_analysis: SecurityAnalysis {
                security_level: crate::security_types::SecurityLevel::Standard,
                concerns: vec![],
                recommendations: vec![],
            },
        };

        let performance = PerformanceMetrics {
            total_time_ms: 100,
            step_timings: HashMap::new(),
        };

        let result = VerificationResultData {
            status: VerificationStatus::Valid,
            signature_valid: true,
            checks,
            diagnostics,
            performance,
            error: None,
        };

        assert_eq!(result.status, VerificationStatus::Valid);
        assert!(result.signature_valid);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_format_issue_severity() {
        let issue = FormatIssue {
            severity: FormatIssueSeverity::Error,
            code: "TEST_ERROR".to_string(),
            message: "Test error message".to_string(),
            component: Some("test-component".to_string()),
        };

        assert!(matches!(issue.severity, FormatIssueSeverity::Error));
        assert_eq!(issue.code, "TEST_ERROR");
        assert_eq!(issue.component, Some("test-component".to_string()));
    }
}