# Best Practices for Unified Reporting

## Overview

This document provides recommended practices for using the DataFold Unified Reporting system effectively. Following these guidelines ensures consistent, maintainable, and secure reporting across all modules.

## Section Design Best Practices

### Section Naming Conventions

**Use snake_case format:**
```rust
// ✅ Good
impl UnifiedSummarySection for SecurityIncidentSummary {
    fn section_name(&self) -> &'static str {
        "security_incidents"
    }
}

// ❌ Avoid
fn section_name(&self) -> &'static str {
    "SecurityIncidents"  // PascalCase
    "security-incidents" // kebab-case
    "security incidents" // spaces
}
```

**Choose descriptive, unique names:**
```rust
// ✅ Good - Specific and clear
"executive_summary"
"security_incidents"
"audit_trail_analysis"
"performance_metrics"
"compliance_assessment"
"key_rotation_metrics"
"event_correlation_summary"

// ❌ Avoid - Too generic or ambiguous
"summary"
"data"
"results"
"info"
```

**Use consistent naming patterns:**

| Section Type | Naming Pattern | Examples |
|--------------|----------------|----------|
| **High-level overviews** | `{domain}_summary` | `executive_summary`, `security_summary` |
| **Incident/Event data** | `{type}_incidents` | `security_incidents`, `compliance_incidents` |
| **Metrics/Analytics** | `{domain}_metrics` | `performance_metrics`, `rotation_metrics` |
| **Audit/Trail data** | `{domain}_trail` | `audit_trail`, `access_trail` |
| **Assessment results** | `{domain}_assessment` | `compliance_assessment`, `risk_assessment` |

### Data Structure Guidelines

**Keep sections focused and cohesive:**
```rust
// ✅ Good - Focused on security incidents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentSummary {
    pub total_incidents: u64,
    pub incidents_by_severity: HashMap<String, u64>,
    pub incidents_by_type: HashMap<String, u64>,
    pub resolution_stats: Option<ResolutionStatistics>,
    pub top_threats: Vec<ThreatSummary>,
}

// ❌ Avoid - Mixing concerns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedSummary {
    pub incidents: u64,           // Security data
    pub response_time: f64,       // Performance data
    pub compliance_score: f64,    // Compliance data
    pub user_count: u64,          // Usage data
}
```

**Use appropriate data types:**
```rust
// ✅ Good - Semantic types
pub struct MetricsSummary {
    pub timestamp: DateTime<Utc>,           // Time data
    pub duration: Duration,                 // Time periods
    pub percentage: f64,                    // 0.0 to 100.0
    pub counts: HashMap<String, u64>,       // Categorical counts
    pub measurements: Vec<f64>,             // Numeric measurements
    pub status: String,                     // Enum-like values
    pub optional_data: Option<String>,      // Nullable fields
}

// ❌ Avoid - Generic or inappropriate types
pub struct BadMetrics {
    pub timestamp: String,                  // Should be DateTime
    pub duration_seconds: String,           // Should be numeric
    pub percentage: String,                 // Should be numeric
    pub everything: Value,                  // Too generic
}
```

**Include contextual metadata:**
```rust
// ✅ Good - Rich contextual information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    pub measurement_period: TimeRange,
    pub sample_size: usize,
    pub confidence_level: f64,
    pub baseline_comparison: Option<String>,
    pub metrics: PerformanceMetrics,
    pub trends: Vec<TrendIndicator>,
    pub recommendations: Vec<String>,
}
```

### Section Lifecycle Management

**Initialize sections with sensible defaults:**
```rust
impl SecurityIncidentSummary {
    pub fn new() -> Self {
        Self {
            total_incidents: 0,
            incidents_by_severity: HashMap::new(),
            incidents_by_type: HashMap::new(),
            resolution_stats: None,
            top_threats: Vec::new(),
        }
    }
    
    // Builder pattern for complex initialization
    pub fn with_incidents(mut self, incidents: Vec<Incident>) -> Self {
        self.total_incidents = incidents.len() as u64;
        self.populate_from_incidents(incidents);
        self
    }
}
```

