# Unified Reporting Structures & Interfaces: Design Proposal

## Rationale for Unification

The audit revealed significant duplication and inconsistency in reporting and summary structs across security, compliance, performance, and validation modules. Unifying these structures will:
- Reduce code duplication and maintenance burden
- Ensure consistent reporting formats and field names
- Simplify onboarding and code review
- Enable easier extension and integration across modules

## Groupings of Similar Structs/Enums

### 1. **Report Metadata & Config**
- `ReportMetadata`, `ComplianceReportConfig`, `ReportConfig`, `PerformanceAnalysisConfig`
- Common fields: report ID, type, time range/period, generated_at, generated_by, output format(s)

### 2. **Report Format Enums**
- `ReportFormat` (multiple definitions)
- Common values: PDF, JSON, CSV, HTML, XML, Markdown

### 3. **Summary/Section Structs**
- `ExecutiveSummary`, `SecurityIncidentSummary`, `AuditTrailSummary`, `RiskAssessmentSummary`, `PolicyComplianceSummary`, `CorrelationSummary`, `SecurityEffectivenessSummary`, `PerformanceMeasurement`, etc.
- Common fields: summary/description, statistics, counts, trends, recommendations

### 4. **Recommendations & Trends**
- `RecommendationsSummary`, `Recommendation`, `ComplianceTrends`, `RiskTrends`

### 5. **Module-Specific Extensions**
- Some modules (e.g., cross-platform validation, performance) have unique fields but follow similar patterns

## Proposed Unified Struct/Interface Definitions (Rust)

```rust
// Unified report format enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum UnifiedReportFormat {
    Pdf,
    Json,
    Csv,
    Html,
    Xml,
    Markdown,
}

// Unified report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedReportMetadata {
    pub report_id: Uuid,
    pub report_type: String, // e.g., "compliance", "performance", etc.
    pub generated_at: DateTime<Utc>,
    pub generated_by: String,
    pub period: Option<TimeRange>,
    pub version: Option<String>,
    pub organization: Option<String>,
}

// Unified report config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedReportConfig {
    pub formats: Vec<UnifiedReportFormat>,
    pub include_sections: Vec<String>, // e.g., ["summary", "incidents", ...]
    pub anonymize_data: bool,
    pub require_signature: bool,
    pub additional_options: HashMap<String, String>,
}

// Unified summary trait for all summary/section structs
pub trait UnifiedSummarySection: Serialize + std::fmt::Debug {
    fn section_name(&self) -> &'static str;
}

// Example: Executive summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveSummary {
    pub description: String,
    pub key_findings: Vec<String>,
    pub overall_status: String,
}

impl UnifiedSummarySection for ExecutiveSummary {
    fn section_name(&self) -> &'static str { "executive_summary" }
}

// Example: Security incident summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentSummary {
    pub total_incidents: u64,
    pub incidents_by_severity: HashMap<String, u64>,
    pub incidents_by_type: HashMap<String, u64>,
    pub resolution_stats: Option<ResolutionStatistics>,
    pub top_threats: Vec<ThreatSummary>,
}

impl UnifiedSummarySection for SecurityIncidentSummary {
    fn section_name(&self) -> &'static str { "security_incidents" }
}

// ... (other summary/section structs follow similar pattern)

// Unified report struct (generic over sections)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedReport {
    pub metadata: UnifiedReportMetadata,
    pub config: UnifiedReportConfig,
    pub sections: HashMap<String, serde_json::Value>, // section_name -> serialized section
    pub digital_signature: Option<String>,
}
```

## Mapping/Migration Notes by Module

- **Compliance/Key Rotation:**
  - Map `ComplianceReport`, `ComplianceReportConfig`, `ReportMetadata`, etc. to `UnifiedReport`, `UnifiedReportConfig`, `UnifiedReportMetadata`.
  - Section structs (e.g., `ExecutiveSummary`, `AuditTrailSummary`) implement `UnifiedSummarySection`.
- **Performance/Testing:**
  - Map `ReportConfig`, `PerformanceAnalysisConfig`, `PerformanceMeasurement` to unified config/section pattern.
  - Use `UnifiedReportFormat` for output formats.
- **Cross-Platform Validation:**
  - Map `CrossPlatformReport`, `SecurityEffectivenessSummary`, etc. to unified report/section pattern.
- **Event Correlation:**
  - `CorrelationSummary` becomes a section in `UnifiedReport`.
- **Protocol Validation:**
  - `ValidationSummary` and report generation logic use unified types.

## Open Questions & Recommendations

1. Should all section structs be required to implement a trait (for dynamic section handling)? YES
2. Should we use `serde_json::Value` for section storage, or generic/trait objects? YES
3. How should we handle module-specific fields that don't fit the unified pattern? IGNORE
4. Should we standardize field naming (snake_case vs. camelCase) for serialization? snake_case
5. Where should the unified module live? (e.g., `src/reporting/types.rs`) OK

---

**Review and feedback are requested before implementation.** 