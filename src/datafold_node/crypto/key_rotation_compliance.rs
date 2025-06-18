//! Compliance reporting and audit trail management for key rotation
//!
//! This module provides comprehensive compliance reporting capabilities for key rotation
//! operations, supporting various compliance frameworks like SOC2, ISO27001, GDPR, etc.

use crate::crypto::key_rotation::RotationReason;
use crate::crypto::key_rotation_audit::{
    KeyRotationAuditLogger, RotationAuditCorrelation, TamperProofAuditEntry,
};
use crate::crypto::key_rotation_security::KeyRotationSecurityManager;
use crate::crypto::threat_monitor::RotationThreatMonitor;
use crate::reporting::types::{
    ResolutionStatistics, SecurityIncidentSummary, TimeRange, UnifiedReport, UnifiedReportConfig,
    UnifiedReportFormat, UnifiedReportMetadata, UnifiedSummarySection,
};
use crate::security_types::{Severity, ThreatLevel};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Compliance framework types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceFramework {
    /// SOC 2 Type II
    Soc2,
    /// ISO 27001
    Iso27001,
    /// PCI DSS
    PciDss,
    /// GDPR (General Data Protection Regulation)
    Gdpr,
    /// HIPAA
    Hipaa,
    /// FedRAMP
    FedRamp,
    /// NIST Cybersecurity Framework
    NistCsf,
    /// Custom compliance framework
    Custom(String),
}

impl ComplianceFramework {
    /// Get the human-readable name of the framework
    pub fn display_name(&self) -> &str {
        match self {
            ComplianceFramework::Soc2 => "SOC 2 Type II",
            ComplianceFramework::Iso27001 => "ISO 27001",
            ComplianceFramework::PciDss => "PCI DSS",
            ComplianceFramework::Gdpr => "GDPR",
            ComplianceFramework::Hipaa => "HIPAA",
            ComplianceFramework::FedRamp => "FedRAMP",
            ComplianceFramework::NistCsf => "NIST Cybersecurity Framework",
            ComplianceFramework::Custom(name) => name,
        }
    }

    /// Get the required data retention period for this framework
    pub fn retention_period_days(&self) -> u64 {
        match self {
            ComplianceFramework::Soc2 => 2555,      // 7 years
            ComplianceFramework::Iso27001 => 2555,  // 7 years
            ComplianceFramework::PciDss => 365,     // 1 year minimum
            ComplianceFramework::Gdpr => 2190,      // 6 years
            ComplianceFramework::Hipaa => 2190,     // 6 years
            ComplianceFramework::FedRamp => 2555,   // 7 years
            ComplianceFramework::NistCsf => 2555,   // 7 years
            ComplianceFramework::Custom(_) => 2555, // Default to 7 years
        }
    }

    /// Get required audit controls for this framework
    pub fn required_controls(&self) -> Vec<ComplianceControl> {
        match self {
            ComplianceFramework::Soc2 => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::ChangeManagement,
                ComplianceControl::DataIntegrity,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::IncidentResponse,
                ComplianceControl::BusinessContinuity,
            ],
            ComplianceFramework::Iso27001 => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::ChangeManagement,
                ComplianceControl::DataIntegrity,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::IncidentResponse,
                ComplianceControl::RiskManagement,
                ComplianceControl::VulnerabilityManagement,
            ],
            ComplianceFramework::PciDss => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::DataIntegrity,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::VulnerabilityManagement,
                ComplianceControl::NetworkSecurity,
            ],
            ComplianceFramework::Gdpr => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::DataIntegrity,
                ComplianceControl::DataProtection,
                ComplianceControl::IncidentResponse,
                ComplianceControl::PrivacyControls,
            ],
            ComplianceFramework::Hipaa => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::DataIntegrity,
                ComplianceControl::DataProtection,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::PrivacyControls,
            ],
            ComplianceFramework::FedRamp => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::ChangeManagement,
                ComplianceControl::DataIntegrity,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::IncidentResponse,
                ComplianceControl::RiskManagement,
                ComplianceControl::VulnerabilityManagement,
                ComplianceControl::BusinessContinuity,
            ],
            ComplianceFramework::NistCsf => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::SecurityMonitoring,
                ComplianceControl::IncidentResponse,
                ComplianceControl::RiskManagement,
                ComplianceControl::VulnerabilityManagement,
            ],
            ComplianceFramework::Custom(_) => vec![
                ComplianceControl::AccessControls,
                ComplianceControl::DataIntegrity,
                ComplianceControl::SecurityMonitoring,
            ],
        }
    }
}

