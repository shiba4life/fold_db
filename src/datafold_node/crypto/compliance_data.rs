//! Compliance report data structures and analysis types
//!
//! This module contains all the data structures used in compliance reports,
//! including statistics, assessments, and analysis results.

use crate::crypto::key_rotation::RotationReason;
use crate::crypto::key_rotation_audit::{
    RotationAuditCorrelation, TamperProofAuditEntry,
};
use crate::reporting::types::{
    SecurityIncidentSummary, UnifiedSummarySection,
};
use crate::security_types::Severity;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use super::compliance_frameworks::{ComplianceControl, ComplianceFramework};
use super::compliance_reporting::{ExportMetadata, ReportMetadata};

/// Compliance status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Fully compliant
    Compliant,
    /// Mostly compliant with minor issues
    MostlyCompliant,
    /// Partially compliant with significant issues
    PartiallyCompliant,
    /// Non-compliant
    NonCompliant,
    /// Under review
    UnderReview,
}

/// Compliance report data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Executive summary
    pub executive_summary: Option<crate::reporting::types::ExecutiveSummary>,
    /// Key rotation statistics
    pub rotation_statistics: Option<RotationStatistics>,
    /// Security incidents
    pub security_incidents: Option<SecurityIncidentSummary>,
    /// Control assessment
    pub control_assessment: Option<ControlAssessment>,
    /// Audit trail data
    pub audit_trail: Option<crate::reporting::types::AuditTrailSummary>,
    /// Risk assessment
    pub risk_assessment: Option<RiskAssessmentSummary>,
    /// Policy compliance
    pub policy_compliance: Option<PolicyComplianceSummary>,
    /// Recommendations
    pub recommendations: Option<RecommendationsSummary>,
    /// Raw audit data (if requested)
    pub raw_data: Option<RawAuditData>,
    /// Digital signature (if required)
    pub digital_signature: Option<String>,
}

/// Key rotation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationStatistics {
    /// Total rotations performed
    pub total_rotations: u64,
    /// Successful rotations
    pub successful_rotations: u64,
    /// Failed rotations
    pub failed_rotations: u64,
    /// Rotations by reason
    pub rotations_by_reason: HashMap<RotationReason, u64>,
    /// Average rotation time
    pub average_rotation_time: Duration,
    /// Peak rotation periods
    pub peak_periods: Vec<PeakPeriod>,
    /// User activity summary
    pub user_activity: HashMap<String, u64>,
}

/// Peak activity period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakPeriod {
    /// Start of peak period
    pub start_time: DateTime<Utc>,
    /// End of peak period
    pub end_time: DateTime<Utc>,
    /// Number of rotations during period
    pub rotation_count: u64,
    /// Reason for peak activity
    pub reason: Option<String>,
}

/// Control assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlAssessment {
    /// Overall control effectiveness
    pub overall_effectiveness: f64,
    /// Control results by category
    pub control_results: HashMap<ComplianceControl, ControlResult>,
    /// Framework-specific assessments
    pub framework_assessments: HashMap<ComplianceFramework, f64>,
    /// Control gaps identified
    pub control_gaps: Vec<ControlGap>,
}

/// Individual control assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlResult {
    /// Control effectiveness score (0-100)
    pub effectiveness_score: f64,
    /// Implementation status
    pub implementation_status: ImplementationStatus,
    /// Evidence collected
    pub evidence_count: u64,
    /// Issues identified
    pub issues: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Control implementation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationStatus {
    /// Fully implemented and effective
    Implemented,
    /// Partially implemented
    PartiallyImplemented,
    /// Not implemented
    NotImplemented,
    /// Implementation in progress
    InProgress,
    /// Not applicable
    NotApplicable,
}

/// Control gap identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlGap {
    /// Control category
    pub control: ComplianceControl,
    /// Gap description
    pub description: String,
    /// Risk level using unified severity classification
    pub risk_level: Severity,
    /// Recommended remediation
    pub remediation: String,
    /// Target completion date
    pub target_date: Option<DateTime<Utc>>,
}

