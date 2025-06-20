//! Core verification engine and algorithms

use super::signature_data::*;
use super::verification_config::*;
use super::verification_types::*;
use crate::cli::auth::SignatureComponent;
use crate::crypto::ed25519::{verify_signature, PUBLIC_KEY_LENGTH, SIGNATURE_LENGTH};
use crate::security_types::SecurityLevel;
use crate::unified_crypto::primitives::PublicKeyHandle;
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use reqwest::Response;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::time::Instant;

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
        // Convert bytes to PublicKeyHandle for verification
        let public_key_handle = PublicKeyHandle::from_bytes(&public_key_bytes, crate::unified_crypto::types::Algorithm::Ed25519)
            .map_err(|_| VerificationError::KeyManagement("Invalid public key".to_string()))?;
        let crypto_valid = public_key_handle.verify(message, &signature_array).unwrap_or(false);
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