//! # Unified Reporting Module
//!
//! This module provides unified reporting structures and interfaces for all security modules,
//! compliance, performance monitoring, and validation systems. It serves as the foundation
//! for consistent reporting across the entire DataFold platform.
//!
//! ## Core Components
//!
//! - [`UnifiedReportFormat`] - Standardized output format enumeration
//! - [`UnifiedReportMetadata`] - Common metadata structure for all reports
//! - [`UnifiedReportConfig`] - Configuration management for report generation
//! - [`UnifiedReport`] - Main report container with digital signature support
//! - [`UnifiedSummarySection`] - Base trait for all summary/section implementations
//!
//! ## Design Principles
//!
//! - **Consistency**: All reports use the same metadata structure and format options
//! - **Extensibility**: New section types can be added by implementing the trait
//! - **Type Safety**: Rust's type system ensures correct usage patterns
//! - **Flexibility**: Dynamic section storage allows for module-specific extensions
//! - **Security**: Digital signature support for report integrity verification
//!
//! ## Usage Example
//!
//! ```rust
//! use datafold::reporting::types::{
//!     UnifiedReport, UnifiedReportMetadata, UnifiedReportConfig,
//!     UnifiedReportFormat, ExecutiveSummary, UnifiedSummarySection
//! };
//! use uuid::Uuid;
//! use chrono::Utc;
//! use std::collections::HashMap;
//!
//! // Create report metadata
//! let metadata = UnifiedReportMetadata {
//!     report_id: Uuid::new_v4(),
//!     report_type: "security_compliance".to_string(),
//!     generated_at: Utc::now(),
//!     generated_by: "datafold_system".to_string(),
//!     period: None,
//!     version: Some("1.0".to_string()),
//!     organization: Some("ACME Corp".to_string()),
//! };
//!
//! // Create report configuration
//! let config = UnifiedReportConfig {
//!     formats: vec![UnifiedReportFormat::Json, UnifiedReportFormat::Pdf],
//!     include_sections: vec!["executive_summary".to_string()],
//!     anonymize_data: false,
//!     require_signature: true,
//!     additional_options: HashMap::new(),
//! };
//!
//! // Create a section
//! let summary = ExecutiveSummary {
//!     description: "Monthly security assessment".to_string(),
//!     key_findings: vec!["No critical vulnerabilities found".to_string()],
//!     overall_status: "Secure".to_string(),
//! };
//!
//! // Add section to report
//! let mut sections = HashMap::new();
//! sections.insert(
//!     summary.section_name().to_string(),
//!     serde_json::to_value(&summary).expect("Serialization failed")
//! );
//!
//! // Create the unified report
//! let report = UnifiedReport {
//!     metadata,
//!     config,
//!     sections,
//!     digital_signature: None, // Would be populated by signing process
//! };
//! ```

use chrono::{DateTime, Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Supported output formats for unified reports
///
/// This enum defines all supported report output formats across the platform.
/// Each format serves different use cases and consumption patterns.
///
/// # Format Descriptions
///
/// - **PDF**: Human-readable reports for executive consumption and archival
/// - **JSON**: Machine-readable format for API integration and data processing
/// - **CSV**: Tabular data format for spreadsheet analysis and data import
/// - **HTML**: Web-friendly format for dashboard integration and browser display
/// - **XML**: Structured format for enterprise system integration
/// - **Markdown**: Documentation-friendly format for technical reports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum UnifiedReportFormat {
    /// Portable Document Format for formal reports
    Pdf,
    /// JavaScript Object Notation for API integration
    Json,
    /// Comma-Separated Values for tabular data
    Csv,
    /// HyperText Markup Language for web display
    Html,
    /// eXtensible Markup Language for enterprise integration
    Xml,
    /// Markdown format for documentation
    Markdown,
}

impl UnifiedReportFormat {
    /// Returns the file extension for this format
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Html => "html",
            Self::Xml => "xml",
            Self::Markdown => "md",
        }
    }

    /// Returns the MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Json => "application/json",
            Self::Csv => "text/csv",
            Self::Html => "text/html",
            Self::Xml => "application/xml",
            Self::Markdown => "text/markdown",
        }
    }

    /// Returns true if this format supports binary data
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Pdf)
    }
}