**Implement validation for section data:**
```rust
impl SecurityIncidentSummary {
    pub fn validate(&self) -> Result<(), String> {
        // Verify data consistency
        let calculated_total: u64 = self.incidents_by_severity.values().sum();
        if calculated_total != self.total_incidents {
            return Err(format!(
                "Total incidents mismatch: calculated {}, reported {}",
                calculated_total, self.total_incidents
            ));
        }
        
        // Verify percentage ranges
        if let Some(ref stats) = self.resolution_stats {
            let rate = stats.resolution_rate_percent();
            if rate < 0.0 || rate > 100.0 {
                return Err(format!("Invalid resolution rate: {}%", rate));
            }
        }
        
        Ok(())
    }
}
```

## Report Configuration Best Practices

### Format Selection Guidelines

**Choose formats based on use case:**
```rust
// Executive/Business reports
let executive_config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Pdf,    // Formal presentation
    UnifiedReportFormat::Html,   // Web dashboard
]);

// Technical/Development reports  
let technical_config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Json,   // API integration
    UnifiedReportFormat::Markdown, // Documentation
]);

// Data analysis reports
let analysis_config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Csv,    // Spreadsheet analysis
    UnifiedReportFormat::Json,   // Data processing
]);

// Compliance/Audit reports
let compliance_config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Pdf,    // Official records
    UnifiedReportFormat::Xml,    // System integration
])
.with_signature_requirement();   // Integrity verification
```

### Section Filtering Best Practices

**Use selective section inclusion for performance:**
```rust
// ✅ Good - Include only needed sections
let focused_config = UnifiedReportConfig::new()
    .with_sections(vec![
        "executive_summary".to_string(),
        "key_metrics".to_string(),
    ]);

// ❌ Avoid - Including unnecessary sections
let bloated_config = UnifiedReportConfig::new()
    .with_sections(vec![
        "executive_summary".to_string(),
        "detailed_logs".to_string(),        // Large section
        "raw_data_dump".to_string(),        // Very large section
        "historical_analysis".to_string(),  // Computation-heavy
    ]);
```

**Implement section filtering logic:**
```rust
pub fn generate_filtered_report(
    full_config: &ModuleConfig,
    filter: &ReportFilter
) -> Result<UnifiedReport, Error> {
    let sections = match filter.report_type {
        ReportType::Executive => vec!["executive_summary"],
        ReportType::Technical => vec!["performance_metrics", "error_analysis"],
        ReportType::Compliance => vec!["compliance_assessment", "audit_trail"],
        ReportType::Full => vec![], // Empty means all sections
    };
    
    let config = UnifiedReportConfig::with_formats(filter.formats.clone())
        .with_sections(sections.into_iter().map(String::from).collect());
    
    generate_report(config)
}
```

## Security and Privacy Best Practices

### Digital Signature Implementation

**Always require signatures for compliance reports:**
```rust
pub fn create_compliance_config() -> UnifiedReportConfig {
    UnifiedReportConfig::with_formats(vec![
        UnifiedReportFormat::Pdf,
        UnifiedReportFormat::Json,
    ])
    .with_signature_requirement()  // ✅ Required for compliance
    .with_option("audit_level", "detailed")
}
```

**Implement signature verification:**
```rust
pub fn verify_report_integrity(
    report: &UnifiedReport,
    public_key: &PublicKey
) -> Result<bool, SignatureError> {
    let signature = report.digital_signature
        .as_ref()
        .ok_or(SignatureError::MissingSignature)?;
    
    // Create canonical representation
    let mut report_copy = report.clone();
    report_copy.digital_signature = None; // Remove signature for verification
    let canonical_data = serde_json::to_string(&report_copy)?;
    
    // Verify signature
    signature_verifier::verify(
        public_key,
        canonical_data.as_bytes(),
        signature
    )
}
```

### Data Anonymization Guidelines

**Implement field-level anonymization:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivitySummary {
    #[serde(skip_serializing_if = "should_anonymize")]
    pub user_id: Option<String>,
    
    #[serde(serialize_with = "maybe_anonymize_ip")]
    pub source_ip: String,
    
    pub activity_count: u64,
    pub last_activity: DateTime<Utc>,
}

