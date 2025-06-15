# Unified Reporting API Reference

## Overview

This document provides comprehensive API documentation for the DataFold Unified Reporting system. The API is designed around core types and traits that provide a consistent interface for report generation across all platform modules.

## Core Types

### UnifiedReportFormat

Enum defining supported output formats for reports.

```rust
pub enum UnifiedReportFormat {
    Pdf,        // Portable Document Format
    Json,       // JavaScript Object Notation  
    Csv,        // Comma-Separated Values
    Html,       // HyperText Markup Language
    Xml,        // eXtensible Markup Language
    Markdown,   // Markdown format
}
```

#### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`file_extension()`](../../src/reporting/types.rs:115) | `&'static str` | Returns file extension for the format |
| [`mime_type()`](../../src/reporting/types.rs:127) | `&'static str` | Returns MIME type for HTTP responses |
| [`is_binary()`](../../src/reporting/types.rs:139) | `bool` | Returns true for binary formats (PDF) |

#### Format Details

| Format | Extension | MIME Type | Use Case |
|--------|-----------|-----------|----------|
| **PDF** | `.pdf` | `application/pdf` | Executive reports, archival documents |
| **JSON** | `.json` | `application/json` | API integration, data processing |
| **CSV** | `.csv` | `text/csv` | Spreadsheet analysis, data import |
| **HTML** | `.html` | `text/html` | Web dashboards, browser display |
| **XML** | `.xml` | `application/xml` | Enterprise system integration |
| **Markdown** | `.md` | `text/markdown` | Documentation, technical reports |

#### Example

```rust
use datafold::reporting::UnifiedReportFormat;

let format = UnifiedReportFormat::Json;
println!("Extension: {}", format.file_extension()); // "json"
println!("MIME: {}", format.mime_type());           // "application/json"
println!("Binary: {}", format.is_binary());         // false
```

### TimeRange

Struct defining temporal scope for report data.

```rust
pub struct TimeRange {
    pub start_time: DateTime<Utc>,  // Inclusive start
    pub end_time: DateTime<Utc>,    // Inclusive end
}
```

#### Constructors

| Constructor | Description |
|-------------|-------------|
| [`new(start, end)`](../../src/reporting/types.rs:172) | Creates time range with specific dates |
| [`last_days(days)`](../../src/reporting/types.rs:187) | Creates range for last N days |
| [`last_hours(hours)`](../../src/reporting/types.rs:194) | Creates range for last N hours |
| [`current_month()`](../../src/reporting/types.rs:201) | Creates range for current month |

#### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`duration_seconds()`](../../src/reporting/types.rs:177) | `i64` | Duration in seconds |
| [`contains(timestamp)`](../../src/reporting/types.rs:182) | `bool` | Tests if timestamp is in range |

#### Example

```rust
use datafold::reporting::TimeRange;
use chrono::Utc;

// Last 7 days
let range = TimeRange::last_days(7);

// Specific period
let start = Utc::now() - chrono::Duration::hours(24);
let end = Utc::now();
let custom_range = TimeRange::new(start, end);

// Check if timestamp is included
let timestamp = Utc::now() - chrono::Duration::hours(12);
if range.contains(timestamp) {
    println!("Timestamp is within range");
}
```

### UnifiedReportMetadata

Standardized metadata for all reports.

```rust
pub struct UnifiedReportMetadata {
    pub report_id: Uuid,                    // Unique identifier
    pub report_type: String,                // Type classification
    pub generated_at: DateTime<Utc>,        // Generation timestamp
    pub generated_by: String,               // Generator identification
    pub period: Option<TimeRange>,          // Data time range
    pub version: Option<String>,            // Schema version
    pub organization: Option<String>,       // Organization context
}
```

#### Constructors

| Constructor | Description |
|-------------|-------------|
| [`new(type, generator)`](../../src/reporting/types.rs:244) | Creates metadata with required fields |
| [`with_period(type, generator, period)`](../../src/reporting/types.rs:260) | Creates metadata with time period |