impl std::fmt::Display for UnifiedReportFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pdf => write!(f, "PDF"),
            Self::Json => write!(f, "JSON"),
            Self::Csv => write!(f, "CSV"),
            Self::Html => write!(f, "HTML"),
            Self::Xml => write!(f, "XML"),
            Self::Markdown => write!(f, "Markdown"),
        }
    }
}

/// Time range specification for report data inclusion
///
/// Defines a time window for data collection and analysis in reports.
/// Used to specify the temporal scope of the report contents.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub struct TimeRange {
    /// Start of the time range (inclusive)
    pub start_time: DateTime<Utc>,
    /// End of the time range (inclusive)
    pub end_time: DateTime<Utc>,
}

impl TimeRange {
    /// Creates a new time range
    pub fn new(start_time: DateTime<Utc>, end_time: DateTime<Utc>) -> Self {
        Self {
            start_time,
            end_time,
        }
    }

    /// Returns the duration of this time range in seconds
    pub fn duration_seconds(&self) -> i64 {
        (self.end_time - self.start_time).num_seconds()
    }

    /// Returns true if the given timestamp falls within this range
    pub fn contains(&self, timestamp: DateTime<Utc>) -> bool {
        timestamp >= self.start_time && timestamp <= self.end_time
    }

    /// Creates a time range for the last N days
    pub fn last_days(days: i64) -> Self {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(days);
        Self::new(start_time, end_time)
    }

    /// Creates a time range for the last N hours
    pub fn last_hours(hours: i64) -> Self {
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::hours(hours);
        Self::new(start_time, end_time)
    }

    /// Creates a time range for the current month
    pub fn current_month() -> Self {
        let now = Utc::now();
        let start_time = now
            .with_day(1)
            .unwrap()
            .with_hour(0)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();
        let end_time = now;
        Self::new(start_time, end_time)
    }
}

/// Standardized metadata for all unified reports
///
/// Contains common identification, timing, and organizational information
/// that every report must include for consistency and traceability.
///
/// # Fields
///
/// - `report_id`: Unique identifier for this specific report instance
/// - `report_type`: Categorical type (e.g., "compliance", "security", "performance")
/// - `generated_at`: Timestamp when the report was generated
/// - `generated_by`: System or user that generated the report
/// - `period`: Optional time range covered by the report data
/// - `version`: Optional version identifier for the report format/schema
/// - `organization`: Optional organization identifier for multi-tenant scenarios
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UnifiedReportMetadata {
    /// Unique identifier for this report
    pub report_id: Uuid,
    /// Type classification of the report (e.g., "compliance", "security", "performance")
    pub report_type: String,
    /// UTC timestamp when the report was generated
    pub generated_at: DateTime<Utc>,
    /// Identifier of the system or user that generated the report
    pub generated_by: String,
    /// Optional time range that the report covers
    pub period: Option<TimeRange>,
    /// Optional version identifier for the report schema
    pub version: Option<String>,
    /// Optional organization identifier for multi-tenant systems
    pub organization: Option<String>,
}

impl UnifiedReportMetadata {
    /// Creates new report metadata with minimal required fields
    pub fn new(report_type: impl Into<String>, generated_by: impl Into<String>) -> Self {
        Self {
            report_id: Uuid::new_v4(),
            report_type: report_type.into(),
            generated_at: Utc::now(),
            generated_by: generated_by.into(),
            period: None,
            version: None,
            organization: None,
        }
    }

    /// Creates new report metadata with a specific time period
    pub fn with_period(
        report_type: impl Into<String>,
        generated_by: impl Into<String>,
        period: TimeRange,
    ) -> Self {
        Self {
            report_id: Uuid::new_v4(),
            report_type: report_type.into(),
            generated_at: Utc::now(),
            generated_by: generated_by.into(),
            period: Some(period),
            version: None,
            organization: None,
        }
    }

    /// Sets the organization identifier
    pub fn with_organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    /// Sets the version identifier
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

/// Configuration settings for unified report generation
///
/// Specifies how reports should be generated, formatted, and processed.
/// Controls output formats, content inclusion, privacy settings, and security requirements.
///
/// # Fields
///
/// - `formats`: List of output formats to generate
/// - `include_sections`: Specific sections to include (empty means all)
/// - `anonymize_data`: Whether to anonymize sensitive data
/// - `require_signature`: Whether digital signature is required
/// - `additional_options`: Module-specific configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UnifiedReportConfig {
    /// Output formats to generate for this report
    pub formats: Vec<UnifiedReportFormat>,
    /// Specific sections to include (empty vector means include all available)
    pub include_sections: Vec<String>,
    /// Whether to anonymize personally identifiable information
    pub anonymize_data: bool,
    /// Whether digital signature is required for this report
    pub require_signature: bool,
    /// Additional module-specific configuration options
    pub additional_options: HashMap<String, String>,
}

