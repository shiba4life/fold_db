//! RFC 9421 HTTP Message Signatures compliance validation
//!
//! This module provides comprehensive validation for RFC 9421 compliance,
//! including header format validation, canonical message construction,
//! signature component validation, and test vector verification.

pub mod header_validator;
pub mod canonical_message;
pub mod signature_components;
pub mod test_vectors;

use crate::{CategoryResult, TestFailure, TestWarning, ValidationCategory, ValidationStatus};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Configuration for RFC 9421 compliance validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RFC9421Config {
    /// Enable strict header format validation
    pub strict_header_validation: bool,
    /// Validate signature component ordering
    pub validate_component_ordering: bool,
    /// Check for required signature components
    pub required_components: Vec<String>,
    /// Optional components to validate if present
    pub optional_components: Vec<String>,
    /// Supported signature algorithms
    pub supported_algorithms: Vec<String>,
    /// Enable test vector validation
    pub enable_test_vectors: bool,
    /// Path to custom test vectors
    pub custom_test_vectors_path: Option<String>,
    /// Enable canonical message validation
    pub validate_canonical_message: bool,
    /// Enable signature format validation
    pub validate_signature_format: bool,
}

impl Default for RFC9421Config {
    fn default() -> Self {
        Self {
            strict_header_validation: true,
            validate_component_ordering: true,
            required_components: vec![
                "@method".to_string(),
                "@target-uri".to_string(),
                "content-type".to_string(),
                "content-digest".to_string(),
            ],
            optional_components: vec![
                "authorization".to_string(),
                "@authority".to_string(),
                "@path".to_string(),
                "@query".to_string(),
                "date".to_string(),
                "x-datafold-client-id".to_string(),
                "x-datafold-user-id".to_string(),
                "x-datafold-metadata".to_string(),
            ],
            supported_algorithms: vec!["ed25519".to_string()],
            enable_test_vectors: true,
            custom_test_vectors_path: None,
            validate_canonical_message: true,
            validate_signature_format: true,
        }
    }
}

/// RFC 9421 compliance validator
pub struct RFC9421Validator {
    config: RFC9421Config,
    header_validator: header_validator::HeaderValidator,
    canonical_validator: canonical_message::CanonicalMessageValidator,
    component_validator: signature_components::SignatureComponentValidator,
    test_vector_validator: test_vectors::TestVectorValidator,
}

impl RFC9421Validator {
    /// Create a new RFC 9421 validator
    pub fn new(config: RFC9421Config) -> Result<Self> {
        let header_validator = header_validator::HeaderValidator::new(config.clone())?;
        let canonical_validator = canonical_message::CanonicalMessageValidator::new(config.clone())?;
        let component_validator = signature_components::SignatureComponentValidator::new(config.clone())?;
        let test_vector_validator = test_vectors::TestVectorValidator::new(config.clone())?;

        Ok(Self {
            config,
            header_validator,
            canonical_validator,
            component_validator,
            test_vector_validator,
        })
    }

