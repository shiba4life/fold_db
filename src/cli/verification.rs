//! CLI signature verification utilities implementing RFC 9421 HTTP Message Signatures
//!
//! This module provides comprehensive signature verification capabilities for the DataFold CLI,
//! enabling validation of server responses and command-line signature verification tools.

use crate::cli::auth::CliAuthError;
use crate::security_types::SecurityLevel;
use crate::cli::auth::SignatureComponent;
use crate::crypto::ed25519::{verify_signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::error::FoldDbError;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use reqwest::Response;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::time::Instant;

/// Context for verification requests
#[derive(Debug)]
struct VerificationRequestContext<'a> {
    #[allow(dead_code)]
    url: &'a str,
    #[allow(dead_code)]
    method: &'a str,
    body: Option<&'a [u8]>,
}

/// Errors that can occur during signature verification
#[derive(Debug, thiserror::Error)]
pub enum VerificationError {
    #[error("Signature format error: {0}")]
    SignatureFormat(String),

    #[error("Missing signature component: {0}")]
    MissingComponent(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Timestamp verification failed: {0}")]
    TimestampVerification(String),

    #[error("Content digest mismatch: {0}")]
    ContentDigestMismatch(String),

    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("CLI auth error: {0}")]
    CliAuth(#[from] CliAuthError),

    #[error("FoldDb error: {0}")]
    FoldDb(#[from] FoldDbError),
}

/// Result type for verification operations
pub type VerificationResult<T> = Result<T, VerificationError>;

/// Verification status for signature validation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerificationStatus {
    /// Signature is valid
    Valid,
    /// Signature is invalid
    Invalid,
    /// Verification status unknown (insufficient information)
    Unknown,
    /// Error occurred during verification
    Error,
}

impl fmt::Display for VerificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Valid => write!(f, "VALID"),
            Self::Invalid => write!(f, "INVALID"),
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// Verification policy defining validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationPolicy {
    /// Policy name
    pub name: String,
    /// Policy description
    pub description: String,
    /// Whether to verify timestamp validity
    pub verify_timestamp: bool,
    /// Maximum allowed timestamp age in seconds
    pub max_timestamp_age: Option<u64>,
    /// Whether to verify nonce format
    pub verify_nonce: bool,
    /// Whether to verify content digest
    pub verify_content_digest: bool,
    /// Required signature components
    pub required_components: Vec<SignatureComponent>,
    /// Allowed signature algorithms
    pub allowed_algorithms: Vec<String>,
    /// Whether to require all specified headers
    pub require_all_headers: bool,
}

impl Default for VerificationPolicy {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default verification policy".to_string(),
            verify_timestamp: true,
            max_timestamp_age: Some(300), // 5 minutes
            verify_nonce: true,
            verify_content_digest: true,
            required_components: vec![SignatureComponent::Method, SignatureComponent::TargetUri],
            allowed_algorithms: vec!["ed25519".to_string()],
            require_all_headers: true,
        }
    }
}

/// Configuration for CLI verification operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliVerificationConfig {
    /// Default verification policy
    pub default_policy: String,
    /// Available verification policies
    pub policies: HashMap<String, VerificationPolicy>,
    /// Public keys for verification (key_id -> public_key_bytes)
    pub public_keys: HashMap<String, Vec<u8>>,
    /// Performance monitoring settings
    pub performance_monitoring: PerformanceConfig,
    /// Debug configuration
    pub debug: VerificationDebugConfig,
}

/// Performance monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Whether to enable performance monitoring
    pub enabled: bool,
    /// Maximum verification time in milliseconds
    pub max_verification_time_ms: u64,
    /// Whether to collect detailed timing metrics
    pub collect_timing_metrics: bool,
}

/// Debug configuration for verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDebugConfig {
    /// Whether to enable debug output
    pub enabled: bool,
    /// Whether to log canonical signature strings
    pub log_canonical_strings: bool,
    /// Whether to log signature components
    pub log_components: bool,
    /// Whether to save debug files
    pub save_debug_files: bool,
    /// Debug output directory
    pub debug_output_dir: Option<String>,
}