#### Builder Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`with_organization(org)`](../../src/reporting/types.rs:277) | `Self` | Sets organization identifier |
| [`with_version(version)`](../../src/reporting/types.rs:283) | `Self` | Sets version identifier |

#### Example

```rust
use datafold::reporting::{UnifiedReportMetadata, TimeRange};

// Basic metadata
let metadata = UnifiedReportMetadata::new("security_audit", "datafold_system");

// With additional context
let metadata = UnifiedReportMetadata::with_period(
    "compliance_report",
    "audit_system", 
    TimeRange::last_days(30)
)
.with_organization("ACME Corp")
.with_version("2.1");
```

### UnifiedReportConfig

Configuration for report generation and processing.

```rust
pub struct UnifiedReportConfig {
    pub formats: Vec<UnifiedReportFormat>,     // Output formats
    pub include_sections: Vec<String>,         // Section filter
    pub anonymize_data: bool,                  // Privacy flag
    pub require_signature: bool,               // Security requirement
    pub additional_options: HashMap<String, String>, // Extension options
}
```

#### Constructors

| Constructor | Description |
|-------------|-------------|
| [`new()`](../../src/reporting/types.rs:318) | Creates default config (JSON format) |
| [`with_formats(formats)`](../../src/reporting/types.rs:329) | Creates config with specific formats |

#### Builder Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`with_sections(sections)`](../../src/reporting/types.rs:340) | `Self` | Sets specific sections to include |
| [`with_anonymization()`](../../src/reporting/types.rs:346) | `Self` | Enables data anonymization |
| [`with_signature_requirement()`](../../src/reporting/types.rs:352) | `Self` | Requires digital signature |
| [`with_option(key, value)`](../../src/reporting/types.rs:358) | `Self` | Adds module-specific option |

#### Utility Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`should_include_section(name)`](../../src/reporting/types.rs:364) | `bool` | Tests if section should be included |

#### Example

```rust
use datafold::reporting::{UnifiedReportConfig, UnifiedReportFormat};

// Multi-format configuration
let config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Json,
    UnifiedReportFormat::Pdf,
    UnifiedReportFormat::Html,
])
.with_sections(vec![
    "executive_summary".to_string(),
    "security_incidents".to_string(),
])
.with_anonymization()
.with_signature_requirement()
.with_option("include_charts", "true");

// Check section inclusion
if config.should_include_section("executive_summary") {
    // Include this section
}
```

## Traits

### UnifiedSummarySection

Base trait for all report sections.

```rust
pub trait UnifiedSummarySection: Serialize + std::fmt::Debug + Send + Sync {
    fn section_name(&self) -> &'static str;
}
```

#### Requirements

1. **Serializable**: Must implement `Serialize` for JSON storage
2. **Debug**: Must implement `Debug` for troubleshooting
3. **Thread Safe**: Must be `Send + Sync` for concurrent processing
4. **Section Name**: Must provide unique section identifier

#### Implementation Example

```rust
use datafold::reporting::UnifiedSummarySection;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSummary {
    pub metric_value: f64,
    pub status: String,
}

impl UnifiedSummarySection for CustomSummary {
    fn section_name(&self) -> &'static str {
        "custom_summary"  // Unique identifier
    }
}
```

## Built-in Section Types

### ExecutiveSummary

High-level report overview section.

```rust
pub struct ExecutiveSummary {
    pub description: String,        // Report description
    pub key_findings: Vec<String>,  // Important findings
    pub overall_status: String,     // Status assessment
}
```

#### Constructor

```rust
impl ExecutiveSummary {
    pub fn new(
        description: impl Into<String>,
        key_findings: Vec<String>,
        overall_status: impl Into<String>,
    ) -> Self
}
```

#### Example

```rust
use datafold::reporting::ExecutiveSummary;

let summary = ExecutiveSummary::new(
    "Q4 2024 Security Assessment",
    vec![
        "No critical vulnerabilities found".to_string(),
        "All compliance requirements met".to_string(),
        "Performance within acceptable limits".to_string(),
    ],
    "Secure"
);
```

