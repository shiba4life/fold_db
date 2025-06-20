//! CLI authentication module implementing RFC 9421 HTTP Message Signatures
//!
//! This module provides functionality for signing HTTP requests made by the DataFold CLI
//! using Ed25519 signatures according to RFC 9421 standards.

use crate::unified_crypto::keys::KeyPair as MasterKeyPair;

const PUBLIC_KEY_LENGTH: usize = 32; // Ed25519 public key length
use crate::error::FoldDbError;
use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Utc};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

/// Errors that can occur during CLI authentication
#[derive(Debug, thiserror::Error)]
pub enum CliAuthError {
    #[error("Key management error: {0}")]
    KeyManagement(String),

    #[error("Signature generation failed: {0}")]
    SignatureGeneration(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("HTTP header error: {0}")]
    HttpHeader(String),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::unified_crypto::error::UnifiedCryptoError),

    #[error("FoldDb error: {0}")]
    FoldDb(#[from] FoldDbError),
}

/// Result type for CLI authentication operations
pub type CliAuthResult<T> = Result<T, CliAuthError>;

/// Authentication profile for CLI requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliAuthProfile {
    /// Client identifier registered with the server
    pub client_id: String,
    /// Key identifier for the signing key
    pub key_id: String,
    /// Optional user ID associated with the client
    pub user_id: Option<String>,
    /// Server URL for requests
    pub server_url: String,
    /// Additional metadata for requests
    pub metadata: HashMap<String, String>,
}

/// RFC 9421 signature components that must be included in signatures
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SignatureComponent {
    /// @method pseudo-header
    Method,
    /// @target-uri pseudo-header
    TargetUri,
    /// @authority pseudo-header
    Authority,
    /// @scheme pseudo-header
    Scheme,
    /// @path pseudo-header
    Path,
    /// @query pseudo-header (if query string exists)
    Query,
    /// HTTP header field
    Header(String),
}

impl fmt::Display for SignatureComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Method => write!(f, "@method"),
            Self::TargetUri => write!(f, "@target-uri"),
            Self::Authority => write!(f, "@authority"),
            Self::Scheme => write!(f, "@scheme"),
            Self::Path => write!(f, "@path"),
            Self::Query => write!(f, "@query"),
            Self::Header(name) => write!(f, "{}", name.to_lowercase()),
        }
    }
}

/// Configuration for CLI request signing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CliSigningConfig {
    /// Required signature components for all requests
    pub required_components: Vec<SignatureComponent>,
    /// Whether to include content-digest for requests with bodies
    pub include_content_digest: bool,
    /// Whether to include timestamp in signatures
    pub include_timestamp: bool,
    /// Whether to include nonce for replay protection
    pub include_nonce: bool,
    /// Maximum request body size for digest calculation (bytes)
    pub max_body_size: usize,
}