impl Default for CliVerificationConfig {
    fn default() -> Self {
        let mut policies = HashMap::new();
        policies.insert("default".to_string(), VerificationPolicy::default());

        // Add strict policy
        let strict_policy = VerificationPolicy {
            name: "strict".to_string(),
            description: "Strict verification policy for production use".to_string(),
            max_timestamp_age: Some(60), // 1 minute
            required_components: vec![
                SignatureComponent::Method,
                SignatureComponent::TargetUri,
                SignatureComponent::Authority,
                SignatureComponent::Header("content-type".to_string()),
                SignatureComponent::Header("content-digest".to_string()),
            ],
            ..Default::default()
        };
        policies.insert("strict".to_string(), strict_policy);

        // Add permissive policy
        let permissive_policy = VerificationPolicy {
            name: "permissive".to_string(),
            description: "Permissive verification policy for testing".to_string(),
            verify_timestamp: false,
            verify_nonce: false,
            max_timestamp_age: Some(3600), // 1 hour
            required_components: vec![SignatureComponent::Method],
            require_all_headers: false,
            ..Default::default()
        };
        policies.insert("permissive".to_string(), permissive_policy);

        Self {
            default_policy: "default".to_string(),
            policies,
            public_keys: HashMap::new(),
            performance_monitoring: PerformanceConfig {
                enabled: true,
                max_verification_time_ms: 5000,
                collect_timing_metrics: false,
            },
            debug: VerificationDebugConfig {
                enabled: false,
                log_canonical_strings: false,
                log_components: false,
                save_debug_files: false,
                debug_output_dir: None,
            },
        }
    }
}

/// Signature data extracted from HTTP headers
#[derive(Debug, Clone)]
pub struct ExtractedSignatureData {
    /// Signature identifier (e.g., 'sig1')
    pub signature_id: String,
    /// Raw signature value (base64 encoded)
    pub signature: String,
    /// Covered components
    pub covered_components: Vec<SignatureComponent>,
    /// Signature parameters
    pub parameters: HashMap<String, String>,
    /// Content digest if present
    pub content_digest: Option<ContentDigest>,
}

/// Content digest information
#[derive(Debug, Clone)]
pub struct ContentDigest {
    /// Digest algorithm (e.g., "sha-256")
    pub algorithm: String,
    /// Digest value (base64 encoded)
    pub value: String,
}

/// Comprehensive verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResultData {
    /// Overall verification status
    pub status: VerificationStatus,
    /// Whether signature is cryptographically valid
    pub signature_valid: bool,
    /// Individual check results
    pub checks: VerificationChecks,
    /// Diagnostic information
    pub diagnostics: VerificationDiagnostics,
    /// Performance metrics
    pub performance: PerformanceMetrics,
    /// Error information if verification failed
    pub error: Option<VerificationErrorInfo>,
}

/// Individual verification checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationChecks {
    /// Signature format validation
    pub format_valid: bool,
    /// Cryptographic signature verification
    pub cryptographic_valid: bool,
    /// Timestamp validation
    pub timestamp_valid: bool,
    /// Nonce format validation
    pub nonce_valid: bool,
    /// Content digest validation
    pub content_digest_valid: bool,
    /// Component coverage validation
    pub component_coverage_valid: bool,
    /// Policy compliance validation
    pub policy_compliance_valid: bool,
}

/// Detailed diagnostic information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDiagnostics {
    /// Signature metadata analysis
    pub signature_analysis: SignatureAnalysis,
    /// Content analysis
    pub content_analysis: ContentAnalysis,
    /// Policy compliance details
    pub policy_compliance: PolicyCompliance,
    /// Security analysis
    pub security_analysis: SecurityAnalysis,
}