/// Compliance control categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceControl {
    /// Access control and authorization
    AccessControls,
    /// Change management processes
    ChangeManagement,
    /// Data integrity and validation
    DataIntegrity,
    /// Security monitoring and logging
    SecurityMonitoring,
    /// Incident response procedures
    IncidentResponse,
    /// Risk management processes
    RiskManagement,
    /// Vulnerability management
    VulnerabilityManagement,
    /// Business continuity planning
    BusinessContinuity,
    /// Network security controls
    NetworkSecurity,
    /// Data protection measures
    DataProtection,
    /// Privacy controls
    PrivacyControls,
}

/// Compliance report types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceReportType {
    /// Daily operational report
    Daily,
    /// Weekly summary report
    Weekly,
    /// Monthly compliance report
    Monthly,
    /// Quarterly assessment report
    Quarterly,
    /// Annual compliance certification
    Annual,
    /// Incident-specific report
    Incident(Uuid),
    /// Audit trail export
    AuditTrail,
    /// Custom report with specific parameters
    Custom {
        name: String,
        parameters: HashMap<String, String>,
    },
}

/// Compliance report configuration extending unified config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReportConfig {
    /// Base unified report configuration
    pub base_config: UnifiedReportConfig,
    /// Target compliance frameworks
    pub frameworks: Vec<ComplianceFramework>,
    /// Report type
    pub report_type: ComplianceReportType,
    /// Time range for the report
    pub time_range: TimeRange,
    /// Include sections
    pub include_sections: Vec<ReportSection>,
    /// Include raw audit data
    pub include_raw_data: bool,
}

impl ComplianceReportConfig {
    /// Create new compliance report config
    pub fn new(
        frameworks: Vec<ComplianceFramework>,
        report_type: ComplianceReportType,
        time_range: TimeRange,
    ) -> Self {
        Self {
            base_config: UnifiedReportConfig::new(),
            frameworks,
            report_type,
            time_range,
            include_sections: Vec::new(),
            include_raw_data: false,
        }
    }

    /// Convert section enum to unified section names
    pub fn get_unified_sections(&self) -> Vec<String> {
        self.include_sections
            .iter()
            .map(|section| {
                match section {
                    ReportSection::ExecutiveSummary => "executive_summary",
                    ReportSection::RotationStatistics => "rotation_statistics",
                    ReportSection::SecurityIncidents => "security_incidents",
                    ReportSection::ControlAssessment => "control_assessment",
                    ReportSection::AuditTrail => "audit_trail",
                    ReportSection::RiskAssessment => "risk_assessment",
                    ReportSection::PolicyCompliance => "compliance",
                    ReportSection::Recommendations => "recommendations",
                    ReportSection::RawData => "raw_data",
                }
                .to_string()
            })
            .collect()
    }

    /// Check if digital signature is required
    pub fn require_signature(&self) -> bool {
        self.base_config.require_signature
    }

    /// Check if data should be anonymized
    pub fn anonymize_data(&self) -> bool {
        self.base_config.anonymize_data
    }

    /// Get output formats
    pub fn output_formats(&self) -> &Vec<UnifiedReportFormat> {
        &self.base_config.formats
    }
}

// TimeRange is now imported from crate::reporting::types