fn should_anonymize(_: &Option<String>) -> bool {
    // Check global anonymization config
    get_global_config().anonymize_data
}

fn maybe_anonymize_ip<S>(ip: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    if get_global_config().anonymize_data {
        serializer.serialize_str("xxx.xxx.xxx.xxx")
    } else {
        serializer.serialize_str(ip)
    }
}
```

**Create anonymization helpers:**
```rust
pub struct AnonymizationUtils;

impl AnonymizationUtils {
    pub fn anonymize_email(email: &str) -> String {
        if let Some(at_pos) = email.find('@') {
            let local = &email[..at_pos];
            let domain = &email[at_pos..];
            format!("{}***{}", &local[..1], domain)
        } else {
            "***@***.***".to_string()
        }
    }
    
    pub fn anonymize_ip(ip: &str) -> String {
        if ip.contains(':') {
            // IPv6
            "xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx:xxxx".to_string()
        } else {
            // IPv4
            "xxx.xxx.xxx.xxx".to_string()
        }
    }
    
    pub fn hash_identifier(id: &str, salt: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(id.as_bytes());
        hasher.update(salt.as_bytes());
        format!("{:x}", hasher.finalize())[..16].to_string()
    }
}
```

## Performance Best Practices

### Efficient Report Generation

**Use streaming for large reports:**
```rust
pub async fn generate_large_report(
    config: &ReportConfig,
    output: &mut dyn AsyncWrite
) -> Result<(), Error> {
    let metadata = create_metadata();
    let unified_config = config.to_unified_config();
    
    // Stream report header
    let header = serde_json::to_string(&ReportHeader { metadata, config: unified_config })?;
    output.write_all(header.as_bytes()).await?;
    
    // Stream sections incrementally
    for section_name in &config.include_sections {
        let section_data = generate_section_streaming(section_name).await?;
        output.write_all(section_data.as_bytes()).await?;
    }
    
    Ok(())
}
```

**Cache expensive computations:**
```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct CachedReportGenerator {
    cache: Arc<RwLock<HashMap<String, CachedSection>>>,
    cache_ttl: Duration,
}

impl CachedReportGenerator {
    pub async fn get_or_generate_section<T>(
        &self,
        section_name: &str,
        generator: impl Fn() -> Result<T, Error>
    ) -> Result<T, Error>
    where
        T: Clone + Serialize + for<'de> Deserialize<'de>,
    {
        let cache_key = format!("{}_{}", section_name, Utc::now().format("%Y%m%d%H"));
        
        // Try cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(&cache_key) {
                if cached.expires_at > Utc::now() {
                    return Ok(serde_json::from_value(cached.data.clone())?);
                }
            }
        }
        
        // Generate fresh data
        let fresh_data = generator()?;
        let cached_section = CachedSection {
            data: serde_json::to_value(&fresh_data)?,
            expires_at: Utc::now() + chrono::Duration::from_std(self.cache_ttl)?,
        };
        
        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, cached_section);
        }
        
        Ok(fresh_data)
    }
}
```

### Memory Management

**Minimize memory usage for large datasets:**
```rust
// ✅ Good - Iterator-based processing
pub fn calculate_statistics_efficient(
    events: impl Iterator<Item = SecurityEvent>
) -> StatisticsSummary {
    let mut total_count = 0;
    let mut severity_counts = HashMap::new();
    let mut sum_response_time = 0.0;
    
    for event in events {
        total_count += 1;
        *severity_counts.entry(event.severity.clone()).or_insert(0) += 1;
        sum_response_time += event.response_time_ms;
    }
    
    StatisticsSummary {
        total_events: total_count,
        severity_distribution: severity_counts,
        avg_response_time: sum_response_time / total_count as f64,
    }
}

// ❌ Avoid - Loading all data into memory
pub fn calculate_statistics_inefficient(
    events: Vec<SecurityEvent>  // Loads everything into memory
) -> StatisticsSummary {
    // Process the full vector...
}
```

## Error Handling Best Practices

### Comprehensive Error Handling

**Define domain-specific error types:**
```rust
#[derive(Debug, thiserror::Error)]
pub enum ReportGenerationError {
    #[error("Section serialization failed: {section_name}")]
    SectionSerializationFailed {
        section_name: String,
        #[source]
        source: serde_json::Error,
    },
    
