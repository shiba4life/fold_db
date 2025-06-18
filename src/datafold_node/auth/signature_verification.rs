//! Core signature verification algorithms and logic for DataFold node
//!
//! This module implements RFC 9421-compliant HTTP message signatures using Ed25519
//! cryptography for authentication and replay prevention with comprehensive security validation.

use crate::datafold_node::error::NodeResult;
use crate::error::FoldDbError;
use actix_web::dev::ServiceRequest;
use log::{debug, error, warn};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use super::auth_errors::AuthenticationError;
pub use super::auth_types::SignatureComponents;

/// Parse signature components from HTTP headers
impl SignatureComponents {
    /// Parse signature components from HTTP headers
    pub fn parse_from_headers(req: &ServiceRequest) -> NodeResult<Self> {
        // Extract Signature-Input header
        let signature_input = req
            .headers()
            .get("signature-input")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| FoldDbError::Permission("Missing Signature-Input header".to_string()))?;

        // Extract Signature header
        let signature = req
            .headers()
            .get("signature")
            .and_then(|h| h.to_str().ok())
            .ok_or_else(|| FoldDbError::Permission("Missing Signature header".to_string()))?;

        // Parse signature input components
        let (covered_components, params) = Self::parse_signature_input(signature_input)?;

        // Extract required parameters
        let created = params
            .get("created")
            .ok_or_else(|| FoldDbError::Permission("Missing 'created' parameter".to_string()))?
            .trim_matches('"')
            .parse::<u64>()
            .map_err(|_| FoldDbError::Permission("Invalid 'created' timestamp".to_string()))?;