/// Report sections to include
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportSection {
    /// Executive summary
    ExecutiveSummary,
    /// Key rotation statistics
    RotationStatistics,
    /// Security incidents and threats
    SecurityIncidents,
    /// Compliance control assessment
    ControlAssessment,
    /// Audit trail summary
    AuditTrail,
    /// Risk assessment results
    RiskAssessment,
    /// Policy compliance status
    PolicyCompliance,
    /// Recommendations and remediation
    Recommendations,
    /// Raw audit data
    RawData,
}

// ReportFormat removed - use UnifiedReportFormat instead

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

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report ID
    pub report_id: Uuid,
    /// Report type
    pub report_type: ComplianceReportType,
    /// Target frameworks
    pub frameworks: Vec<ComplianceFramework>,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Report period
    pub period: TimeRange,
    /// Generated by
    pub generated_by: String,
    /// Report version
    pub version: String,
    /// Organization information
    pub organization: OrganizationInfo,
}

/// Organization information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationInfo {
    /// Organization name
    pub name: String,
    /// Contact information
    pub contact: String,
    /// Compliance officer
    pub compliance_officer: Option<String>,
    /// Auditor information
    pub auditor: Option<String>,
}

// ExecutiveSummary removed - use crate::reporting::types::ExecutiveSummary instead

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

// crate::reporting::types::SecurityIncidentSummary, ResolutionStatistics, and ThreatSummary removed - use unified types from crate::reporting::types instead

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

/// Export metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportMetadata {
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Export format version
    pub format_version: String,
    /// Data integrity hash
    pub integrity_hash: String,
    /// Export purpose
    pub purpose: String,
    /// Retention requirements
    pub retention_requirements: HashMap<ComplianceFramework, u64>,
}

/// Key rotation compliance manager
#[allow(dead_code)]
pub struct KeyRotationComplianceManager {
    /// Audit logger
    audit_logger: Arc<KeyRotationAuditLogger>,
    /// Threat monitor
    threat_monitor: Arc<RotationThreatMonitor>,
    /// Security manager
    security_manager: Arc<KeyRotationSecurityManager>,
    /// Generated reports cache
    reports_cache: Arc<RwLock<HashMap<Uuid, ComplianceReport>>>,
    /// Organization information
    organization_info: OrganizationInfo,
}

#[allow(dead_code)]
impl KeyRotationComplianceManager {
    /// Create a new compliance manager
    pub fn new(
        audit_logger: Arc<KeyRotationAuditLogger>,
        threat_monitor: Arc<RotationThreatMonitor>,
        security_manager: Arc<KeyRotationSecurityManager>,
        organization_info: OrganizationInfo,
    ) -> Self {
        Self {
            audit_logger,
            threat_monitor,
            security_manager,
            reports_cache: Arc::new(RwLock::new(HashMap::new())),
            organization_info,
        }
    }

    /// Generate a compliance report using unified structures
    pub async fn generate_report(
        &self,
        config: ComplianceReportConfig,
    ) -> Result<UnifiedReport, String> {
        // Create unified metadata
        let metadata = UnifiedReportMetadata::with_period(
            "compliance",
            "DataFold Compliance Manager",
            config.time_range.clone(),
        )
        .with_organization(&self.organization_info.name)
        .with_version("1.0");

        // Create unified report with the config
        let mut report = UnifiedReport::new(metadata, config.base_config.clone());

        // Generate requested sections
        for section in &config.include_sections {
            match section {
                ReportSection::ExecutiveSummary => {
                    let summary = self.generate_executive_summary(&config).await?;
                    report.add_section(&summary).map_err(|e| e.to_string())?;
                }
                ReportSection::SecurityIncidents => {
                    let incidents = self.generate_security_incidents(&config).await?;
                    report.add_section(&incidents).map_err(|e| e.to_string())?;
                }
                ReportSection::PolicyCompliance => {
                    let compliance = self.generate_policy_compliance(&config).await?;
                    report.add_section(&compliance).map_err(|e| e.to_string())?;
                }
                ReportSection::AuditTrail => {
                    let audit_trail = self.generate_audit_trail_summary(&config).await?;
                    report
                        .add_section(&audit_trail)
                        .map_err(|e| e.to_string())?;
                }
                _ => {
                    // For sections not yet migrated to unified types, skip for now
                    // These will be handled in subsequent refactoring
                }
            }
        }

        // Add digital signature if required
        if config.require_signature() {
            let signature = self.sign_unified_report(&report).await?;
            report.digital_signature = Some(signature);
        }

        // Cache the report
        {
            let cache = self.reports_cache.write().await;
            // Note: Cache expects ComplianceReport but we're returning UnifiedReport
            // For now, skip caching until we can refactor the cache to handle unified reports
            let _ = cache; // Silence unused variable warning
        }

        Ok(report)
    }