    #[error("Required section missing: {section_name}")]
    RequiredSectionMissing { section_name: String },
    
    #[error("Configuration validation failed: {reason}")]
    ConfigurationInvalid { reason: String },
    
    #[error("Digital signature generation failed")]
    SignatureFailed(#[from] SignatureError),
    
    #[error("Data source unavailable: {source}")]
    DataSourceUnavailable { source: String },
}
```

**Implement graceful degradation:**
```rust
pub fn generate_report_with_fallbacks(
    config: &ReportConfig
) -> Result<UnifiedReport, ReportGenerationError> {
    let metadata = UnifiedReportMetadata::new("system_report", "report_generator");
    let mut report = UnifiedReport::new(metadata, config.base_config.clone());
    
    // Always try to include executive summary
    match generate_executive_summary() {
        Ok(summary) => { report.add_section(&summary)?; },
        Err(e) => {
            log::warn!("Failed to generate executive summary: {}", e);
            // Add minimal fallback summary
            let fallback = ExecutiveSummary::new(
                "Report generation encountered issues",
                vec!["Some sections may be incomplete".to_string()],
                "Partial"
            );
            report.add_section(&fallback)?;
        }
    }
    
    // Try optional sections with individual error handling
    for section_type in &config.optional_sections {
        if let Err(e) = try_generate_section(&mut report, section_type) {
            log::error!("Failed to generate section {}: {}", section_type, e);
            // Continue with other sections
        }
    }
    
    Ok(report)
}
```

## Testing Best Practices

### Unit Testing Guidelines

**Test section implementations thoroughly:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_incident_summary_creation() {
        let summary = SecurityIncidentSummary::new()
            .with_severity_counts(create_test_severity_map());
        
        assert_eq!(summary.section_name(), "security_incidents");
        assert_eq!(summary.total_incidents, 10);
        assert!(summary.validate().is_ok());
    }
    
    #[test]
    fn test_section_serialization_roundtrip() {
        let original = create_test_security_summary();
        
        // Serialize
        let json = serde_json::to_string(&original).unwrap();
        
        // Deserialize
        let deserialized: SecurityIncidentSummary = 
            serde_json::from_str(&json).unwrap();
        
        assert_eq!(original.total_incidents, deserialized.total_incidents);
        assert_eq!(original.section_name(), deserialized.section_name());
    }
    
    #[test]
    fn test_report_validation() {
        let config = UnifiedReportConfig::new()
            .with_signature_requirement()
            .with_sections(vec!["executive_summary".to_string()]);
        
        let metadata = UnifiedReportMetadata::new("test", "system");
        let mut report = UnifiedReport::new(metadata, config);
        
        // Should fail without required section and signature
        assert!(report.validate_configuration().is_err());
        
        // Add required section
        let summary = ExecutiveSummary::new("Test", vec![], "OK");
        report.add_section(&summary).unwrap();
        
        // Should still fail without signature
        assert!(report.validate_configuration().is_err());
        
        // Add signature
        report.set_digital_signature("test_signature");
        
        // Should now pass
        assert!(report.validate_configuration().is_ok());
    }
}
```

### Integration Testing

**Test complete report generation workflows:**
```rust
#[tokio::test]
async fn test_end_to_end_report_generation() {
    let config = create_test_config();
    
    // Generate report
    let report = generate_compliance_report(&config).await.unwrap();
    
    // Verify structure
    assert!(!report.section_names().is_empty());
    assert!(report.is_signed());
    assert!(report.validate_configuration().is_ok());
    
    // Test serialization to all configured formats
    for format in &config.base_config.formats {
        match format {
            UnifiedReportFormat::Json => {
                let json = serde_json::to_string_pretty(&report).unwrap();
                assert!(!json.is_empty());
            },
            UnifiedReportFormat::Csv => {
                let csv = convert_to_csv(&report).unwrap();
                assert!(!csv.is_empty());
            },
            // Test other formats...
            _ => {}
        }
    }
}
```

## Documentation Best Practices

### Code Documentation

**Document section structures clearly:**
```rust
/// Security incident summary section for unified reports
///
/// This section provides aggregated security incident data including
/// counts by severity and type, resolution statistics, and threat analysis.
///
/// # Fields
///
/// * `total_incidents` - Total number of incidents in the reporting period
/// * `incidents_by_severity` - Breakdown of incidents by severity level
/// * `incidents_by_type` - Breakdown of incidents by incident type  
/// * `resolution_stats` - Optional resolution timing and method statistics
/// * `top_threats` - List of most significant threats identified
///
/// # Example
///
/// ```rust
/// use datafold::reporting::SecurityIncidentSummary;
/// use std::collections::HashMap;
///
/// let mut severity_counts = HashMap::new();
/// severity_counts.insert("Critical".to_string(), 2);
/// severity_counts.insert("High".to_string(), 8);
///
/// let summary = SecurityIncidentSummary::new()
///     .with_severity_counts(severity_counts);
///
/// assert_eq!(summary.total_incidents, 10);
/// assert_eq!(summary.section_name(), "security_incidents");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentSummary {
    // ... fields
}
```

### Module Documentation

**Include usage examples in module docs:**
```rust
//! # Compliance Reporting Module
//!
//! This module provides comprehensive compliance reporting capabilities
//! using the unified reporting system.
//!
//! ## Quick Start
//!
//! ```rust
//! use crate::compliance::{ComplianceAnalysisConfig, generate_compliance_report};
//! use crate::reporting::{UnifiedReportFormat, TimeRange};
//!
//! let config = ComplianceAnalysisConfig::new()
//!     .with_formats(vec![UnifiedReportFormat::Json, UnifiedReportFormat::Pdf])
//!     .with_period(TimeRange::last_days(30));
//!
//! let report = generate_compliance_report(&config)?;
//! println!("Generated report with {} sections", report.section_count());
//! ```
//!
//! ## Section Types
//!
//! This module provides the following report sections:
//!
//! - [`ComplianceAssessment`] - Overall compliance status and scores
//! - [`PolicyViolations`] - Policy violation analysis and trends
//! - [`RemediationTracking`] - Remediation action status and timelines
```

## Deployment and Operations

### Configuration Management

**Use environment-based configuration:**
```rust
#[derive(Debug, Clone)]
pub struct ReportingEnvironmentConfig {
    pub default_formats: Vec<UnifiedReportFormat>,
    pub require_signatures: bool,
    pub enable_anonymization: bool,
    pub cache_ttl_seconds: u64,
    pub max_report_size_mb: usize,
}