impl UnifiedReportConfig {
    /// Creates a default configuration for JSON output
    pub fn new() -> Self {
        Self {
            formats: vec![UnifiedReportFormat::Json],
            include_sections: Vec::new(),
            anonymize_data: false,
            require_signature: false,
            additional_options: HashMap::new(),
        }
    }

    /// Creates a configuration for multiple formats
    pub fn with_formats(formats: Vec<UnifiedReportFormat>) -> Self {
        Self {
            formats,
            include_sections: Vec::new(),
            anonymize_data: false,
            require_signature: false,
            additional_options: HashMap::new(),
        }
    }

    /// Sets specific sections to include
    pub fn with_sections(mut self, sections: Vec<String>) -> Self {
        self.include_sections = sections;
        self
    }

    /// Enables data anonymization
    pub fn with_anonymization(mut self) -> Self {
        self.anonymize_data = true;
        self
    }

    /// Requires digital signature
    pub fn with_signature_requirement(mut self) -> Self {
        self.require_signature = true;
        self
    }

    /// Adds an additional configuration option
    pub fn with_option(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.additional_options.insert(key.into(), value.into());
        self
    }

    /// Returns true if a specific section should be included
    pub fn should_include_section(&self, section_name: &str) -> bool {
        self.include_sections.is_empty()
            || self.include_sections.contains(&section_name.to_string())
    }
}

impl Default for UnifiedReportConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Base trait for all summary and section implementations
///
/// This trait must be implemented by all structs that represent report sections.
/// It provides a standardized interface for section identification and enables
/// dynamic section handling through trait objects.
///
/// # Implementation Requirements
///
/// - Must be serializable with Serde
/// - Must provide a unique section name
/// - Should implement Debug for troubleshooting
///
/// # Example Implementation
///
/// ```rust
/// use datafold::reporting::types::UnifiedSummarySection;
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Debug, Clone, Serialize, Deserialize)]
/// pub struct CustomSection {
///     pub data: String,
/// }
///
/// impl UnifiedSummarySection for CustomSection {
///     fn section_name(&self) -> &'static str {
///         "custom_section"
///     }
/// }
/// ```
pub trait UnifiedSummarySection: Serialize + std::fmt::Debug + Send + Sync {
    /// Returns the unique name identifier for this section type
    ///
    /// This name is used as the key in the report's sections HashMap
    /// and should be consistent across all instances of the same section type.
    fn section_name(&self) -> &'static str;
}

/// Executive summary section for high-level report overviews
///
/// Provides a concise overview of the report contents, key findings,
/// and overall status assessment. Typically included in all report types
/// for executive consumption.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExecutiveSummary {
    /// High-level description of the report contents and scope
    pub description: String,
    /// List of the most important findings or observations
    pub key_findings: Vec<String>,
    /// Overall status assessment (e.g., "Secure", "At Risk", "Compliant")
    pub overall_status: String,
}

impl UnifiedSummarySection for ExecutiveSummary {
    fn section_name(&self) -> &'static str {
        "executive_summary"
    }
}

impl ExecutiveSummary {
    /// Creates a new executive summary
    pub fn new(
        description: impl Into<String>,
        key_findings: Vec<String>,
        overall_status: impl Into<String>,
    ) -> Self {
        Self {
            description: description.into(),
            key_findings,
            overall_status: overall_status.into(),
        }
    }
}

/// Security incident summary section
///
/// Aggregates security incident data including counts by severity and type,
/// resolution statistics, and top threat information.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SecurityIncidentSummary {
    /// Total number of security incidents in the reporting period
    pub total_incidents: u64,
    /// Incident counts grouped by severity level
    pub incidents_by_severity: HashMap<String, u64>,
    /// Incident counts grouped by incident type
    pub incidents_by_type: HashMap<String, u64>,
    /// Statistics about incident resolution times and methods
    pub resolution_stats: Option<ResolutionStatistics>,
    /// Top threats identified during the reporting period
    pub top_threats: Vec<ThreatSummary>,
}