### SecurityIncidentSummary

Security incident aggregation and analysis.

```rust
pub struct SecurityIncidentSummary {
    pub total_incidents: u64,
    pub incidents_by_severity: HashMap<String, u64>,
    pub incidents_by_type: HashMap<String, u64>,
    pub resolution_stats: Option<ResolutionStatistics>,
    pub top_threats: Vec<ThreatSummary>,
}
```

#### Constructor

```rust
impl SecurityIncidentSummary {
    pub fn new() -> Self
    pub fn with_severity_counts(self, counts: HashMap<String, u64>) -> Self
    pub fn with_type_counts(self, counts: HashMap<String, u64>) -> Self
}
```

#### Example

```rust
use datafold::reporting::SecurityIncidentSummary;
use std::collections::HashMap;

let mut severity_counts = HashMap::new();
severity_counts.insert("Critical".to_string(), 2);
severity_counts.insert("High".to_string(), 5);
severity_counts.insert("Medium".to_string(), 12);

let summary = SecurityIncidentSummary::new()
    .with_severity_counts(severity_counts);
```

### AuditTrailSummary

Audit log analysis and compliance tracking.

```rust
pub struct AuditTrailSummary {
    pub total_events: u64,                    // Total audit events
    pub unique_users: u64,                    // Unique user count
    pub compliance_status: String,            // Compliance assessment
    pub notable_events: Vec<String>,          // Important events
    pub access_patterns: HashMap<String, u64>, // Pattern analysis
}
```

### PerformanceSummary

System performance metrics and analysis.

```rust
pub struct PerformanceSummary {
    pub avg_response_time_ms: f64,             // Average response time
    pub max_response_time_ms: f64,             // Maximum response time
    pub throughput_metrics: HashMap<String, f64>, // Throughput data
    pub error_rate_percent: f64,               // Error rate
    pub trends: Vec<String>,                   // Performance trends
}
```

### ComplianceSummary

Compliance status and policy adherence.

```rust
pub struct ComplianceSummary {
    pub overall_compliance_percent: f64,         // Overall compliance
    pub compliance_by_framework: HashMap<String, String>, // Framework status
    pub policy_violations: u64,                  // Violation count
    pub remediation_actions: Vec<String>,        // Required actions
    pub next_review_date: Option<DateTime<Utc>>, // Next review
}
```

## Helper Types

### ResolutionStatistics

Incident resolution metrics and analysis.

```rust
pub struct ResolutionStatistics {
    pub average_resolution_time_seconds: i64,  // Average resolution time
    pub auto_resolved: u64,                    // Auto-resolved count
    pub manual_resolved: u64,                  // Manual resolution count
    pub unresolved: u64,                       // Unresolved count
    pub median_resolution_time_seconds: i64,   // Median resolution time
}
```

#### Methods

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`resolution_rate_percent()`](../../src/reporting/types.rs:602) | `f64` | Calculate resolution rate percentage |
| [`total_resolved()`](../../src/reporting/types.rs:611) | `u64` | Total resolved incidents |

### ThreatSummary

Individual threat analysis and mitigation status.

```rust
pub struct ThreatSummary {
    pub threat_type: String,         // Threat category
    pub count: u64,                  // Occurrence count
    pub severity: String,            // Severity level
    pub impact: String,              // Impact description
    pub mitigation_status: String,   // Mitigation status
}
```

## Main Report Container

### UnifiedReport

The primary report container combining metadata, configuration, and sections.

```rust
pub struct UnifiedReport {
    pub metadata: UnifiedReportMetadata,        // Report metadata
    pub config: UnifiedReportConfig,            // Generation config
    pub sections: HashMap<String, Value>,       // Dynamic sections
    pub digital_signature: Option<String>,      // Optional signature
}
```

#### Constructors

| Constructor | Description |
|-------------|-------------|
| [`new(metadata, config)`](../../src/reporting/types.rs:666) | Creates empty report with metadata and config |