impl ReportingEnvironmentConfig {
    pub fn from_environment() -> Self {
        Self {
            default_formats: parse_formats_env("REPORTING_DEFAULT_FORMATS")
                .unwrap_or_else(|| vec![UnifiedReportFormat::Json]),
            require_signatures: std::env::var("REPORTING_REQUIRE_SIGNATURES")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            enable_anonymization: std::env::var("REPORTING_ANONYMIZE_DATA")
                .map(|v| v.parse().unwrap_or(false))
                .unwrap_or(false),
            cache_ttl_seconds: std::env::var("REPORTING_CACHE_TTL")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600),
            max_report_size_mb: std::env::var("REPORTING_MAX_SIZE_MB")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(100),
        }
    }
}
```

### Monitoring and Observability

**Add metrics to report generation:**
```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref REPORT_GENERATION_TOTAL: Counter = register_counter!(
        "datafold_reports_generated_total",
        "Total number of reports generated"
    ).unwrap();
    
    static ref REPORT_GENERATION_DURATION: Histogram = register_histogram!(
        "datafold_report_generation_duration_seconds",
        "Time spent generating reports"
    ).unwrap();
}

pub async fn generate_report_with_metrics(
    config: &ReportConfig
) -> Result<UnifiedReport, Error> {
    let _timer = REPORT_GENERATION_DURATION.start_timer();
    
    let result = generate_report(config).await;
    
    if result.is_ok() {
        REPORT_GENERATION_TOTAL.inc();
    }
    
    result
}
```

---

Following these best practices ensures that your unified reporting implementation is secure, performant, maintainable, and consistent with the overall DataFold architecture.