        let keyid = params
            .get("keyid")
            .ok_or_else(|| FoldDbError::Permission("Missing 'keyid' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let algorithm = params
            .get("alg")
            .ok_or_else(|| FoldDbError::Permission("Missing 'alg' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        let nonce = params
            .get("nonce")
            .ok_or_else(|| FoldDbError::Permission("Missing 'nonce' parameter".to_string()))?
            .trim_matches('"')
            .to_string();

        // Validate algorithm
        if algorithm != "ed25519" {
            return Err(FoldDbError::Permission(format!(
                "Unsupported algorithm: {}",
                algorithm
            )));
        }

        Ok(Self {
            signature_input: signature_input.to_string(),
            signature: signature.to_string(),
            created,
            keyid,
            algorithm,
            nonce,
            covered_components,
        })
    }

    /// Parse the signature-input header to extract covered components and parameters
    pub fn parse_signature_input(input: &str) -> NodeResult<(Vec<String>, HashMap<String, String>)> {
        // Find the signature name and its definition
        // Format: sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key-ed25519";alg="ed25519";nonce="abc123"

        let parts: Vec<&str> = input.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(FoldDbError::Permission(
                "Invalid signature-input format".to_string(),
            ));
        }

        let definition = parts[1];

        // Split on semicolon to get components and parameters
        let mut components = Vec::new();
        let mut params = HashMap::new();

        let sections: Vec<&str> = definition.split(';').collect();

        // First section should be the covered components in parentheses
        if let Some(components_str) = sections.first() {
            let components_str = components_str.trim();
            if !components_str.starts_with('(') || !components_str.ends_with(')') {
                return Err(FoldDbError::Permission(
                    "Invalid components format".to_string(),
                ));
            }

            let inner = &components_str[1..components_str.len() - 1];
            for component in inner.split_whitespace() {
                components.push(component.trim_matches('"').to_string());
            }
        }

        // Parse parameters
        for section in sections.iter().skip(1) {
            let param_parts: Vec<&str> = section.splitn(2, '=').collect();
            if param_parts.len() == 2 {
                let key = param_parts[0].trim();
                let value = param_parts[1].trim();
                params.insert(key.to_string(), value.to_string());
            }
        }

        Ok((components, params))
    }

    /// Construct the canonical message for signature verification
    pub fn construct_canonical_message(&self, req: &ServiceRequest) -> NodeResult<String> {
        let mut lines = Vec::new();

        for component in &self.covered_components {
            let line = match component.as_str() {
                "@method" => {
                    format!("\"@method\": {}", req.method().as_str())
                }
                "@target-uri" => {
                    let uri = req.uri();
                    let target_uri = format!(
                        "{}{}",
                        uri.path(),
                        uri.query().map(|q| format!("?{}", q)).unwrap_or_default()
                    );
                    format!("\"@target-uri\": {}", target_uri)
                }
                header_name => {
                    // Regular header
                    let header_value = req
                        .headers()
                        .get(header_name)
                        .and_then(|h| h.to_str().ok())
                        .unwrap_or("");
                    format!("\"{}\": {}", header_name, header_value)
                }
            };
            lines.push(line);
        }

        // Add signature parameters
        lines.push(format!(
            "\"@signature-params\": {}",
            self.build_signature_params()
        ));

        Ok(lines.join("\n"))
    }

    /// Build the signature parameters line
    fn build_signature_params(&self) -> String {
        let components_str = self
            .covered_components
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(" ");

        format!("({})", components_str)
    }
}

/// Core signature verification functionality
pub struct SignatureVerifier;

impl SignatureVerifier {
    /// Verify a request signature against a public key
    pub async fn verify_signature_against_database(
        components: &SignatureComponents,
        req: &actix_web::dev::ServiceRequest,
        app_state: &crate::datafold_node::routes::http_server::AppState,
    ) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        // Get database access and extract db_ops in a scoped block
        let db_ops = {
            let node = app_state.node.lock().await;
            let db_guard = match node.get_db() {
                Ok(guard) => guard,
                Err(_) => {
                    return Err(AuthenticationError::ConfigurationError {
                        reason: "Cannot access database".to_string(),
                        correlation_id,
                    });
                }
            };
            db_guard.db_ops()
        }; // db_guard and node are dropped here

        // Look up the public key
        let public_key_bytes = match Self::lookup_public_key(&components.keyid, db_ops, &correlation_id)
            .await
        {
            Ok(key) => key,
            Err(e) => return Err(e),
        };

        // Construct the canonical message
        let canonical_message = match components.construct_canonical_message(req) {
            Ok(message) => message,
            Err(e) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Failed to construct canonical message: {}", e),
                    correlation_id,
                });
            }
        };

        // Decode the signature
        let signature_bytes = match hex::decode(&components.signature) {
            Ok(bytes) => {
                if bytes.len() != crate::crypto::ed25519::SIGNATURE_LENGTH {
                    return Err(AuthenticationError::InvalidSignatureFormat {
                        reason: format!(
                            "Invalid signature length: expected {}, got {}",
                            crate::crypto::ed25519::SIGNATURE_LENGTH,
                            bytes.len()
                        ),
                        correlation_id,
                    });
                }
                // Convert to fixed-size array
                let mut signature_array = [0u8; crate::crypto::ed25519::SIGNATURE_LENGTH];
                signature_array.copy_from_slice(&bytes);
                signature_array
            }
            Err(_) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: "Invalid hex encoding in signature".to_string(),
                    correlation_id,
                });
            }
        };

        // Verify the signature using Ed25519
        let public_key = match crate::crypto::ed25519::PublicKey::from_bytes(&public_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Invalid public key format: {}", e),
                    correlation_id,
                });
            }
        };

        match public_key.verify(canonical_message.as_bytes(), &signature_bytes) {
            Ok(()) => {
                debug!(
                    "Signature verification successful for key_id: {}",
                    components.keyid
                );
                Ok(())
            }
            Err(e) => {
                warn!(
                    "Signature verification failed for key_id: {}: {}",
                    components.keyid, e
                );
                Err(AuthenticationError::SignatureVerificationFailed {
                    key_id: components.keyid.clone(),
                    correlation_id,
                })
            }
        }
    }

    /// Look up a public key from the database
    async fn lookup_public_key(
        key_id: &str,
        db_ops: std::sync::Arc<crate::db_operations::core::DbOperations>,
        correlation_id: &str,
    ) -> Result<[u8; 32], AuthenticationError> {
        use crate::datafold_node::crypto::crypto_routes::{
            CLIENT_KEY_INDEX_TREE, PUBLIC_KEY_REGISTRATIONS_TREE,
        };

        // Look up registration ID by client ID using the same pattern as crypto_routes
        let client_index_key = format!("{}:{}", CLIENT_KEY_INDEX_TREE, key_id);
        let registration_id_str = match db_ops.get_item::<String>(&client_index_key) {
            Ok(Some(reg_id)) => reg_id,
            Ok(None) => {
                debug!("No registration found for client_id: {}", key_id);
                return Err(AuthenticationError::PublicKeyLookupFailed {
                    key_id: key_id.to_string(),
                    correlation_id: correlation_id.to_string(),
                });
            }
            Err(e) => {
                error!("Failed to lookup client key index: {}", e);
                return Err(AuthenticationError::PublicKeyLookupFailed {
                    key_id: key_id.to_string(),
                    correlation_id: correlation_id.to_string(),
                });
            }
        };

        // Get registration record using the same pattern as crypto_routes
        let registration_key =
            format!("{}:{}", PUBLIC_KEY_REGISTRATIONS_TREE, &registration_id_str);
        let registration: crate::datafold_node::crypto::crypto_routes::PublicKeyRegistration =
            match db_ops.get_item(&registration_key) {
                Ok(Some(reg)) => reg,
                Ok(None) => {
                    debug!("Registration record not found: {}", registration_id_str);
                    return Err(AuthenticationError::PublicKeyLookupFailed {
                        key_id: key_id.to_string(),
                        correlation_id: correlation_id.to_string(),
                    });
                }
                Err(e) => {
                    error!("Failed to get registration record: {}", e);
                    return Err(AuthenticationError::PublicKeyLookupFailed {
                        key_id: key_id.to_string(),
                        correlation_id: correlation_id.to_string(),
                    });
                }
            };

        // Check if the key is active
        if registration.status != "active" {
            debug!(
                "Public key for {} is not active: {}",
                key_id, registration.status
            );
            return Err(AuthenticationError::PublicKeyLookupFailed {
                key_id: key_id.to_string(),
                correlation_id: correlation_id.to_string(),
            });
        }

        debug!(
            "Successfully found active public key for client_id: {}",
            key_id
        );
        Ok(registration.public_key_bytes)
    }

    /// Generate correlation ID for request tracking
    fn generate_correlation_id() -> String {
        Uuid::new_v4().to_string()
    }

    /// Validate signature algorithm
    pub fn validate_algorithm(algorithm: &str) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();
        
        if algorithm != "ed25519" {
            return Err(AuthenticationError::UnsupportedAlgorithm {
                algorithm: algorithm.to_string(),
                correlation_id,
            });
        }
        
        Ok(())
    }

    /// Validate signature format (hex encoding, length, etc.)
    pub fn validate_signature_format(signature: &str) -> Result<Vec<u8>, AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        if signature.is_empty() {
            return Err(AuthenticationError::InvalidSignatureFormat {
                reason: "Signature cannot be empty".to_string(),
                correlation_id,
            });
        }

        // Check for valid hex characters
        if !signature.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AuthenticationError::InvalidSignatureFormat {
                reason: "Signature contains non-hexadecimal characters".to_string(),
                correlation_id,
            });
        }

        // Decode hex
        let signature_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: "Invalid hex encoding in signature".to_string(),
                    correlation_id,
                });
            }
        };

        // Check length for Ed25519
        if signature_bytes.len() != crate::crypto::ed25519::SIGNATURE_LENGTH {
            return Err(AuthenticationError::InvalidSignatureFormat {
                reason: format!(
                    "Invalid signature length: expected {}, got {}",
                    crate::crypto::ed25519::SIGNATURE_LENGTH,
                    signature_bytes.len()
                ),
                correlation_id,
            });
        }

        Ok(signature_bytes)
    }

    /// Validate required signature components are covered
    pub fn validate_signature_components(
        components: &SignatureComponents,
        required_components: &[String],
    ) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        for required_component in required_components {
            if !components.covered_components.contains(required_component) {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!(
                        "Required component '{}' not covered by signature",
                        required_component
                    ),
                    correlation_id,
                });
            }
        }
        Ok(())
    }

    /// Verify signature against public key bytes (for testing)
    pub fn verify_signature_direct(
        canonical_message: &str,
        signature_bytes: &[u8; crate::crypto::ed25519::SIGNATURE_LENGTH],
        public_key_bytes: &[u8; 32],
    ) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        let public_key = match crate::crypto::ed25519::PublicKey::from_bytes(public_key_bytes) {
            Ok(key) => key,
            Err(e) => {
                return Err(AuthenticationError::InvalidSignatureFormat {
                    reason: format!("Invalid public key format: {}", e),
                    correlation_id,
                });
            }
        };

        match public_key.verify(canonical_message.as_bytes(), signature_bytes) {
            Ok(()) => {
                debug!("Direct signature verification successful");
                Ok(())
            }
            Err(e) => {
                warn!("Direct signature verification failed: {}", e);
                Err(AuthenticationError::SignatureVerificationFailed {
                    key_id: "direct".to_string(),
                    correlation_id,
                })
            }
        }
    }

    /// Validate timestamp with enhanced error details
    pub fn validate_timestamp_enhanced(
        created: u64,
        allowed_time_window_secs: u64,
        clock_skew_tolerance_secs: u64,
        max_future_timestamp_secs: u64,
    ) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| AuthenticationError::ConfigurationError {
                reason: "System time error".to_string(),
                correlation_id: correlation_id.clone(),
            })?
            .as_secs();

        // Check for future timestamps
        if created > now {
            let future_diff = created - now;
            if future_diff > max_future_timestamp_secs {
                return Err(AuthenticationError::TimestampValidationFailed {
                    timestamp: created,
                    current_time: now,
                    reason: format!("Timestamp too far in future: {} seconds", future_diff),
                    correlation_id,
                });
            }

            // Allow small future timestamps within clock skew tolerance
            if future_diff <= clock_skew_tolerance_secs {
                debug!(
                    "Accepting future timestamp within clock skew tolerance: {}s",
                    future_diff
                );
                return Ok(());
            }
        }

        // Check for past timestamps
        let time_diff = if now >= created {
            now - created
        } else {
            created - now
        };

        let effective_window = allowed_time_window_secs + clock_skew_tolerance_secs;

        if time_diff > effective_window {
            return Err(AuthenticationError::TimestampValidationFailed {
                timestamp: created,
                current_time: now,
                reason: format!(
                    "Timestamp outside allowed window: {} seconds (max: {})",
                    time_diff, effective_window
                ),
                correlation_id,
            });
        }

        Ok(())
    }

    /// Simple RFC 3339 format validation
    pub fn is_valid_rfc3339_format(timestamp_str: &str) -> bool {
        // Basic pattern check for RFC 3339: YYYY-MM-DDTHH:MM:SSZ
        if timestamp_str.len() < 19 {
            return false;
        }

        let chars: Vec<char> = timestamp_str.chars().collect();

        // Check basic structure: YYYY-MM-DDTHH:MM:SS
        chars.get(4) == Some(&'-')
            && chars.get(7) == Some(&'-')
            && chars.get(10) == Some(&'T')
            && chars.get(13) == Some(&':')
            && chars.get(16) == Some(&':')
            && chars[0..4].iter().all(|c| c.is_ascii_digit())
            && chars[5..7].iter().all(|c| c.is_ascii_digit())
            && chars[8..10].iter().all(|c| c.is_ascii_digit())
            && chars[11..13].iter().all(|c| c.is_ascii_digit())
            && chars[14..16].iter().all(|c| c.is_ascii_digit())
            && chars[17..19].iter().all(|c| c.is_ascii_digit())
    }

    /// Validate nonce format (UUID4 if required)
    pub fn validate_nonce_format(nonce: &str, require_uuid4: bool) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        if require_uuid4 {
            Self::validate_uuid4_format(nonce)?;
            return Ok(()); // If UUID4 validation passes, we're done
        }

        // Basic validation for non-UUID nonces
        if nonce.is_empty() {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Nonce cannot be empty".to_string(),
                correlation_id,
            });
        }

        if nonce.len() > 128 {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Nonce too long (max 128 characters)".to_string(),
                correlation_id,
            });
        }

        // Ensure nonce contains only safe characters (alphanumeric, hyphens, underscores)
        if !nonce
            .chars()
            .all(|c| c.is_alphanumeric() || "-_".contains(c))
        {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Nonce contains invalid characters (only alphanumeric, -, _ allowed)".to_string(),
                correlation_id,
            });
        }

        Ok(())
    }

    /// Simple UUID4 format validation without external dependencies
    fn validate_uuid4_format(nonce: &str) -> Result<(), AuthenticationError> {
        let correlation_id = Self::generate_correlation_id();

        // UUID4 format: 8-4-4-4-12 hexadecimal digits with hyphens
        // Example: 550e8400-e29b-41d4-a716-446655440000
        if nonce.len() != 36 {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "UUID must be 36 characters long".to_string(),
                correlation_id,
            });
        }

        let chars: Vec<char> = nonce.chars().collect();

        // Check hyphen positions
        if chars[8] != '-' || chars[13] != '-' || chars[18] != '-' || chars[23] != '-' {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Invalid UUID format".to_string(),
                correlation_id,
            });
        }

        // Check that version is 4 (position 14)
        if chars[14] != '4' {
            return Err(AuthenticationError::NonceValidationFailed {
                nonce: nonce.to_string(),
                reason: "Nonce must be UUID version 4".to_string(),
                correlation_id,
            });
        }

        // Check that all other characters are hexadecimal
        for (i, &c) in chars.iter().enumerate() {
            if i == 8 || i == 13 || i == 18 || i == 23 {
                continue; // Skip hyphens
            }
            if !c.is_ascii_hexdigit() {
                return Err(AuthenticationError::NonceValidationFailed {
                    nonce: nonce.to_string(),
                    reason: "UUID contains non-hexadecimal characters".to_string(),
                    correlation_id,
                });
            }
        }

        Ok(())
    }
}