impl UnifiedSummarySection for SecurityIncidentSummary {
    fn section_name(&self) -> &'static str {
        "security_incidents"
    }
}

impl SecurityIncidentSummary {
    /// Creates a new security incident summary
    pub fn new() -> Self {
        Self {
            total_incidents: 0,
            incidents_by_severity: HashMap::new(),
            incidents_by_type: HashMap::new(),
            resolution_stats: None,
            top_threats: Vec::new(),
        }
    }

    /// Adds incident counts by severity
    pub fn with_severity_counts(mut self, counts: HashMap<String, u64>) -> Self {
        self.incidents_by_severity = counts;
        self.total_incidents = self.incidents_by_severity.values().sum();
        self
    }

    /// Adds incident counts by type
    pub fn with_type_counts(mut self, counts: HashMap<String, u64>) -> Self {
        self.incidents_by_type = counts;
        self
    }
}

impl Default for SecurityIncidentSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Audit trail summary section
///
/// Summarizes audit log analysis including access patterns,
/// compliance status, and notable audit events.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct AuditTrailSummary {
    /// Total number of audit events recorded
    pub total_events: u64,
    /// Number of unique users who performed audited actions
    pub unique_users: u64,
    /// Compliance status based on audit analysis
    pub compliance_status: String,
    /// Notable or suspicious audit events
    pub notable_events: Vec<String>,
    /// Access pattern analysis results
    pub access_patterns: HashMap<String, u64>,
}

impl UnifiedSummarySection for AuditTrailSummary {
    fn section_name(&self) -> &'static str {
        "audit_trail"
    }
}

/// Performance measurement summary section
///
/// Contains performance metrics, benchmarks, and trend analysis
/// for system components and operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PerformanceSummary {
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Maximum response time observed
    pub max_response_time_ms: f64,
    /// System throughput metrics
    pub throughput_metrics: HashMap<String, f64>,
    /// Error rate percentage
    pub error_rate_percent: f64,
    /// Performance trend indicators
    pub trends: Vec<String>,
}

impl UnifiedSummarySection for PerformanceSummary {
    fn section_name(&self) -> &'static str {
        "performance"
    }
}

/// Compliance assessment summary section
///
/// Summarizes compliance status against various regulations,
/// frameworks, and internal policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ComplianceSummary {
    /// Overall compliance percentage
    pub overall_compliance_percent: f64,
    /// Compliance status by framework or regulation
    pub compliance_by_framework: HashMap<String, String>,
    /// Number of policy violations found
    pub policy_violations: u64,
    /// Required remediation actions
    pub remediation_actions: Vec<String>,
    /// Next compliance review date
    pub next_review_date: Option<DateTime<Utc>>,
}

impl UnifiedSummarySection for ComplianceSummary {
    fn section_name(&self) -> &'static str {
        "compliance"
    }
}

/// Resolution statistics for incidents or issues
///
/// Provides detailed metrics about how incidents are resolved
/// including timing and resolution methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResolutionStatistics {
    /// Average time to resolve incidents (in seconds)
    pub average_resolution_time_seconds: i64,
    /// Number of incidents resolved automatically
    pub auto_resolved: u64,
    /// Number of incidents requiring manual intervention
    pub manual_resolved: u64,
    /// Number of unresolved incidents
    pub unresolved: u64,
    /// Median resolution time (in seconds)
    pub median_resolution_time_seconds: i64,
}

impl ResolutionStatistics {
    /// Calculates the resolution rate as a percentage
    pub fn resolution_rate_percent(&self) -> f64 {
        let total = self.auto_resolved + self.manual_resolved + self.unresolved;
        if total == 0 {
            return 100.0;
        }
        ((self.auto_resolved + self.manual_resolved) as f64 / total as f64) * 100.0
    }

    /// Returns the total number of resolved incidents
    pub fn total_resolved(&self) -> u64 {
        self.auto_resolved + self.manual_resolved
    }
}

/// Threat summary information
///
/// Describes a specific type of security threat including
/// frequency, severity, and mitigation status.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ThreatSummary {
    /// Type or category of the threat
    pub threat_type: String,
    /// Number of occurrences of this threat
    pub count: u64,
    /// Severity level of the threat
    pub severity: String,
    /// Description of the potential impact
    pub impact: String,
    /// Current status of mitigation efforts
    pub mitigation_status: String,
}