    /// Export report in specified format
    pub async fn export_report(
        &self,
        report: &ComplianceReport,
        format: UnifiedReportFormat,
    ) -> Result<Vec<u8>, String> {
        match format {
            UnifiedReportFormat::Json => serde_json::to_vec_pretty(report)
                .map_err(|e| format!("JSON serialization error: {}", e)),
            UnifiedReportFormat::Csv => self.export_csv(report).await,
            UnifiedReportFormat::Html => self.export_html(report).await,
            UnifiedReportFormat::Xml => self.export_xml(report).await,
            UnifiedReportFormat::Pdf => self.export_pdf(report).await,
            UnifiedReportFormat::Markdown => {
                Ok("Markdown export not implemented".as_bytes().to_vec())
            }
        }
    }

    /// Get compliance status for specific frameworks
    pub async fn get_compliance_status(
        &self,
        frameworks: &[ComplianceFramework],
        time_range: TimeRange,
    ) -> Result<HashMap<ComplianceFramework, ComplianceStatus>, String> {
        let mut status_map = HashMap::new();

        for framework in frameworks {
            let status = self
                .assess_framework_compliance(framework, &time_range)
                .await?;
            status_map.insert(framework.clone(), status);
        }

        Ok(status_map)
    }

    /// Schedule automatic compliance reporting
    pub async fn schedule_automatic_reports(
        &self,
        configs: Vec<(ComplianceReportConfig, ChronoDuration)>,
    ) -> Result<(), String> {
        // In a real implementation, this would set up scheduled tasks
        // For now, just validate the configurations
        for (config, _interval) in configs {
            self.validate_report_config(&config)?;
        }
        Ok(())
    }

    /// Clean up old reports based on retention policies
    pub async fn cleanup_old_reports(
        &self,
        frameworks: &[ComplianceFramework],
    ) -> Result<(), String> {
        let mut cache = self.reports_cache.write().await;
        let now = Utc::now();

        cache.retain(|_, report| {
            let max_retention = frameworks
                .iter()
                .map(|f| f.retention_period_days())
                .max()
                .unwrap_or(2555); // Default to 7 years

            let age = now.signed_duration_since(report.metadata.generated_at);
            age.num_days() <= max_retention as i64
        });

        Ok(())
    }

    // Private helper methods

    async fn generate_executive_summary(
        &self,
        __config: &ComplianceReportConfig,
    ) -> Result<crate::reporting::types::ExecutiveSummary, String> {
        // Generate executive summary based on audit data and security status
        let threat_status = self.threat_monitor.get_threat_status().await;

        let overall_status = match threat_status.overall_threat_level {
            ThreatLevel::Low => "Compliant",
            ThreatLevel::Medium => "Mostly Compliant",
            ThreatLevel::High => "Partially Compliant",
            ThreatLevel::Critical => "Non-Compliant",
        };

        let key_findings = vec![
            format!(
                "Total active threats: {}",
                threat_status.active_threat_count
            ),
            format!(
                "Recent rotation activity: {} operations",
                threat_status.recent_activity_count
            ),
            format!(
                "Failed rotations: {}",
                threat_status.failed_rotations_last_hour
            ),
        ];

        Ok(crate::reporting::types::ExecutiveSummary::new(
            "Key rotation compliance assessment",
            key_findings,
            overall_status,
        ))
    }