/// Audit trail summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditTrailSummary {
    /// Total audit events
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    /// Audit trail integrity status
    pub integrity_verified: bool,
    /// Data retention compliance
    pub retention_compliant: bool,
    /// Missing or corrupted events
    pub integrity_issues: Vec<String>,
}

impl UnifiedSummarySection for AuditTrailSummary {
    fn section_name(&self) -> &'static str {
        "audit_trail"
    }
}

/// Risk assessment summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessmentSummary {
    /// Overall risk score
    pub overall_risk_score: f64,
    /// Risk by category
    pub risk_by_category: HashMap<String, f64>,
    /// High-risk areas
    pub high_risk_areas: Vec<RiskArea>,
    /// Risk trends
    pub risk_trends: RiskTrends,
    /// Mitigation effectiveness
    pub mitigation_effectiveness: f64,
}

/// Risk area identification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskArea {
    /// Area name
    pub area: String,
    /// Risk score
    pub risk_score: f64,
    /// Risk description
    pub description: String,
    /// Impact assessment
    pub impact: String,
    /// Likelihood assessment
    pub likelihood: String,
    /// Current mitigations
    pub current_mitigations: Vec<String>,
    /// Additional recommendations
    pub recommendations: Vec<String>,
}

/// Risk trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskTrends {
    /// Risk score change over time
    pub score_trend: String, // "increasing", "decreasing", "stable"
    /// Trend confidence
    pub trend_confidence: f64,
    /// Key trend drivers
    pub trend_drivers: Vec<String>,
    /// Projected risk level
    pub projected_risk: f64,
}

/// Policy compliance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyComplianceSummary {
    /// Overall compliance percentage
    pub overall_compliance: f64,
    /// Policy compliance by category
    pub policy_compliance: HashMap<String, f64>,
    /// Policy violations
    pub policy_violations: Vec<PolicyViolation>,
    /// Compliance trends
    pub compliance_trends: ComplianceTrends,
}

impl UnifiedSummarySection for PolicyComplianceSummary {
    fn section_name(&self) -> &'static str {
        "policy_compliance"
    }
}

/// Policy violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Violation ID
    pub violation_id: Uuid,
    /// Policy name
    pub policy_name: String,
    /// Violation description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Occurrence timestamp
    pub timestamp: DateTime<Utc>,
    /// Resolution status
    pub resolution_status: String,
    /// Assigned to
    pub assigned_to: Option<String>,
}

/// Compliance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTrends {
    /// Compliance score trend
    pub score_trend: String,
    /// Trend direction
    pub trend_direction: f64,
    /// Key improvement areas
    pub improvement_areas: Vec<String>,
    /// Deteriorating areas
    pub deteriorating_areas: Vec<String>,
}

/// Recommendations summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationsSummary {
    /// High priority recommendations
    pub high_priority: Vec<Recommendation>,
    /// Medium priority recommendations
    pub medium_priority: Vec<Recommendation>,
    /// Low priority recommendations
    pub low_priority: Vec<Recommendation>,
    /// Quick wins
    pub quick_wins: Vec<Recommendation>,
    /// Long-term initiatives
    pub long_term: Vec<Recommendation>,
}

/// Individual recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation ID
    pub id: Uuid,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Priority level
    pub priority: String,
    /// Estimated effort
    pub effort: String,
    /// Expected impact
    pub impact: String,
    /// Target completion
    pub target_completion: Option<DateTime<Utc>>,
    /// Responsible party
    pub responsible_party: Option<String>,
    /// Related controls
    pub related_controls: Vec<ComplianceControl>,
}

/// Raw audit data export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawAuditData {
    /// Audit correlations
    pub correlations: Vec<RotationAuditCorrelation>,
    /// Tamper-proof audit entries
    pub audit_entries: Vec<TamperProofAuditEntry>,
    /// Security detections
    pub security_detections: Vec<serde_json::Value>,
    /// Data export metadata
    pub export_metadata: ExportMetadata,
}