//! Signature data structures and content digest handling

use crate::cli::auth::SignatureComponent;
use crate::security_types::SecurityLevel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::verification_types::VerificationStatus;

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