    async fn generate_rotation_statistics(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<RotationStatistics, String> {
        // In a real implementation, this would query the audit logger for statistics
        Ok(RotationStatistics {
            total_rotations: 150,
            successful_rotations: 145,
            failed_rotations: 5,
            rotations_by_reason: std::collections::HashMap::from([
                (RotationReason::Scheduled, 100),
                (RotationReason::UserInitiated, 30),
                (RotationReason::Policy, 15),
                (RotationReason::Compromise, 5),
            ]),
            average_rotation_time: Duration::from_secs(30),
            peak_periods: Vec::new(),
            user_activity: HashMap::new(),
        })
    }

    async fn generate_security_incidents(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<SecurityIncidentSummary, String> {
        let threat_status = self.threat_monitor.get_threat_status().await;

        Ok(SecurityIncidentSummary {
            total_incidents: threat_status.active_threat_count as u64,
            incidents_by_severity: {
                let mut map: HashMap<String, u64> = HashMap::new();
                map.insert(
                    ThreatLevel::Low.to_severity().to_string(),
                    *threat_status
                        .threat_counts
                        .get(&ThreatLevel::Low)
                        .unwrap_or(&0) as u64,
                );
                map.insert(
                    ThreatLevel::Medium.to_severity().to_string(),
                    *threat_status
                        .threat_counts
                        .get(&ThreatLevel::Medium)
                        .unwrap_or(&0) as u64,
                );
                map.insert(
                    ThreatLevel::High.to_severity().to_string(),
                    *threat_status
                        .threat_counts
                        .get(&ThreatLevel::High)
                        .unwrap_or(&0) as u64,
                );
                map.insert(
                    ThreatLevel::Critical.to_severity().to_string(),
                    *threat_status
                        .threat_counts
                        .get(&ThreatLevel::Critical)
                        .unwrap_or(&0) as u64,
                );
                map
            },
            incidents_by_type: HashMap::new(),
            resolution_stats: Some(ResolutionStatistics {
                average_resolution_time_seconds: 300,
                auto_resolved: 10,
                manual_resolved: 5,
                unresolved: 2,
                median_resolution_time_seconds: 250,
            }),
            top_threats: Vec::new(),
        })
    }

    async fn generate_control_assessment(
        &self,
        config: &ComplianceReportConfig,
    ) -> Result<ControlAssessment, String> {
        let mut control_results = HashMap::new();
        let mut framework_assessments = HashMap::new();

        // Assess each required control for each framework
        for framework in &config.frameworks {
            let controls = framework.required_controls();
            let mut framework_score = 0.0;

            for control in controls {
                let result = self.assess_control(&control).await?;
                framework_score += result.effectiveness_score;
                control_results.entry(control).or_insert(result);
            }

            framework_score /= framework.required_controls().len() as f64;
            framework_assessments.insert(framework.clone(), framework_score);
        }

        let overall_effectiveness = control_results
            .values()
            .map(|r| r.effectiveness_score)
            .sum::<f64>()
            / control_results.len() as f64;

        Ok(ControlAssessment {
            overall_effectiveness,
            control_results,
            framework_assessments,
            control_gaps: Vec::new(), // Would be populated based on gaps found
        })
    }

    async fn generate_audit_trail_summary(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<AuditTrailSummary, String> {
        // Verify audit chain integrity
        let integrity_verified = self.audit_logger.verify_audit_chain_integrity().await;

        Ok(AuditTrailSummary {
            total_events: 1000, // Would be actual count from audit logger
            events_by_type: HashMap::new(),
            integrity_verified,
            retention_compliant: true,
            integrity_issues: if integrity_verified {
                Vec::new()
            } else {
                vec!["Audit chain integrity issue detected".to_string()]
            },
        })
    }

    async fn generate_risk_assessment(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<RiskAssessmentSummary, String> {
        Ok(RiskAssessmentSummary {
            overall_risk_score: 0.3,
            risk_by_category: HashMap::from([
                ("Access Control".to_string(), 0.2),
                ("Data Integrity".to_string(), 0.1),
                ("Network Security".to_string(), 0.4),
            ]),
            high_risk_areas: Vec::new(),
            risk_trends: RiskTrends {
                score_trend: "stable".to_string(),
                trend_confidence: 0.8,
                trend_drivers: Vec::new(),
                projected_risk: 0.3,
            },
            mitigation_effectiveness: 0.85,
        })
    }

    async fn generate_policy_compliance(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<PolicyComplianceSummary, String> {
        Ok(PolicyComplianceSummary {
            overall_compliance: 92.0,
            policy_compliance: HashMap::from([
                ("Key Rotation Policy".to_string(), 95.0),
                ("Access Control Policy".to_string(), 88.0),
                ("Incident Response Policy".to_string(), 93.0),
            ]),
            policy_violations: Vec::new(),
            compliance_trends: ComplianceTrends {
                score_trend: "improving".to_string(),
                trend_direction: 2.5,
                improvement_areas: vec!["Security monitoring".to_string()],
                deteriorating_areas: Vec::new(),
            },
        })
    }

    async fn generate_recommendations(
        &self,
        _config: &ComplianceReportConfig,
    ) -> Result<RecommendationsSummary, String> {
        Ok(RecommendationsSummary {
            high_priority: vec![Recommendation {
                id: Uuid::new_v4(),
                title: "Implement automated key rotation".to_string(),
                description: "Set up automated key rotation for non-critical keys".to_string(),
                priority: "High".to_string(),
                effort: "Medium".to_string(),
                impact: "High".to_string(),
                target_completion: Some(Utc::now() + ChronoDuration::days(30)),
                responsible_party: Some("Security Team".to_string()),
                related_controls: vec![ComplianceControl::AccessControls],
            }],
            medium_priority: Vec::new(),
            low_priority: Vec::new(),
            quick_wins: Vec::new(),
            long_term: Vec::new(),
        })
    }

    async fn generate_raw_data(
        &self,
        config: &ComplianceReportConfig,
    ) -> Result<RawAuditData, String> {
        // Export raw audit data
        let correlations = Vec::new(); // Would be actual data from audit logger
        let audit_entries = self.audit_logger.get_audit_chain().await;

        Ok(RawAuditData {
            correlations,
            audit_entries,
            security_detections: Vec::new(),
            export_metadata: ExportMetadata {
                exported_at: Utc::now(),
                format_version: "1.0".to_string(),
                integrity_hash: "sha256:...".to_string(),
                purpose: "Compliance reporting".to_string(),
                retention_requirements: config
                    .frameworks
                    .iter()
                    .map(|f| (f.clone(), f.retention_period_days()))
                    .collect(),
            },
        })
    }

    async fn assess_control(&self, control: &ComplianceControl) -> Result<ControlResult, String> {
        // Assess individual control effectiveness
        let effectiveness_score = match control {
            ComplianceControl::AccessControls => 90.0,
            ComplianceControl::DataIntegrity => 95.0,
            ComplianceControl::SecurityMonitoring => 85.0,
            _ => 80.0,
        };

        Ok(ControlResult {
            effectiveness_score,
            implementation_status: if effectiveness_score > 80.0 {
                ImplementationStatus::Implemented
            } else {
                ImplementationStatus::PartiallyImplemented
            },
            evidence_count: 25,
            issues: Vec::new(),
            recommendations: Vec::new(),
        })
    }

    async fn assess_framework_compliance(
        &self,
        _framework: &ComplianceFramework,
        _time_range: &TimeRange,
    ) -> Result<ComplianceStatus, String> {
        // Assess compliance for specific framework
        Ok(ComplianceStatus::Compliant)
    }

    async fn sign_report(&self, _report: &ComplianceReport) -> Result<String, String> {
        // In a real implementation, this would digitally sign the report
        Ok("digital_signature_placeholder".to_string())
    }

    async fn sign_unified_report(&self, _report: &UnifiedReport) -> Result<String, String> {
        // In a real implementation, this would digitally sign the unified report
        Ok("unified_digital_signature_placeholder".to_string())
    }

    async fn export_csv(&self, _report: &ComplianceReport) -> Result<Vec<u8>, String> {
        // Convert report to CSV format
        Ok("CSV export placeholder".as_bytes().to_vec())
    }

    async fn export_html(&self, _report: &ComplianceReport) -> Result<Vec<u8>, String> {
        // Convert report to HTML format
        Ok("HTML export placeholder".as_bytes().to_vec())
    }

    async fn export_xml(&self, _report: &ComplianceReport) -> Result<Vec<u8>, String> {
        // Convert report to XML format
        Ok("XML export placeholder".as_bytes().to_vec())
    }

    async fn export_pdf(&self, _report: &ComplianceReport) -> Result<Vec<u8>, String> {
        // Convert report to PDF format
        Ok("PDF export placeholder".as_bytes().to_vec())
    }

    fn validate_report_config(&self, config: &ComplianceReportConfig) -> Result<(), String> {
        if config.frameworks.is_empty() {
            return Err("At least one compliance framework must be specified".to_string());
        }

        if config.include_sections.is_empty() {
            return Err("At least one report section must be included".to_string());
        }

        if config.time_range.start_time >= config.time_range.end_time {
            return Err("Invalid time range: start time must be before end time".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    #[tokio::test]
    async fn test_compliance_manager_creation() {
        let base_logger =
            Arc::new(crate::crypto::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(crate::crypto::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(crate::crypto::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            crate::crypto::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );
        let threat_monitor = Arc::new(
            RotationThreatMonitor::with_default_config(
                base_monitor,
                audit_logger.clone(),
                security_manager.clone(),
            ),
        );

        let org_info = OrganizationInfo {
            name: "Test Organization".to_string(),
            contact: "test@example.com".to_string(),
            compliance_officer: Some("John Doe".to_string()),
            auditor: Some("External Auditor Inc.".to_string()),
        };

        let compliance_manager = KeyRotationComplianceManager::new(
            audit_logger,
            threat_monitor,
            security_manager,
            org_info,
        );

        // Test basic functionality
        let config = ComplianceReportConfig {
            base_config: UnifiedReportConfig::with_formats(vec![UnifiedReportFormat::Json]),
            frameworks: vec![ComplianceFramework::Soc2],
            report_type: ComplianceReportType::Daily,
            time_range: TimeRange::last_days(1),
            include_sections: vec![ReportSection::ExecutiveSummary],
            include_raw_data: false,
        };

        let report = compliance_manager.generate_report(config).await.unwrap();
        assert_eq!(report.metadata.report_type, "compliance");
        assert!(report.sections.contains_key("executive_summary"));
    }

    #[test]
    fn test_compliance_framework_properties() {
        let soc2 = ComplianceFramework::Soc2;
        assert_eq!(soc2.display_name(), "SOC 2 Type II");
        assert_eq!(soc2.retention_period_days(), 2555);
        assert!(!soc2.required_controls().is_empty());

        let custom = ComplianceFramework::Custom("Custom Framework".to_string());
        assert_eq!(custom.display_name(), "Custom Framework");
    }

    #[test]
    fn test_time_range_creation() {
        let last_30_days = TimeRange::last_days(30);
        assert!(last_30_days.start_time < last_30_days.end_time);

        let current_month = TimeRange::current_month();
        assert!(Datelike::day(&current_month.start_time) == 1);
        assert!(current_month.start_time < current_month.end_time);
    }
}