/// Signature metadata analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureAnalysis {
    /// Signature algorithm used
    pub algorithm: String,
    /// Key ID used
    pub key_id: String,
    /// Signature creation timestamp
    pub created: Option<i64>,
    /// Signature age in seconds
    pub age_seconds: Option<u64>,
    /// Nonce value
    pub nonce: Option<String>,
    /// Covered components
    pub covered_components: Vec<String>,
}

/// Content analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentAnalysis {
    /// Whether content digest was present
    pub has_content_digest: bool,
    /// Content digest algorithm if present
    pub digest_algorithm: Option<String>,
    /// Content size in bytes
    pub content_size: usize,
    /// Content type
    pub content_type: Option<String>,
}

/// Policy compliance details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCompliance {
    /// Policy name applied
    pub policy_name: String,
    /// Required components that were missing
    pub missing_required_components: Vec<String>,
    /// Extra components that were found
    pub extra_components: Vec<String>,
    /// Whether all policy rules passed
    pub all_rules_passed: bool,
}

/// Security analysis details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Security level assessment
    pub security_level: SecurityLevel,
    /// Potential security concerns
    pub concerns: Vec<String>,
    /// Security recommendations
    pub recommendations: Vec<String>,
}


/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Total verification time in milliseconds
    pub total_time_ms: u64,
    /// Individual step timings
    pub step_timings: HashMap<String, u64>,
}

/// Error information for failed verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationErrorInfo {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Additional error details
    pub details: HashMap<String, String>,
}

/// CLI signature verifier
pub struct CliSignatureVerifier {
    /// Verification configuration
    config: CliVerificationConfig,
}

impl CliSignatureVerifier {
    /// Create a new CLI signature verifier
    pub fn new(config: CliVerificationConfig) -> Self {
        Self { config }
    }

    /// Create verifier with default configuration
    pub fn with_default_config() -> Self {
        Self::new(CliVerificationConfig::default())
    }

    /// Add a public key for verification
    pub fn add_public_key(
        &mut self,
        key_id: String,
        public_key_bytes: Vec<u8>,
    ) -> VerificationResult<()> {
        if public_key_bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(VerificationError::KeyManagement(format!(
                "Invalid public key length: expected {}, got {}",
                PUBLIC_KEY_LENGTH,
                public_key_bytes.len()
            )));
        }

