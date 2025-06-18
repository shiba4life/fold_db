//! Signature inspection and debugging tools

use super::signature_data::*;
use std::collections::HashMap;

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

    /// Analyze signature header compliance
    pub fn analyze_header_compliance(
        &self,
        headers: &HashMap<String, String>,
    ) -> HeaderComplianceReport {
        let mut compliance_issues = vec![];
        let mut rfc9421_score = 100u8;

        // Check signature-input header format
        if let Some(sig_input) = headers.get("signature-input") {
            if !sig_input.contains('(') || !sig_input.contains(')') {
                compliance_issues.push(ComplianceIssue {
                    category: "Format".to_string(),
                    severity: "Warning".to_string(),
                    description: "signature-input header format may not be RFC 9421 compliant".to_string(),
                });
                rfc9421_score -= 10;
            }
        }

        // Check signature header format
        if let Some(signature) = headers.get("signature") {
            // Basic base64 validation
            if !signature.chars().all(|c| c.is_alphanumeric() || "+/=".contains(c)) {
                compliance_issues.push(ComplianceIssue {
                    category: "Encoding".to_string(),
                    severity: "Error".to_string(),
                    description: "signature header contains invalid base64 characters".to_string(),
                });
                rfc9421_score -= 20;
            }
        }

        let recommendations = self.generate_compliance_recommendations(&compliance_issues);
        
        HeaderComplianceReport {
            rfc9421_score,
            compliance_issues,
            recommendations,
        }
    }

    /// Generate content analysis for debugging
    pub fn analyze_content_structure(
        &self,
        content: &[u8],
        content_type: Option<&str>,
    ) -> ContentStructureAnalysis {
        let size = content.len();
        let mut characteristics = vec![];

        // Analyze content characteristics
        if size == 0 {
            characteristics.push("Empty content".to_string());
        } else if size < 1024 {
            characteristics.push("Small content".to_string());
        } else if size > 1024 * 1024 {
            characteristics.push("Large content".to_string());
        }

        // Check for binary vs text content
        let is_binary = content.iter().take(1024).any(|&b| b < 32 && b != 9 && b != 10 && b != 13);
        if is_binary {
            characteristics.push("Binary content detected".to_string());
        } else {
            characteristics.push("Text content detected".to_string());
        }

        // Analyze content type
        let inferred_type = match content_type {
            Some(ct) if ct.starts_with("application/json") => "JSON",
            Some(ct) if ct.starts_with("text/") => "Text",
            Some(ct) if ct.starts_with("application/xml") => "XML",
            Some(ct) if ct.starts_with("application/octet-stream") => "Binary",
            Some(_) => "Other",
            None => "Unknown",
        };

        ContentStructureAnalysis {
            size,
            content_type: content_type.map(|s| s.to_string()),
            inferred_type: inferred_type.to_string(),
            characteristics,
            digest_recommendations: self.recommend_digest_algorithm(size, is_binary),
        }
    }

    /// Generate compliance recommendations
    fn generate_compliance_recommendations(&self, issues: &[ComplianceIssue]) -> Vec<String> {
        let mut recommendations = vec![];

        for issue in issues {
            match issue.category.as_str() {
                "Format" => {
                    recommendations.push(
                        "Ensure signature-input header follows RFC 9421 format: sig1=(components);parameters".to_string()
                    );
                }
                "Encoding" => {
                    recommendations.push(
                        "Use proper base64 encoding for signature values".to_string()
                    );
                }
                _ => {
                    recommendations.push(
                        format!("Address {} issue: {}", issue.category, issue.description)
                    );
                }
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Signature headers appear to be compliant with RFC 9421".to_string());
        }

        recommendations
    }

    /// Recommend digest algorithm based on content
    fn recommend_digest_algorithm(&self, size: usize, is_binary: bool) -> Vec<String> {
        let mut recommendations = vec![];

        // Default recommendation
        recommendations.push("sha-256: Standard choice for most content".to_string());

        if size > 1024 * 1024 {
            recommendations.push("Consider sha-512 for large content for additional security".to_string());
        }

        if is_binary {
            recommendations.push("Binary content detected - ensure proper encoding".to_string());
        }

        recommendations
    }
}

/// Header compliance analysis report
#[derive(Debug, Clone)]
pub struct HeaderComplianceReport {
    /// RFC 9421 compliance score (0-100)
    pub rfc9421_score: u8,
    /// List of compliance issues found
    pub compliance_issues: Vec<ComplianceIssue>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

/// Individual compliance issue
#[derive(Debug, Clone)]
pub struct ComplianceIssue {
    /// Issue category
    pub category: String,
    /// Issue severity (Error, Warning, Info)
    pub severity: String,
    /// Issue description
    pub description: String,
}

/// Content structure analysis result
#[derive(Debug, Clone)]
pub struct ContentStructureAnalysis {
    /// Content size in bytes
    pub size: usize,
    /// Declared content type
    pub content_type: Option<String>,
    /// Inferred content type
    pub inferred_type: String,
    /// Content characteristics
    pub characteristics: Vec<String>,
    /// Digest algorithm recommendations
    pub digest_recommendations: Vec<String>,
}