    /// Run complete RFC 9421 compliance validation
    pub async fn run_validation(&self) -> Result<CategoryResult> {
        let start_time = Instant::now();
        info!("Starting RFC 9421 compliance validation");

        let mut tests_run = 0;
        let mut tests_passed = 0;
        let mut tests_failed = 0;
        let mut failures = Vec::new();
        let mut warnings = Vec::new();

        // Run header format validation
        if self.config.validate_signature_format {
            let result = self.run_header_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run canonical message validation
        if self.config.validate_canonical_message {
            let result = self.run_canonical_message_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        // Run signature component validation
        let result = self.run_signature_component_validation().await?;
        tests_run += result.tests_run;
        tests_passed += result.tests_passed;
        tests_failed += result.tests_failed;
        failures.extend(result.failures);
        warnings.extend(result.warnings);

        // Run test vector validation
        if self.config.enable_test_vectors {
            let result = self.run_test_vector_validation().await?;
            tests_run += result.tests_run;
            tests_passed += result.tests_passed;
            tests_failed += result.tests_failed;
            failures.extend(result.failures);
            warnings.extend(result.warnings);
        }

        let duration = start_time.elapsed();
        let status = if tests_failed > 0 {
            ValidationStatus::Failed
        } else if !warnings.is_empty() {
            ValidationStatus::Warning
        } else {
            ValidationStatus::Passed
        };

        info!("RFC 9421 compliance validation completed: {} passed, {} failed, {} warnings",
              tests_passed, tests_failed, warnings.len());

        Ok(CategoryResult {
            category: ValidationCategory::RFC9421Compliance,
            status,
            tests_run,
            tests_passed,
            tests_failed,
            tests_skipped: 0,
            duration_ms: duration.as_millis() as u64,
            failures,
            warnings,
        })
    }

    /// Run header format validation tests
    async fn run_header_validation(&self) -> Result<ValidationTestResult> {
        debug!("Running header format validation");
        self.header_validator.validate().await
    }

    /// Run canonical message validation tests
    async fn run_canonical_message_validation(&self) -> Result<ValidationTestResult> {
        debug!("Running canonical message validation");
        self.canonical_validator.validate().await
    }

    /// Run signature component validation tests
    async fn run_signature_component_validation(&self) -> Result<ValidationTestResult> {
        debug!("Running signature component validation");
        self.component_validator.validate().await
    }

    /// Run test vector validation tests
    async fn run_test_vector_validation(&self) -> Result<ValidationTestResult> {
        debug!("Running test vector validation");
        self.test_vector_validator.validate().await
    }
}

/// Internal result structure for individual validation tests
#[derive(Debug)]
struct ValidationTestResult {
    tests_run: usize,
    tests_passed: usize,
    tests_failed: usize,
    failures: Vec<TestFailure>,
    warnings: Vec<TestWarning>,
}

/// Test case definition for RFC 9421 validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RFC9421TestCase {
    pub name: String,
    pub description: String,
    pub request: TestHttpRequest,
    pub expected_signature_input: String,
    pub expected_signature: String,
    pub expected_canonical_message: String,
    pub private_key: String,
    pub public_key: String,
    pub key_id: String,
    pub algorithm: String,
    pub created: u64,
    pub nonce: Option<String>,
    pub expires: Option<u64>,
    pub should_pass: bool,
    pub failure_reason: Option<String>,
}

/// HTTP request structure for testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHttpRequest {
    pub method: String,
    pub uri: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

/// Signature validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureValidationResult {
    pub valid: bool,
    pub error_message: Option<String>,
    pub canonical_message: String,
    pub signature_input: String,
    pub signature: String,
    pub verification_details: HashMap<String, String>,
}

/// RFC 9421 specification constants
pub mod constants {
    /// Required signature components per DataFold protocol
    pub const REQUIRED_COMPONENTS: &[&str] = &[
        "@method",
        "@target-uri", 
        "content-type",
        "content-digest",
    ];

    /// Supported algorithms
    pub const SUPPORTED_ALGORITHMS: &[&str] = &["ed25519"];

    /// Reserved parameter names from RFC 9421
    pub const RESERVED_PARAMETERS: &[&str] = &[
        "alg",
        "created", 
        "expires",
        "keyid",
        "nonce",
        "tag",
    ];

    /// Maximum signature age in seconds (5 minutes default)
    pub const MAX_SIGNATURE_AGE: u64 = 300;

    /// Maximum future timestamp tolerance in seconds (1 minute default)
    pub const MAX_FUTURE_TOLERANCE: u64 = 60;
}

/// Utility functions for RFC 9421 validation
pub mod utils {
    use super::*;
    use base64::{Engine as _, engine::general_purpose};
    use std::collections::BTreeMap;

    /// Parse signature input header according to RFC 9421
    pub fn parse_signature_input(header_value: &str) -> Result<(Vec<String>, BTreeMap<String, String>)> {
        // Implementation follows RFC 9421 section 4.1
        // Format: sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key-ed25519"
        
        let parts: Vec<&str> = header_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(anyhow::anyhow!("Invalid signature input format"));
        }

        let signature_name = parts[0];
        let signature_params = parts[1];