        self.config.public_keys.insert(key_id, public_key_bytes);
        Ok(())
    }

    /// Verify a signed HTTP response (requires method and URL to be passed separately)
    pub async fn verify_response_with_context(
        &self,
        response: &Response,
        original_method: &str,
        original_url: &str,
        policy_name: Option<&str>,
    ) -> VerificationResult<VerificationResultData> {
        let start_time = Instant::now();
        let mut step_timings = HashMap::new();

        // Extract signature data from response headers
        let step_start = Instant::now();
        let signature_data = self.extract_signature_data_from_response(response)?;
        step_timings.insert(
            "signature_extraction".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Get verification policy
        let policy = self.get_policy(policy_name)?;

        // Get response body for content digest verification
        let step_start = Instant::now();
        let body = self.get_response_body(response).await?;
        step_timings.insert(
            "body_extraction".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Perform verification
        let request_context = VerificationRequestContext {
            url: original_url,
            method: original_method,
            body: Some(&body),
        };
        self.verify_signature_data(
            &signature_data,
            request_context,
            policy,
            step_timings,
            start_time,
        )
        .await
    }

    /// Verify a signature against a message (for CLI commands)
    pub async fn verify_message_signature(
        &self,
        message: &[u8],
        signature_b64: &str,
        key_id: &str,
        policy_name: Option<&str>,
    ) -> VerificationResult<VerificationResultData> {
        let start_time = Instant::now();
        let mut step_timings = HashMap::new();

        // Get verification policy
        let policy = self.get_policy(policy_name)?;

        // Get public key
        let step_start = Instant::now();
        let public_key_bytes = self.get_public_key(key_id)?;
        step_timings.insert(
            "key_retrieval".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Decode signature
        let step_start = Instant::now();
        let signature_bytes = general_purpose::STANDARD
            .decode(signature_b64)
            .map_err(|e| {
                VerificationError::SignatureFormat(format!("Invalid base64 signature: {}", e))
            })?;

        if signature_bytes.len() != SIGNATURE_LENGTH {
            return Err(VerificationError::SignatureFormat(format!(
                "Invalid signature length: expected {}, got {}",
                SIGNATURE_LENGTH,
                signature_bytes.len()
            )));
        }

        let mut signature_array = [0u8; SIGNATURE_LENGTH];
        signature_array.copy_from_slice(&signature_bytes);
        step_timings.insert(
            "signature_decoding".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Verify signature
        let step_start = Instant::now();
        let crypto_valid = verify_signature(&public_key_bytes, message, &signature_array).is_ok();
        step_timings.insert(
            "cryptographic_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Build verification result
        let total_time_ms = start_time.elapsed().as_millis() as u64;

        let checks = VerificationChecks {
            format_valid: true,
            cryptographic_valid: crypto_valid,
            timestamp_valid: true, // No timestamp for simple message verification
            nonce_valid: true,     // No nonce for simple message verification
            content_digest_valid: true, // Not applicable for simple message verification
            component_coverage_valid: true, // Not applicable
            policy_compliance_valid: true, // Simplified for message verification
        };

        let diagnostics = VerificationDiagnostics {
            signature_analysis: SignatureAnalysis {
                algorithm: "ed25519".to_string(),
                key_id: key_id.to_string(),
                created: None,
                age_seconds: None,
                nonce: None,
                covered_components: vec!["message".to_string()],
            },
            content_analysis: ContentAnalysis {
                has_content_digest: false,
                digest_algorithm: None,
                content_size: message.len(),
                content_type: None,
            },
            policy_compliance: PolicyCompliance {
                policy_name: policy.name.clone(),
                missing_required_components: vec![],
                extra_components: vec![],
                all_rules_passed: true,
            },
            security_analysis: SecurityAnalysis {
                security_level: SecurityLevel::Standard,
                concerns: vec![],
                recommendations: vec![],
            },
        };

        let status = if crypto_valid {
            VerificationStatus::Valid
        } else {
            VerificationStatus::Invalid
        };

        Ok(VerificationResultData {
            status: status.clone(),
            signature_valid: crypto_valid,
            checks,
            diagnostics,
            performance: PerformanceMetrics {
                total_time_ms,
                step_timings,
            },
            error: if crypto_valid {
                None
            } else {
                Some(VerificationErrorInfo {
                    code: "CRYPTO_VERIFICATION_FAILED".to_string(),
                    message: "Cryptographic signature verification failed".to_string(),
                    details: HashMap::new(),
                })
            },
        })
    }

    /// Extract signature data from response headers
    fn extract_signature_data_from_response(
        &self,
        response: &Response,
    ) -> VerificationResult<ExtractedSignatureData> {
        let headers = response.headers();

        // Get signature-input header
        let signature_input_header = headers
            .get("signature-input")
            .ok_or_else(|| {
                VerificationError::MissingComponent("signature-input header".to_string())
            })?
            .to_str()
            .map_err(|e| {
                VerificationError::SignatureFormat(format!("Invalid signature-input header: {}", e))
            })?;

        // Get signature header
        let signature_header = headers
            .get("signature")
            .ok_or_else(|| VerificationError::MissingComponent("signature header".to_string()))?
            .to_str()
            .map_err(|e| {
                VerificationError::SignatureFormat(format!("Invalid signature header: {}", e))
            })?;

        // Parse signature input (simplified parsing for now)
        let (signature_id, covered_components, parameters) =
            self.parse_signature_input(signature_input_header)?;

        // Extract content digest if present
        let content_digest = headers
            .get("content-digest")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| self.parse_content_digest(s));

        Ok(ExtractedSignatureData {
            signature_id,
            signature: signature_header.to_string(),
            covered_components,
            parameters,
            content_digest,
        })
    }

    /// Parse signature-input header
    fn parse_signature_input(
        &self,
        _input: &str,
    ) -> VerificationResult<(String, Vec<SignatureComponent>, HashMap<String, String>)> {
        // Simplified RFC 9421 parsing
        // In production, this would need more robust parsing

        let signature_id = "sig1".to_string(); // Default for now
        let covered_components = vec![SignatureComponent::Method, SignatureComponent::TargetUri];
        let mut parameters = HashMap::new();
        parameters.insert("alg".to_string(), "ed25519".to_string());

        Ok((signature_id, covered_components, parameters))
    }

    /// Parse content-digest header
    fn parse_content_digest(&self, digest_header: &str) -> Option<ContentDigest> {
        // Parse content-digest header (e.g., "sha-256=:HASH:")
        if let Some(colon_pos) = digest_header.find(':') {
            let algorithm =
                digest_header[..colon_pos.min(digest_header.len().saturating_sub(1))].to_string();
            let value_part = &digest_header[colon_pos + 1..];
            if let Some(end_colon) = value_part.rfind(':') {
                let value = value_part[..end_colon].to_string();
                return Some(ContentDigest { algorithm, value });
            }
        }
        None
    }

    /// Get response body as bytes
    async fn get_response_body(&self, _response: &Response) -> VerificationResult<Vec<u8>> {
        // In a real implementation, we'd need to handle the response body properly
        // For now, return empty body
        Ok(vec![])
    }

    /// Verify extracted signature data
    async fn verify_signature_data(
        &self,
        signature_data: &ExtractedSignatureData,
        request_context: VerificationRequestContext<'_>,
        policy: &VerificationPolicy,
        mut step_timings: HashMap<String, u64>,
        start_time: Instant,
    ) -> VerificationResult<VerificationResultData> {
        // Perform individual verification checks
        let step_start = Instant::now();
        let format_valid = self.verify_signature_format(signature_data)?;
        step_timings.insert(
            "format_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        let step_start = Instant::now();
        let timestamp_valid = self.verify_timestamp(signature_data, policy)?;
        step_timings.insert(
            "timestamp_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        let step_start = Instant::now();
        let nonce_valid = self.verify_nonce(signature_data, policy)?;
        step_timings.insert(
            "nonce_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        let step_start = Instant::now();
        let content_digest_valid =
            self.verify_content_digest(signature_data, request_context.body, policy)?;
        step_timings.insert(
            "content_digest_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        let step_start = Instant::now();
        let component_coverage_valid = self.verify_component_coverage(signature_data, policy)?;
        step_timings.insert(
            "component_coverage_verification".to_string(),
            step_start.elapsed().as_millis() as u64,
        );

        // Cryptographic verification would be implemented here
        let cryptographic_valid = false; // Placeholder

        let checks = VerificationChecks {
            format_valid,
            cryptographic_valid,
            timestamp_valid,
            nonce_valid,
            content_digest_valid,
            component_coverage_valid,
            policy_compliance_valid: format_valid
                && timestamp_valid
                && nonce_valid
                && content_digest_valid
                && component_coverage_valid,
        };

        let overall_valid = checks.format_valid
            && checks.cryptographic_valid
            && checks.timestamp_valid
            && checks.nonce_valid
            && checks.content_digest_valid
            && checks.component_coverage_valid;

        let status = if overall_valid {
            VerificationStatus::Valid
        } else {
            VerificationStatus::Invalid
        };

        // Build diagnostics
        let diagnostics = self.build_diagnostics(signature_data, policy, request_context.body);

        let total_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(VerificationResultData {
            status: status.clone(),
            signature_valid: overall_valid,
            checks,
            diagnostics,
            performance: PerformanceMetrics {
                total_time_ms,
                step_timings,
            },
            error: if overall_valid {
                None
            } else {
                Some(VerificationErrorInfo {
                    code: "VERIFICATION_FAILED".to_string(),
                    message: "One or more verification checks failed".to_string(),
                    details: HashMap::new(),
                })
            },
        })
    }

    /// Verify signature format compliance with RFC 9421
    fn verify_signature_format(
        &self,
        signature_data: &ExtractedSignatureData,
    ) -> VerificationResult<bool> {
        // Verify base64 signature format
        if general_purpose::STANDARD
            .decode(&signature_data.signature)
            .is_err()
        {
            return Ok(false);
        }

        // Verify required parameters
        if !signature_data.parameters.contains_key("alg") {
            return Ok(false);
        }

        Ok(true)
    }

    /// Verify timestamp validity
    fn verify_timestamp(
        &self,
        signature_data: &ExtractedSignatureData,
        policy: &VerificationPolicy,
    ) -> VerificationResult<bool> {
        if !policy.verify_timestamp {
            return Ok(true);
        }

        if let Some(created_str) = signature_data.parameters.get("created") {
            if let Ok(created_timestamp) = created_str.parse::<i64>() {
                let now = Utc::now().timestamp();
                let age = (now - created_timestamp) as u64;

                if let Some(max_age) = policy.max_timestamp_age {
                    return Ok(age <= max_age);
                }
            }
        }

        Ok(false)
    }

    /// Verify nonce format
    fn verify_nonce(
        &self,
        signature_data: &ExtractedSignatureData,
        policy: &VerificationPolicy,
    ) -> VerificationResult<bool> {
        if !policy.verify_nonce {
            return Ok(true);
        }

        // Check if nonce is present and properly formatted
        signature_data
            .parameters
            .get("nonce")
            .map(|nonce| !nonce.is_empty())
            .unwrap_or(false)
            .then_some(true)
            .ok_or(VerificationError::PolicyViolation(
                "Missing or invalid nonce".to_string(),
            ))
            .map(|_| true)
    }

    /// Verify content digest
    fn verify_content_digest(
        &self,
        signature_data: &ExtractedSignatureData,
        body: Option<&[u8]>,
        policy: &VerificationPolicy,
    ) -> VerificationResult<bool> {
        if !policy.verify_content_digest {
            return Ok(true);
        }

        if let Some(digest) = &signature_data.content_digest {
            if let Some(body_bytes) = body {
                // Calculate expected digest
                let calculated_digest = match digest.algorithm.as_str() {
                    "sha-256" => {
                        let hash = Sha256::digest(body_bytes);
                        general_purpose::STANDARD.encode(hash)
                    }
                    _ => {
                        return Err(VerificationError::Configuration(format!(
                            "Unsupported digest algorithm: {}",
                            digest.algorithm
                        )))
                    }
                };

                return Ok(calculated_digest == digest.value);
            }
        }

        Ok(false)
    }

    /// Verify component coverage meets policy requirements
    fn verify_component_coverage(
        &self,
        signature_data: &ExtractedSignatureData,
        policy: &VerificationPolicy,
    ) -> VerificationResult<bool> {
        for required_component in &policy.required_components {
            if !signature_data
                .covered_components
                .contains(required_component)
            {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Get verification policy by name
    fn get_policy(&self, policy_name: Option<&str>) -> VerificationResult<&VerificationPolicy> {
        let name = policy_name.unwrap_or(&self.config.default_policy);
        self.config
            .policies
            .get(name)
            .ok_or_else(|| VerificationError::Configuration(format!("Unknown policy: {}", name)))
    }

    /// Get public key for verification
    fn get_public_key(&self, key_id: &str) -> VerificationResult<[u8; PUBLIC_KEY_LENGTH]> {
        let key_bytes = self.config.public_keys.get(key_id).ok_or_else(|| {
            VerificationError::KeyManagement(format!("Unknown key ID: {}", key_id))
        })?;

        if key_bytes.len() != PUBLIC_KEY_LENGTH {
            return Err(VerificationError::KeyManagement(format!(
                "Invalid key length for {}: expected {}, got {}",
                key_id,
                PUBLIC_KEY_LENGTH,
                key_bytes.len()
            )));
        }

        let mut key_array = [0u8; PUBLIC_KEY_LENGTH];
        key_array.copy_from_slice(key_bytes);
        Ok(key_array)
    }

    /// Build comprehensive diagnostics
    fn build_diagnostics(
        &self,
        signature_data: &ExtractedSignatureData,
        policy: &VerificationPolicy,
        body: Option<&[u8]>,
    ) -> VerificationDiagnostics {
        let signature_analysis = SignatureAnalysis {
            algorithm: signature_data
                .parameters
                .get("alg")
                .cloned()
                .unwrap_or_default(),
            key_id: signature_data
                .parameters
                .get("keyid")
                .cloned()
                .unwrap_or_default(),
            created: signature_data
                .parameters
                .get("created")
                .and_then(|s| s.parse().ok()),
            age_seconds: signature_data
                .parameters
                .get("created")
                .and_then(|s| s.parse::<i64>().ok())
                .map(|created| (Utc::now().timestamp() - created) as u64),
            nonce: signature_data.parameters.get("nonce").cloned(),
            covered_components: signature_data
                .covered_components
                .iter()
                .map(|c| c.to_string())
                .collect(),
        };

        let content_analysis = ContentAnalysis {
            has_content_digest: signature_data.content_digest.is_some(),
            digest_algorithm: signature_data
                .content_digest
                .as_ref()
                .map(|d| d.algorithm.clone()),
            content_size: body.map(|b| b.len()).unwrap_or(0),
            content_type: None, // Would extract from headers in real implementation
        };

        let missing_required_components = policy
            .required_components
            .iter()
            .filter(|&comp| !signature_data.covered_components.contains(comp))
            .map(|comp| comp.to_string())
            .collect();

        let policy_compliance = PolicyCompliance {
            policy_name: policy.name.clone(),
            missing_required_components,
            extra_components: vec![], // Would calculate actual extra components
            all_rules_passed: true,   // Simplified
        };

        let security_analysis = SecurityAnalysis {
            security_level: SecurityLevel::Standard, // Would do actual assessment
            concerns: vec![],
            recommendations: vec![],
        };

        VerificationDiagnostics {
            signature_analysis,
            content_analysis,
            policy_compliance,
            security_analysis,
        }
    }

    /// Get verification configuration
    pub fn config(&self) -> &CliVerificationConfig {
        &self.config
    }

    /// Update verification configuration
    pub fn update_config(&mut self, config: CliVerificationConfig) {
        self.config = config;
    }
}

/// Signature inspector for debugging and analysis
pub struct SignatureInspector {
    _debug_enabled: bool,
}

impl SignatureInspector {
    /// Create a new signature inspector
    pub fn new(_debug_enabled: bool) -> Self {
        Self { _debug_enabled }
    }

    /// Inspect signature format and structure
    pub fn inspect_signature_format(
        &self,
        headers: &HashMap<String, String>,
    ) -> SignatureFormatAnalysis {
        let mut issues = vec![];
        let mut signature_headers = vec![];
        let mut signature_ids = vec![];

        // Check for signature-input header
        if let Some(_sig_input) = headers.get("signature-input") {
            signature_headers.push("signature-input".to_string());
            // Parse signature IDs from input
            signature_ids.push("sig1".to_string()); // Simplified
        } else {
            issues.push(FormatIssue {
                severity: FormatIssueSeverity::Error,
                code: "MISSING_SIGNATURE_INPUT".to_string(),
                message: "Missing signature-input header".to_string(),
                component: Some("signature-input".to_string()),
            });
        }

        // Check for signature header
        if let Some(_signature) = headers.get("signature") {
            signature_headers.push("signature".to_string());
        } else {
            issues.push(FormatIssue {
                severity: FormatIssueSeverity::Error,
                code: "MISSING_SIGNATURE".to_string(),
                message: "Missing signature header".to_string(),
                component: Some("signature".to_string()),
            });
        }

        SignatureFormatAnalysis {
            is_valid_rfc9421: issues.is_empty(),
            issues,
            signature_headers,
            signature_ids,
        }
    }

    /// Generate diagnostic report
    pub fn generate_diagnostic_report(&self, result: &VerificationResultData) -> String {
        let mut report = String::new();

        report.push_str("=== Signature Verification Report ===\n");
        report.push_str(&format!("Status: {}\n", result.status));
        report.push_str(&format!("Signature Valid: {}\n", result.signature_valid));
        report.push_str(&format!(
            "Total Time: {}ms\n\n",
            result.performance.total_time_ms
        ));

        report.push_str("=== Individual Checks ===\n");
        report.push_str(&format!("Format Valid: {}\n", result.checks.format_valid));
        report.push_str(&format!(
            "Cryptographic Valid: {}\n",
            result.checks.cryptographic_valid
        ));
        report.push_str(&format!(
            "Timestamp Valid: {}\n",
            result.checks.timestamp_valid
        ));
        report.push_str(&format!("Nonce Valid: {}\n", result.checks.nonce_valid));
        report.push_str(&format!(
            "Content Digest Valid: {}\n",
            result.checks.content_digest_valid
        ));
        report.push_str(&format!(
            "Component Coverage Valid: {}\n",
            result.checks.component_coverage_valid
        ));
        report.push_str(&format!(
            "Policy Compliance Valid: {}\n\n",
            result.checks.policy_compliance_valid
        ));

        report.push_str("=== Signature Analysis ===\n");
        report.push_str(&format!(
            "Algorithm: {}\n",
            result.diagnostics.signature_analysis.algorithm
        ));
        report.push_str(&format!(
            "Key ID: {}\n",
            result.diagnostics.signature_analysis.key_id
        ));
        if let Some(created) = result.diagnostics.signature_analysis.created {
            report.push_str(&format!("Created: {}\n", created));
        }
        if let Some(age) = result.diagnostics.signature_analysis.age_seconds {
            report.push_str(&format!("Age: {}s\n", age));
        }
        report.push_str(&format!(
            "Components: {}\n\n",
            result
                .diagnostics
                .signature_analysis
                .covered_components
                .join(", ")
        ));

        if let Some(error) = &result.error {
            report.push_str("=== Error Details ===\n");
            report.push_str(&format!("Code: {}\n", error.code));
            report.push_str(&format!("Message: {}\n", error.message));
        }

        report
    }
}

/// Signature format analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureFormatAnalysis {
    /// Whether format is valid RFC 9421
    pub is_valid_rfc9421: bool,
    /// Format issues found
    pub issues: Vec<FormatIssue>,
    /// Signature headers found
    pub signature_headers: Vec<String>,
    /// Detected signature identifiers
    pub signature_ids: Vec<String>,
}

/// Format issue description
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatIssue {
    /// Issue severity
    pub severity: FormatIssueSeverity,
    /// Issue code
    pub code: String,
    /// Human-readable message
    pub message: String,
    /// Affected header or component
    pub component: Option<String>,
}

/// Format issue severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormatIssueSeverity {
    Error,
    Warning,
    Info,
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(verifier.config.policies.len() >= 3);
    }

    #[test]
    fn test_add_public_key() {
        let mut verifier = CliSignatureVerifier::with_default_config();
        let key_bytes = vec![0u8; PUBLIC_KEY_LENGTH];

        let result = verifier.add_public_key("test-key".to_string(), key_bytes);
        assert!(result.is_ok());
        assert!(verifier.config.public_keys.contains_key("test-key"));
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
}