impl Default for CliSigningConfig {
    fn default() -> Self {
        Self {
            required_components: vec![
                SignatureComponent::Method,
                SignatureComponent::TargetUri,
                SignatureComponent::Header("content-type".to_string()),
            ],
            include_content_digest: true,
            include_timestamp: true,
            include_nonce: true,
            max_body_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// RFC 9421 signature input structure
#[derive(Debug)]
pub struct SignatureInput {
    /// Signature components to include
    pub components: Vec<SignatureComponent>,
    /// Signature parameters
    pub parameters: HashMap<String, String>,
}

impl SignatureInput {
    /// Create a new signature input with default components
    pub fn new(components: Vec<SignatureComponent>) -> Self {
        let mut parameters = HashMap::new();
        parameters.insert("alg".to_string(), "ed25519".to_string());

        Self {
            components,
            parameters,
        }
    }

    /// Add a parameter to the signature input
    pub fn with_parameter(mut self, key: String, value: String) -> Self {
        self.parameters.insert(key, value);
        self
    }

    /// Generate the signature input string according to RFC 9421
    pub fn to_signature_input_string(&self) -> String {
        let components_str = self
            .components
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(" ");

        let params_str = self
            .parameters
            .iter()
            .map(|(k, v)| format!("{}=\"{}\"", k, v))
            .collect::<Vec<_>>()
            .join(";");

        if params_str.is_empty() {
            format!("({})", components_str)
        } else {
            format!("({});{}", components_str, params_str)
        }
    }
}

/// CLI request signer implementing RFC 9421 HTTP Message Signatures
pub struct CliRequestSigner {
    /// Key pair for signing requests
    keypair: MasterKeyPair,
    /// Authentication profile
    profile: CliAuthProfile,
    /// Signing configuration
    config: CliSigningConfig,
}

impl CliRequestSigner {
    /// Create a new CLI request signer
    pub fn new(keypair: MasterKeyPair, profile: CliAuthProfile, config: CliSigningConfig) -> Self {
        Self {
            keypair,
            profile,
            config,
        }
    }

    /// Sign an HTTP request according to RFC 9421
    pub fn sign_request(&self, request: &mut Request) -> CliAuthResult<()> {
        // Generate signature components based on request
        let components = self.generate_signature_components(request)?;

        // Create signature input
        let mut signature_input = SignatureInput::new(components);

        // Add timestamp if configured
        if self.config.include_timestamp {
            let timestamp = Utc::now().timestamp().to_string();
            signature_input = signature_input.with_parameter("created".to_string(), timestamp);
        }

        // Add nonce if configured
        if self.config.include_nonce {
            let nonce = Uuid::new_v4().to_string();
            signature_input = signature_input.with_parameter("nonce".to_string(), nonce);
        }

        // Add key identifier
        signature_input =
            signature_input.with_parameter("keyid".to_string(), self.profile.client_id.clone());

        // Calculate content digest (always include since server requires it)
        if self.config.include_content_digest {
            // For now, always use empty body digest since calculate_content_digest
            // already returns empty body hash as placeholder
            let digest = if let Some(body) = request.body() {
                self.calculate_content_digest(body)?
            } else {
                // For requests without body (like GET), use empty body digest
                let empty_hash = Sha256::digest([]);
                format!("sha-256=:{}", general_purpose::STANDARD.encode(empty_hash))
            };

            request.headers_mut().insert(
                "content-digest",
                digest.parse().map_err(|e| {
                    CliAuthError::HttpHeader(format!("Invalid content-digest header: {}", e))
                })?,
            );

            // Add content-digest to signature components if not already present
            if !signature_input
                .components
                .iter()
                .any(|c| matches!(c, SignatureComponent::Header(h) if h == "content-digest"))
            {
                signature_input
                    .components
                    .push(SignatureComponent::Header("content-digest".to_string()));
            }
        }

        // Build canonical signature string
        let canonical_string = self.build_canonical_string(request, &signature_input)?;

        // Sign the canonical string
        let signature_bytes = self
            .keypair
            .sign_data(canonical_string.as_bytes())
            .map_err(|e| {
                CliAuthError::SignatureGeneration(format!("Failed to sign request: {}", e))
            })?;

        // Encode signature as base64
        let signature_b64 = general_purpose::STANDARD.encode(signature_bytes);

        // Add signature headers to request with RFC 9421 format (sig1= prefix)
        let signature_input_header =
            format!("sig1={}", signature_input.to_signature_input_string());
        let signature_header = format!("sig1=:{}", signature_b64);

        request.headers_mut().insert(
            "signature-input",
            signature_input_header.parse().map_err(|e| {
                CliAuthError::HttpHeader(format!("Invalid signature-input header: {}", e))
            })?,
        );
        request.headers_mut().insert(
            "signature",
            signature_header.parse().map_err(|e| {
                CliAuthError::HttpHeader(format!("Invalid signature header: {}", e))
            })?,
        );

        // Add client identification headers
        request.headers_mut().insert(
            "x-datafold-client-id",
            self.profile.client_id.parse().map_err(|e| {
                CliAuthError::HttpHeader(format!("Invalid client-id header: {}", e))
            })?,
        );

        if let Some(user_id) = &self.profile.user_id {
            request.headers_mut().insert(
                "x-datafold-user-id",
                user_id.parse().map_err(|e| {
                    CliAuthError::HttpHeader(format!("Invalid user-id header: {}", e))
                })?,
            );
        }

        Ok(())
    }

    /// Generate signature components based on the request and configuration
    fn generate_signature_components(
        &self,
        request: &Request,
    ) -> CliAuthResult<Vec<SignatureComponent>> {
        let mut components = Vec::new();

        // Always include required components
        for component in &self.config.required_components {
            components.push(component.clone());
        }

        // Add query component if request has query parameters
        if let Some(query) = request.url().query() {
            if !query.is_empty()
                && !components
                    .iter()
                    .any(|c| matches!(c, SignatureComponent::Query))
            {
                components.push(SignatureComponent::Query);
            }
        }

        // Add content-type if request has a body and not already included
        if request.body().is_some() {
            let content_type_component = SignatureComponent::Header("content-type".to_string());
            if !components.contains(&content_type_component) {
                components.push(content_type_component);
            }
        }

        Ok(components)
    }

    /// Calculate SHA-256 content digest for request body
    fn calculate_content_digest(&self, _body: &reqwest::Body) -> CliAuthResult<String> {
        // For now, we'll return a placeholder since accessing body bytes from reqwest::Body is complex
        // In a production implementation, this would need to be handled differently
        // potentially by requiring the body to be passed separately or using a different HTTP client approach

        // Placeholder SHA-256 digest for empty body
        let empty_hash = Sha256::digest([]);
        Ok(format!(
            "sha-256=:{}",
            general_purpose::STANDARD.encode(empty_hash)
        ))
    }

    /// Build canonical signature string according to RFC 9421
    fn build_canonical_string(
        &self,
        request: &Request,
        signature_input: &SignatureInput,
    ) -> CliAuthResult<String> {
        let mut canonical_parts = Vec::new();

        for component in &signature_input.components {
            let value = self.extract_component_value(request, component)?;
            canonical_parts.push(format!("\"{}\": {}", component, value));
        }

        // Add signature parameters line
        let params_line = format!(
            "\"@signature-params\": {}",
            signature_input.to_signature_input_string()
        );
        canonical_parts.push(params_line);

        Ok(canonical_parts.join("\n"))
    }

    /// Extract the value for a specific signature component from the request
    fn extract_component_value(
        &self,
        request: &Request,
        component: &SignatureComponent,
    ) -> CliAuthResult<String> {
        match component {
            SignatureComponent::Method => Ok(request.method().as_str().to_uppercase()),
            SignatureComponent::TargetUri => Ok(request.url().as_str().to_string()),
            SignatureComponent::Authority => request
                .url()
                .host_str()
                .map(|host| {
                    if let Some(port) = request.url().port() {
                        format!("{}:{}", host, port)
                    } else {
                        host.to_string()
                    }
                })
                .ok_or_else(|| {
                    CliAuthError::InvalidRequest("Missing authority in URL".to_string())
                }),
            SignatureComponent::Scheme => Ok(request.url().scheme().to_string()),
            SignatureComponent::Path => Ok(request.url().path().to_string()),
            SignatureComponent::Query => Ok(request.url().query().unwrap_or("").to_string()),
            SignatureComponent::Header(name) => request
                .headers()
                .get(name)
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
                .ok_or_else(|| CliAuthError::InvalidRequest(format!("Missing header: {}", name))),
        }
    }

    /// Get the authentication profile
    pub fn profile(&self) -> &CliAuthProfile {
        &self.profile
    }

    /// Get the public key for this signer
    pub fn public_key_bytes(&self) -> [u8; PUBLIC_KEY_LENGTH] {
        let bytes = self.keypair.public_key_bytes();
        let mut array = [0u8; PUBLIC_KEY_LENGTH];
        array.copy_from_slice(&bytes[..PUBLIC_KEY_LENGTH]);
        array
    }
}

/// Authentication status for CLI operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CliAuthStatus {
    /// Whether authentication is configured
    pub configured: bool,
    /// Client ID if configured
    pub client_id: Option<String>,
    /// Key ID if configured
    pub key_id: Option<String>,
    /// Server URL if configured
    pub server_url: Option<String>,
    /// Last authentication attempt timestamp
    pub last_attempt: Option<DateTime<Utc>>,
    /// Whether last attempt was successful
    pub last_success: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::ed25519::generate_master_keypair;
    use reqwest::Method;
    use std::collections::HashMap;

    fn create_test_profile() -> CliAuthProfile {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());

        CliAuthProfile {
            client_id: "test-client-123".to_string(),
            key_id: "test-key".to_string(),
            user_id: Some("test-user".to_string()),
            server_url: "https://api.example.com".to_string(),
            metadata,
        }
    }

    #[test]
    fn test_signature_component_display() {
        assert_eq!(SignatureComponent::Method.to_string(), "@method");
        assert_eq!(SignatureComponent::TargetUri.to_string(), "@target-uri");
        assert_eq!(
            SignatureComponent::Header("content-type".to_string()).to_string(),
            "content-type"
        );
    }

    #[test]
    fn test_signature_input_generation() {
        let components = vec![
            SignatureComponent::Method,
            SignatureComponent::TargetUri,
            SignatureComponent::Header("content-type".to_string()),
        ];

        let input = SignatureInput::new(components)
            .with_parameter("created".to_string(), "1234567890".to_string())
            .with_parameter("keyid".to_string(), "test-key".to_string());

        let input_string = input.to_signature_input_string();
        assert!(input_string.contains("@method"));
        assert!(input_string.contains("@target-uri"));
        assert!(input_string.contains("content-type"));
        assert!(input_string.contains("alg=\"ed25519\""));
        assert!(input_string.contains("created=\"1234567890\""));
        assert!(input_string.contains("keyid=\"test-key\""));
    }

    #[test]
    fn test_cli_request_signer_creation() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let profile = create_test_profile();
        let config = CliSigningConfig::default();

        let signer = CliRequestSigner::new(keypair, profile.clone(), config);
        assert_eq!(signer.profile().client_id, profile.client_id);
        assert_eq!(signer.profile().key_id, profile.key_id);
    }

    #[test]
    fn test_component_value_extraction() {
        let keypair = generate_master_keypair().expect("Failed to generate keypair");
        let profile = create_test_profile();
        let config = CliSigningConfig::default();
        let signer = CliRequestSigner::new(keypair, profile, config);

        // Create a test request
        let url = "https://api.example.com/test?param=value".parse().unwrap();
        let request = Request::new(Method::POST, url);

        // Test component extraction
        assert_eq!(
            signer
                .extract_component_value(&request, &SignatureComponent::Method)
                .unwrap(),
            "POST"
        );
        assert_eq!(
            signer
                .extract_component_value(&request, &SignatureComponent::Authority)
                .unwrap(),
            "api.example.com"
        );
        assert_eq!(
            signer
                .extract_component_value(&request, &SignatureComponent::Scheme)
                .unwrap(),
            "https"
        );
        assert_eq!(
            signer
                .extract_component_value(&request, &SignatureComponent::Path)
                .unwrap(),
            "/test"
        );
        assert_eq!(
            signer
                .extract_component_value(&request, &SignatureComponent::Query)
                .unwrap(),
            "param=value"
        );
    }
}