/// Verify the signature of an HTTP request (legacy compatibility function)
#[allow(dead_code)]
pub async fn verify_request_signature(
    req: &ServiceRequest,
    app_state: &crate::datafold_node::routes::http_server::AppState,
    required_signature_components: &[String],
) -> NodeResult<String> {
    // Parse signature components from headers
    let components = SignatureComponents::parse_from_headers(req)?;

    // Validate required signature components are covered
    for required_component in required_signature_components {
        if !components.covered_components.contains(required_component) {
            return Err(FoldDbError::Permission(format!(
                "Required component '{}' not covered by signature",
                required_component
            )));
        }
    }

    // Verify the signature against the stored public key
    match SignatureVerifier::verify_signature_against_database(&components, req, app_state)
        .await
    {
        Ok(()) => {
            debug!(
                "Signature verification successful for client: {}",
                components.keyid
            );
            Ok(components.keyid)
        }
        Err(auth_error) => {
            warn!(
                "Signature verification failed for client {}: {}",
                components.keyid, auth_error
            );
            Err(FoldDbError::Permission(format!(
                "Signature verification failed: {}",
                auth_error
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_input_parsing() {
        let input = r#"sig1=("@method" "@target-uri" "content-type");created=1618884473;keyid="test-key";alg="ed25519";nonce="abc123""#;
        
        let (components, params) = SignatureComponents::parse_signature_input(input).unwrap();
        
        assert_eq!(components, vec!["@method", "@target-uri", "content-type"]);
        assert_eq!(params.get("created"), Some(&"1618884473".to_string()));
        assert_eq!(params.get("keyid"), Some(&"\"test-key\"".to_string()));
        assert_eq!(params.get("alg"), Some(&"\"ed25519\"".to_string()));
        assert_eq!(params.get("nonce"), Some(&"\"abc123\"".to_string()));
    }

    #[test]
    fn test_signature_input_parsing_invalid_format() {
        let invalid_input = "invalid-format";
        let result = SignatureComponents::parse_signature_input(invalid_input);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_algorithm() {
        assert!(SignatureVerifier::validate_algorithm("ed25519").is_ok());
        assert!(SignatureVerifier::validate_algorithm("rsa").is_err());
        assert!(SignatureVerifier::validate_algorithm("").is_err());
    }

    #[test]
    fn test_validate_signature_format() {
        // Valid hex signature of correct length
        let valid_sig = "a".repeat(128); // 64 bytes = 128 hex chars
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
    fn test_validate_nonce_format() {
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
    fn test_validate_uuid4_format() {
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
    fn test_rfc3339_format_validation() {
        // Valid RFC 3339 format
        assert!(SignatureVerifier::is_valid_rfc3339_format("2023-01-01T12:00:00Z"));
        assert!(SignatureVerifier::is_valid_rfc3339_format("2023-12-31T23:59:59"));

        // Invalid formats
        assert!(!SignatureVerifier::is_valid_rfc3339_format("not-a-date"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format("2023-1-1T12:00:00Z"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format("23-01-01T12:00:00Z"));
        assert!(!SignatureVerifier::is_valid_rfc3339_format(""));
    }
}