#### Section Management

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`add_section<T>(section)`](../../src/reporting/types.rs:692) | `Result<(), serde_json::Error>` | Adds typed section to report |
| [`get_section<T>(name)`](../../src/reporting/types.rs:712) | `Option<Result<T, serde_json::Error>>` | Retrieves and deserializes section |
| [`section_names()`](../../src/reporting/types.rs:717) | `Vec<String>` | Lists all section names |
| [`section_count()`](../../src/reporting/types.rs:722) | `usize` | Returns number of sections |

#### Digital Signature

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`set_digital_signature(sig)`](../../src/reporting/types.rs:727) | `()` | Sets digital signature |
| [`is_signed()`](../../src/reporting/types.rs:732) | `bool` | Checks if report is signed |

#### Validation

| Method | Return Type | Description |
|--------|-------------|-------------|
| [`validate_configuration()`](../../src/reporting/types.rs:740) | `Result<(), String>` | Validates config requirements |

#### Complete Example

```rust
use datafold::reporting::{
    UnifiedReport, UnifiedReportMetadata, UnifiedReportConfig,
    UnifiedReportFormat, ExecutiveSummary, SecurityIncidentSummary,
    TimeRange
};
use std::collections::HashMap;

// Create metadata
let metadata = UnifiedReportMetadata::with_period(
    "monthly_security_report",
    "security_system",
    TimeRange::last_days(30)
)
.with_organization("ACME Corp")
.with_version("1.0");

// Create configuration
let config = UnifiedReportConfig::with_formats(vec![
    UnifiedReportFormat::Json,
    UnifiedReportFormat::Pdf,
])
.with_sections(vec![
    "executive_summary".to_string(),
    "security_incidents".to_string(),
])
.with_signature_requirement();

// Create report
let mut report = UnifiedReport::new(metadata, config);

// Add executive summary
let executive_summary = ExecutiveSummary::new(
    "Monthly security assessment for November 2024",
    vec![
        "Zero critical vulnerabilities detected".to_string(),
        "All security policies compliant".to_string(),
        "Incident response time improved by 15%".to_string(),
    ],
    "Secure"
);
report.add_section(&executive_summary)?;

// Add security incidents summary
let mut severity_counts = HashMap::new();
severity_counts.insert("Critical".to_string(), 0);
severity_counts.insert("High".to_string(), 2);
severity_counts.insert("Medium".to_string(), 8);
severity_counts.insert("Low".to_string(), 15);

let incidents_summary = SecurityIncidentSummary::new()
    .with_severity_counts(severity_counts);
report.add_section(&incidents_summary)?;

// Set digital signature (would be generated by cryptographic system)
report.set_digital_signature("crypto_signature_hash_here");

// Validate the report
report.validate_configuration()?;

// Report is now ready for generation in configured formats
println!("Report generated with {} sections", report.section_count());
println!("Sections: {:?}", report.section_names());
```

## Error Handling

The API uses standard Rust error handling patterns:

- **Serialization Errors**: Methods that serialize/deserialize sections return `Result<T, serde_json::Error>`
- **Validation Errors**: Configuration validation returns `Result<(), String>` with descriptive error messages
- **Section Retrieval**: Returns `Option<Result<T, serde_json::Error>>` for safe optional access

## Thread Safety

All types in the unified reporting API are thread-safe:

- All structs implement `Send + Sync` where appropriate
- The `UnifiedSummarySection` trait requires `Send + Sync`
- Reports can be safely shared across threads and processed concurrently

## Serialization

All types are fully serializable using Serde:

- **JSON**: Primary format for API integration and storage
- **Snake Case**: Consistent field naming convention
- **Optional Fields**: Proper handling of optional data
- **Round Trip**: Full serialization/deserialization support

---

**See Also:**
- [Architecture Overview](unified-reporting-architecture.md) - System design and principles
- [Migration Guide](migration-guide.md) - Migrating existing reports
- [Best Practices](best-practices.md) - Usage recommendations