        // Parse component list
        let component_start = signature_params.find('(').ok_or_else(|| {
            anyhow::anyhow!("Missing component list start")
        })?;
        let component_end = signature_params.find(')').ok_or_else(|| {
            anyhow::anyhow!("Missing component list end")
        })?;

        let component_list = &signature_params[component_start + 1..component_end];
        let components: Vec<String> = component_list
            .split_whitespace()
            .map(|s| s.trim_matches('"').to_string())
            .collect();

        // Parse parameters
        let params_str = &signature_params[component_end + 1..];
        let mut parameters = BTreeMap::new();

        for param in params_str.split(';') {
            let param = param.trim();
            if param.is_empty() {
                continue;
            }

            let param_parts: Vec<&str> = param.splitn(2, '=').collect();
            if param_parts.len() == 2 {
                let key = param_parts[0].trim();
                let value = param_parts[1].trim().trim_matches('"');
                parameters.insert(key.to_string(), value.to_string());
            }
        }

        Ok((components, parameters))
    }

    /// Parse signature header according to RFC 9421
    pub fn parse_signature_header(header_value: &str) -> Result<HashMap<String, Vec<u8>>> {
        // Format: sig1=:base64-signature:
        let mut signatures = HashMap::new();

        for signature_item in header_value.split(',') {
            let parts: Vec<&str> = signature_item.trim().splitn(2, '=').collect();
            if parts.len() != 2 {
                continue;
            }

            let signature_name = parts[0].trim();
            let signature_value = parts[1].trim();

            // Remove wrapping colons
            if signature_value.starts_with(':') && signature_value.ends_with(':') {
                let base64_sig = &signature_value[1..signature_value.len() - 1];
                let signature_bytes = general_purpose::STANDARD
                    .decode(base64_sig)
                    .context("Failed to decode base64 signature")?;
                signatures.insert(signature_name.to_string(), signature_bytes);
            }
        }

        Ok(signatures)
    }

    /// Validate algorithm identifier
    pub fn validate_algorithm(algorithm: &str) -> bool {
        constants::SUPPORTED_ALGORITHMS.contains(&algorithm)
    }

    /// Validate component name format
    pub fn validate_component_name(component: &str) -> bool {
        // RFC 9421 component name validation
        if component.starts_with('@') {
            // Derived component
            matches!(component, "@method" | "@target-uri" | "@authority" | "@scheme" | 
                     "@request-target" | "@path" | "@query" | "@status")
        } else {
            // HTTP header component - must be valid header name
            component.chars().all(|c| c.is_ascii() && !c.is_control() && c != ':')
        }
    }

    /// Generate test vector identifier
    pub fn generate_test_vector_id(test_case: &RFC9421TestCase) -> String {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        hasher.update(test_case.name.as_bytes());
        hasher.update(test_case.request.method.as_bytes());
        hasher.update(test_case.request.uri.as_bytes());
        
        let hash = hasher.finalize();
        hex::encode(&hash[..8]) // First 8 bytes as hex
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rfc9421_validator_creation() {
        let config = RFC9421Config::default();
        let validator = RFC9421Validator::new(config).unwrap();
        assert!(validator.config.strict_header_validation);
    }

    #[test]
    fn test_parse_signature_input() {
        let input = r#"sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key""#;
        let result = utils::parse_signature_input(input).unwrap();
        
        assert_eq!(result.0, vec!["@method", "@target-uri", "content-type"]);
        assert_eq!(result.1.get("created"), Some(&"1618884473".to_string()));
        assert_eq!(result.1.get("keyid"), Some(&"test-key".to_string()));
    }

    #[test]
    fn test_validate_algorithm() {
        assert!(utils::validate_algorithm("ed25519"));
        assert!(!utils::validate_algorithm("rsa-sha256"));
        assert!(!utils::validate_algorithm("invalid"));
    }

    #[test]
    fn test_validate_component_name() {
        assert!(utils::validate_component_name("@method"));
        assert!(utils::validate_component_name("content-type"));
        assert!(utils::validate_component_name("x-custom-header"));
        assert!(!utils::validate_component_name("@invalid-derived"));
        assert!(!utils::validate_component_name("invalid:header"));
    }
}