/// The main unified report container
///
/// Combines metadata, configuration, and dynamic section content into a single
/// report structure. Supports digital signatures for integrity verification.
///
/// # Section Storage
///
/// Sections are stored as a HashMap where:
/// - Key: Section name (from `UnifiedSummarySection::section_name()`)
/// - Value: Serialized section data as `serde_json::Value`
///
/// This approach allows for:
/// - Type-safe section creation through traits
/// - Flexible storage of different section types
/// - Easy serialization/deserialization of the entire report
/// - Module-specific extensions without core changes
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UnifiedReport {
    /// Report identification and timing metadata
    pub metadata: UnifiedReportMetadata,
    /// Configuration used to generate this report
    pub config: UnifiedReportConfig,
    /// Dynamic section storage (section_name -> serialized section)
    pub sections: HashMap<String, Value>,
    /// Optional digital signature for report integrity verification
    pub digital_signature: Option<String>,
}

impl UnifiedReport {
    /// Creates a new unified report with metadata and configuration
    pub fn new(metadata: UnifiedReportMetadata, config: UnifiedReportConfig) -> Self {
        Self {
            metadata,
            config,
            sections: HashMap::new(),
            digital_signature: None,
        }
    }

    /// Adds a section to the report
    ///
    /// # Arguments
    ///
    /// * `section` - Any struct implementing `UnifiedSummarySection`
    ///
    /// # Returns
    ///
    /// Result indicating success or serialization error
    ///
    /// # Example
    ///
    /// ```rust
    /// use datafold::reporting::types::{UnifiedReport, UnifiedReportMetadata, UnifiedReportConfig, ExecutiveSummary};
    ///
    /// let metadata = UnifiedReportMetadata::new("test_report", "test_system");
    /// let config = UnifiedReportConfig::new();
    /// let mut report = UnifiedReport::new(metadata, config);
    /// let summary = ExecutiveSummary::new("Report overview", vec![], "Good");
    /// report.add_section(&summary).unwrap();
    /// ```
    pub fn add_section<T: UnifiedSummarySection>(
        &mut self,
        section: &T,
    ) -> Result<(), serde_json::Error> {
        let section_name = section.section_name().to_string();
        let serialized_section = serde_json::to_value(section)?;
        self.sections.insert(section_name, serialized_section);
        Ok(())
    }

    /// Retrieves a section from the report and deserializes it
    ///
    /// # Type Parameters
    ///
    /// * `T` - The section type to deserialize to
    ///
    /// # Arguments
    ///
    /// * `section_name` - Name of the section to retrieve
    ///
    /// # Returns
    ///
    /// Option containing the deserialized section, or None if not found
    pub fn get_section<T: for<'de> Deserialize<'de>>(
        &self,
        section_name: &str,
    ) -> Option<Result<T, serde_json::Error>> {
        self.sections
            .get(section_name)
            .map(|value| serde_json::from_value(value.clone()))
    }

    /// Returns the list of section names in this report
    pub fn section_names(&self) -> Vec<String> {
        self.sections.keys().cloned().collect()
    }

    /// Returns the number of sections in this report
    pub fn section_count(&self) -> usize {
        self.sections.len()
    }

    /// Sets the digital signature for this report
    pub fn set_digital_signature(&mut self, signature: impl Into<String>) {
        self.digital_signature = Some(signature.into());
    }

    /// Returns true if this report has a digital signature
    pub fn is_signed(&self) -> bool {
        self.digital_signature.is_some()
    }

