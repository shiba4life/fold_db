//! Verification configuration and policy management

use crate::cli::auth::SignatureComponent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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