    /// Validates that the report configuration is satisfied
    ///
    /// Checks if all required sections specified in the configuration
    /// are present in the report.
    pub fn validate_configuration(&self) -> Result<(), String> {
        // Check if signature is required but missing
        if self.config.require_signature && !self.is_signed() {
            return Err("Digital signature is required but not present".to_string());
        }

        // Check if all required sections are present
        for required_section in &self.config.include_sections {
            if !self.sections.contains_key(required_section) {
                return Err(format!(
                    "Required section '{}' is missing",
                    required_section
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_report_format_properties() {
        assert_eq!(UnifiedReportFormat::Json.file_extension(), "json");
        assert_eq!(UnifiedReportFormat::Pdf.mime_type(), "application/pdf");
        assert!(UnifiedReportFormat::Pdf.is_binary());
        assert!(!UnifiedReportFormat::Json.is_binary());
    }

    #[test]
    fn test_time_range_functionality() {
        let now = Utc::now();
        let past = now - chrono::Duration::hours(1);
        let range = TimeRange::new(past, now);

        assert_eq!(range.duration_seconds(), 3600);
        assert!(range.contains(now - chrono::Duration::minutes(30)));
        assert!(!range.contains(now + chrono::Duration::minutes(30)));
    }

    #[test]
    fn test_unified_report_metadata_creation() {
        let metadata = UnifiedReportMetadata::new("security", "test_system")
            .with_organization("ACME Corp")
            .with_version("1.0");

        assert_eq!(metadata.report_type, "security");
        assert_eq!(metadata.generated_by, "test_system");
        assert_eq!(metadata.organization, Some("ACME Corp".to_string()));
        assert_eq!(metadata.version, Some("1.0".to_string()));
    }

    #[test]
    fn test_unified_report_config_functionality() {
        let config = UnifiedReportConfig::with_formats(vec![UnifiedReportFormat::Json])
            .with_sections(vec!["executive_summary".to_string()])
            .with_anonymization()
            .with_signature_requirement();

        assert!(config.anonymize_data);
        assert!(config.require_signature);
        assert!(config.should_include_section("executive_summary"));
        assert!(!config.should_include_section("other_section"));
    }

    #[test]
    fn test_section_implementation() {
        let summary = ExecutiveSummary::new("Test report", vec!["Finding 1".to_string()], "Good");

        assert_eq!(summary.section_name(), "executive_summary");
        assert_eq!(summary.description, "Test report");
    }

    #[test]
    fn test_unified_report_section_management() {
        let metadata = UnifiedReportMetadata::new("test", "system");
        let config = UnifiedReportConfig::new();
        let mut report = UnifiedReport::new(metadata, config);

        let summary = ExecutiveSummary::new("Test", vec![], "OK");

        // Test adding section
        assert!(report.add_section(&summary).is_ok());
        assert_eq!(report.section_count(), 1);
        assert!(report
            .section_names()
            .contains(&"executive_summary".to_string()));

        // Test retrieving section
        let retrieved: Option<Result<ExecutiveSummary, _>> =
            report.get_section("executive_summary");
        assert!(retrieved.is_some());
        assert!(retrieved.unwrap().is_ok());
    }

    #[test]
    fn test_report_validation() {
        let metadata = UnifiedReportMetadata::new("test", "system");
        let config = UnifiedReportConfig::new()
            .with_signature_requirement()
            .with_sections(vec!["executive_summary".to_string()]);
        let mut report = UnifiedReport::new(metadata, config);

        // Should fail validation - missing signature and section
        assert!(report.validate_configuration().is_err());

        // Add signature
        report.set_digital_signature("test_signature");
        assert!(report.is_signed());

        // Should still fail - missing required section
        assert!(report.validate_configuration().is_err());

        // Add required section
        let summary = ExecutiveSummary::new("Test", vec![], "OK");
        report.add_section(&summary).unwrap();

        // Should now pass validation
        assert!(report.validate_configuration().is_ok());
    }

    #[test]
    fn test_resolution_statistics() {
        let stats = ResolutionStatistics {
            average_resolution_time_seconds: 3600,
            auto_resolved: 80,
            manual_resolved: 15,
            unresolved: 5,
            median_resolution_time_seconds: 3000,
        };

        assert_eq!(stats.total_resolved(), 95);
        assert_eq!(stats.resolution_rate_percent(), 95.0);
    }

    #[test]
    fn test_serialization() {
        let metadata = UnifiedReportMetadata::new("test", "system");
        let config = UnifiedReportConfig::new();
        let mut report = UnifiedReport::new(metadata, config);

        let summary = ExecutiveSummary::new("Test report", vec![], "Good");
        report.add_section(&summary).unwrap();

        // Test that the report can be serialized and deserialized
        let json = serde_json::to_string(&report).expect("Failed to serialize report");
        let deserialized: UnifiedReport =
            serde_json::from_str(&json).expect("Failed to deserialize report");

        assert_eq!(
            report.metadata.report_type,
            deserialized.metadata.report_type
        );
        assert_eq!(report.section_count(), deserialized.section_count());
    }
}
