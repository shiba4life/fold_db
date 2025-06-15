# Migration Guide: Unified Reporting System

## Overview

This guide provides step-by-step instructions for migrating existing DataFold modules to use the unified reporting system. The migration process is designed to be incremental and backward-compatible, allowing for gradual adoption without disrupting existing functionality.

## Migration Benefits

Migrating to the unified reporting system provides:

- **Consistency**: Standardized report formats and metadata across modules
- **Reduced Maintenance**: Single API to maintain instead of module-specific implementations
- **Enhanced Features**: Built-in digital signatures, multiple output formats, and extensible configuration
- **Type Safety**: Compile-time verification of report structure correctness
- **Integration**: Easy creation of cross-module reports and dashboards

## Prerequisites

Before starting migration, ensure:

1. **Module Import**: Add unified reporting to your module imports
2. **Dependency Update**: Ensure your module can access `crate::reporting::types`
3. **Testing Framework**: Have unit tests in place for existing reporting functionality
4. **Backup**: Version control your existing reporting structures for reference

## Step-by-Step Migration Process

### Step 1: Import Unified Types

Replace or supplement existing reporting imports with unified types:

```rust
// Add these imports to your module
use crate::reporting::types::{
    UnifiedReport, UnifiedReportMetadata, UnifiedReportConfig,
    UnifiedReportFormat, UnifiedSummarySection, TimeRange
};
```

**Example from `key_rotation_compliance.rs`:**
```rust
use crate::reporting::types::{
    UnifiedReportFormat, UnifiedReportMetadata, UnifiedReportConfig, UnifiedReport,
    UnifiedSummarySection, TimeRange
};
```

### Step 2: Identify Legacy Structures

Catalog your existing reporting structures that need migration:

**Common Legacy Patterns:**
- Custom report configuration structs
- Module-specific metadata types  
- Individual summary/section structs
- Format enumerations
- Report generation logic

**Example Legacy Structure:**
```rust
// BEFORE: Module-specific structures
pub struct ComplianceReportConfig {
    pub output_format: ReportFormat,
    pub include_sections: Vec<String>,
    pub retention_period: Duration,
}

pub struct ComplianceReport {
    pub metadata: ReportMetadata,
    pub config: ComplianceReportConfig,
    pub executive_summary: ExecutiveSummary,
    pub audit_trail: AuditTrailSummary,
    pub recommendations: Vec<Recommendation>,
}
```

### Step 3: Migrate Configuration Structures

Replace module-specific configuration with unified configuration:

```rust
// AFTER: Using unified configuration
pub struct ComplianceAnalysisConfig {
    /// Base unified configuration for reports
    pub base_config: UnifiedReportConfig,
    /// Module-specific compliance settings
    pub retention_period: Duration,
    pub frameworks: Vec<ComplianceFramework>,
    pub audit_level: AuditLevel,
}

impl ComplianceAnalysisConfig {
    pub fn new() -> Self {
        Self {
            base_config: UnifiedReportConfig::with_formats(vec![
                UnifiedReportFormat::Json,
                UnifiedReportFormat::Pdf,
            ])
            .with_signature_requirement(),
            retention_period: Duration::from_secs(86400 * 365 * 7), // 7 years
            frameworks: vec![ComplianceFramework::Soc2],
            audit_level: AuditLevel::Detailed,
        }
    }
}
```

**Real Example from `performance/mod.rs`:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysisConfig {
    /// Base unified configuration for reports
    pub base_config: UnifiedReportConfig,
    pub regression_threshold_percent: f64,
    pub improvement_threshold_percent: f64,
    // ... other module-specific fields
}

impl PerformanceAnalysisConfig {
    pub fn new() -> Self {
        Self {
            base_config: UnifiedReportConfig::with_formats(vec![
                UnifiedReportFormat::Json,
                UnifiedReportFormat::Html
            ]),
            // ... initialize other fields
        }
    }
}
```

### Step 4: Migrate Section Structures

Convert existing summary/section structs to implement `UnifiedSummarySection`:

```rust
// BEFORE: Standalone section struct
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditSummary {
    pub total_controls: u64,
    pub passed_controls: u64,
    pub failed_controls: u64,
    pub compliance_percentage: f64,
}

// AFTER: Implementing UnifiedSummarySection
impl UnifiedSummarySection for ComplianceAuditSummary {
    fn section_name(&self) -> &'static str {
        "compliance_audit"
    }
}
```

**Section Name Guidelines:**
- Use snake_case format
- Choose descriptive, unique names
- Keep names consistent across similar modules
- Avoid special characters or spaces

**Common Section Names:**
- `"executive_summary"` - High-level overview
- `"security_incidents"` - Security event analysis
- `"audit_trail"` - Audit log summary
- `"performance"` - Performance metrics
- `"compliance"` - Compliance status
- `"recommendations"` - Suggested actions

### Step 5: Update Report Generation Logic

Replace custom report generation with unified report creation:

```rust
// BEFORE: Custom report generation
pub fn generate_compliance_report(
    config: &ComplianceReportConfig
) -> Result<ComplianceReport, Error> {
    let metadata = ReportMetadata {
        id: Uuid::new_v4(),
        generated_at: Utc::now(),
        // ... other fields
    };
    
    let report = ComplianceReport {
        metadata,
        config: config.clone(),
        executive_summary: generate_executive_summary()?,
        audit_trail: generate_audit_trail()?,
        recommendations: generate_recommendations()?,
    };
    
    Ok(report)
}

// AFTER: Using unified report generation
pub fn generate_compliance_report(
    config: &ComplianceAnalysisConfig
) -> Result<UnifiedReport, Box<dyn std::error::Error>> {
    // Create unified metadata
    let metadata = UnifiedReportMetadata::with_period(
        "compliance_audit",
        "compliance_system",
        TimeRange::current_month()
    )
    .with_organization("ACME Corp")
    .with_version("2.1");

    // Create the unified report
    let mut report = UnifiedReport::new(metadata, config.base_config.clone());

    // Add sections
    let executive_summary = generate_executive_summary()?;
    report.add_section(&executive_summary)?;

    let audit_summary = generate_audit_summary()?;
    report.add_section(&audit_summary)?;

    let compliance_summary = generate_compliance_summary()?;
    report.add_section(&compliance_summary)?;

    // Add digital signature if required
    if config.base_config.require_signature {
        let signature = generate_digital_signature(&report)?;
        report.set_digital_signature(signature);
    }

    // Validate the report
    report.validate_configuration()?;

    Ok(report)
}
```

### Step 6: Update Section Generation Functions

Modify section generation functions to use unified types:

```rust
// Section generation function
fn generate_executive_summary() -> Result<ExecutiveSummary, Error> {
    let summary = ExecutiveSummary::new(
        "Monthly compliance assessment for key rotation operations",
        vec![
            "All key rotations completed within policy timeframes".to_string(),
            "Zero security incidents related to key management".to_string(),
            "100% compliance with SOC2 requirements".to_string(),
        ],
        "Compliant"
    );
    
    Ok(summary)
}

fn generate_audit_summary() -> Result<AuditTrailSummary, Error> {
    let summary = AuditTrailSummary {
        total_events: 1247,
        unique_users: 23,
        compliance_status: "Fully Compliant".to_string(),
        notable_events: vec![
            "Emergency key rotation completed successfully".to_string(),
        ],
        access_patterns: std::collections::HashMap::new(),
    };
    
    Ok(summary)
}
```

### Step 7: Handle Module-Specific Extensions

For module-specific data that doesn't fit standard sections, create custom section types:

```rust
// Custom section for module-specific data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationMetrics {
    pub total_rotations: u64,
    pub scheduled_rotations: u64,
    pub emergency_rotations: u64,
    pub average_rotation_time_seconds: f64,
    pub rotation_success_rate: f64,
    pub next_scheduled_rotation: Option<DateTime<Utc>>,
}

impl UnifiedSummarySection for KeyRotationMetrics {
    fn section_name(&self) -> &'static str {
        "key_rotation_metrics"
    }
}

// Use in report generation
let rotation_metrics = KeyRotationMetrics {
    total_rotations: 45,
    scheduled_rotations: 43,
    emergency_rotations: 2,
    average_rotation_time_seconds: 127.5,
    rotation_success_rate: 100.0,
    next_scheduled_rotation: Some(Utc::now() + chrono::Duration::days(30)),
};

report.add_section(&rotation_metrics)?;
```

## Migration Patterns by Module Type

### Security Modules

**Common Migration Pattern:**
1. Replace security-specific report configs with `UnifiedReportConfig`
2. Migrate incident summaries to `SecurityIncidentSummary`
3. Convert threat analysis to `ThreatSummary` arrays
4. Add executive summary with security posture assessment

**Example:**
```rust
// Security module migration
use crate::reporting::types::{SecurityIncidentSummary, ThreatSummary, ExecutiveSummary};

let incidents = SecurityIncidentSummary::new()
    .with_severity_counts(severity_map)
    .with_type_counts(type_map);

let threats = vec![
    ThreatSummary {
        threat_type: "Brute Force Attack".to_string(),
        count: 3,
        severity: "High".to_string(),
        impact: "Authentication disruption".to_string(),
        mitigation_status: "Mitigated".to_string(),
    }
];

report.add_section(&incidents)?;
```

### Performance Modules  

**Common Migration Pattern:**
1. Integrate `UnifiedReportConfig` into performance config
2. Use `PerformanceSummary` for metrics aggregation
3. Include time ranges for performance periods
4. Add trend analysis in executive summary

**Example:**
```rust
// Performance module migration
use crate::reporting::types::PerformanceSummary;

let performance = PerformanceSummary {
    avg_response_time_ms: 42.3,
    max_response_time_ms: 156.8,
    throughput_metrics: throughput_map,
    error_rate_percent: 0.02,
    trends: vec!["Response time improving".to_string()],
};

report.add_section(&performance)?;
```

### Compliance Modules

**Common Migration Pattern:**
1. Use `UnifiedReportConfig` with signature requirements
2. Implement `ComplianceSummary` for framework adherence
3. Include `AuditTrailSummary` for audit analysis
4. Add retention period metadata

**Example:**
```rust
// Compliance module migration
use crate::reporting::types::{ComplianceSummary, AuditTrailSummary};

let compliance = ComplianceSummary {
    overall_compliance_percent: 98.5,
    compliance_by_framework: framework_status,
    policy_violations: 2,
    remediation_actions: vec!["Update access controls".to_string()],
    next_review_date: Some(Utc::now() + chrono::Duration::days(90)),
};

report.add_section(&compliance)?;
```

## Common Migration Challenges and Solutions

### Challenge 1: Complex Nested Data Structures

**Problem**: Existing reports have deeply nested or complex data structures that don't map cleanly to unified types.

**Solution**: Create custom section types that implement `UnifiedSummarySection`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexAnalysisSection {
    pub nested_data: HashMap<String, NestedStructure>,
    pub computed_metrics: Vec<ComputedMetric>,
    // Keep existing complex structure
}

impl UnifiedSummarySection for ComplexAnalysisSection {
    fn section_name(&self) -> &'static str {
        "complex_analysis"
    }
}
```

### Challenge 2: Multiple Report Types per Module

**Problem**: Module generates different types of reports (daily, weekly, monthly, incident-specific).

**Solution**: Use configuration and metadata to differentiate report types:

```rust
pub fn generate_daily_report() -> Result<UnifiedReport, Error> {
    let metadata = UnifiedReportMetadata::with_period(
        "daily_security_report",  // Different report type
        "security_system",
        TimeRange::last_days(1)
    );
    
    let config = UnifiedReportConfig::with_formats(vec![UnifiedReportFormat::Json])
        .with_sections(vec!["executive_summary".to_string()]);
    
    // Generate with daily-specific logic
}

pub fn generate_incident_report(incident_id: Uuid) -> Result<UnifiedReport, Error> {
    let metadata = UnifiedReportMetadata::new(
        "incident_report",        // Different report type
        "incident_system"
    );
    
    // Generate with incident-specific logic
}
```

### Challenge 3: Backward Compatibility

**Problem**: Need to maintain existing API while migrating to unified system.

**Solution**: Create adapter functions that convert between old and new formats:

```rust
// Maintain old API for backward compatibility
pub fn generate_legacy_compliance_report(
    config: &LegacyConfig
) -> Result<LegacyReport, Error> {
    // Generate unified report
    let unified_report = generate_unified_compliance_report(config)?;
    
    // Convert to legacy format
    let legacy_report = convert_to_legacy_format(unified_report)?;
    
    Ok(legacy_report)
}

fn convert_to_legacy_format(
    unified: UnifiedReport
) -> Result<LegacyReport, Error> {
    // Extract sections and convert to legacy structure
    let executive_summary: ExecutiveSummary = unified
        .get_section("executive_summary")
        .ok_or("Missing executive summary")?
        .map_err(|e| format!("Failed to deserialize: {}", e))?;
    
    // Build legacy report
    Ok(LegacyReport {
        // Map unified fields to legacy structure
    })
}
```

### Challenge 4: Digital Signature Integration

**Problem**: Adding digital signature support to existing workflows.

**Solution**: Integrate signature generation into report finalization:

```rust
use crate::crypto::digital_signature::SignatureGenerator;

pub fn finalize_report_with_signature(
    mut report: UnifiedReport,
    signature_generator: &SignatureGenerator
) -> Result<UnifiedReport, Error> {
    if report.config.require_signature {
        // Generate canonical representation for signing
        let canonical_data = serde_json::to_string(&report)?;
        
        // Generate digital signature
        let signature = signature_generator.sign(canonical_data.as_bytes())?;
        
        // Add signature to report
        report.set_digital_signature(signature);
    }
    
    Ok(report)
}
```

## Testing Migration

### Unit Tests

Ensure your migration maintains functionality:

```rust
#[cfg(test)]
mod migration_tests {
    use super::*;

    #[test]
    fn test_unified_report_generation() {
        let config = ComplianceAnalysisConfig::new();
        let report = generate_compliance_report(&config).unwrap();
        
        // Verify report structure
        assert!(!report.section_names().is_empty());
        assert!(report.section_names().contains(&"executive_summary".to_string()));
        
        // Verify metadata
        assert_eq!(report.metadata.report_type, "compliance_audit");
        
        // Verify configuration compliance
        assert!(report.validate_configuration().is_ok());
    }

    #[test]
    fn test_section_serialization() {
        let summary = ComplianceAuditSummary {
            total_controls: 25,
            passed_controls: 24,
            failed_controls: 1,
            compliance_percentage: 96.0,
        };
        
        // Test that section can be serialized and added to report
        let json = serde_json::to_string(&summary).unwrap();
        assert!(!json.is_empty());
        
        // Test section name
        assert_eq!(summary.section_name(), "compliance_audit");
    }
}
```

### Integration Tests

Test the full reporting pipeline:

```rust
#[tokio::test]
async fn test_end_to_end_report_generation() {
    let config = ComplianceAnalysisConfig::new();
    
    // Generate report
    let report = generate_compliance_report(&config).unwrap();
    
    // Test serialization
    let json_output = serde_json::to_string_pretty(&report).unwrap();
    assert!(!json_output.is_empty());
    
    // Test deserialization round-trip
    let deserialized: UnifiedReport = serde_json::from_str(&json_output).unwrap();
    assert_eq!(report.metadata.report_id, deserialized.metadata.report_id);
    assert_eq!(report.section_count(), deserialized.section_count());
}
```

## Validation Checklist

After migration, verify:

- [ ] **Imports Updated**: All unified reporting types imported correctly
- [ ] **Configuration Migrated**: Module config uses `UnifiedReportConfig` as base
- [ ] **Sections Implement Trait**: All section types implement `UnifiedSummarySection`
- [ ] **Report Generation**: Reports use `UnifiedReport` container
- [ ] **Section Names**: Unique, descriptive section names chosen
- [ ] **Serialization Works**: All sections serialize/deserialize correctly
- [ ] **Validation Passes**: Reports pass `validate_configuration()`
- [ ] **Tests Updated**: Unit tests cover unified reporting functionality
- [ ] **Documentation Updated**: Module documentation references unified system
- [ ] **Backward Compatibility**: Existing APIs still function (if required)

## Migration Timeline Recommendations

### Phase 1: Preparation (1-2 weeks)
- Review existing reporting structures
- Plan section mapping and naming
- Update module imports and dependencies
- Write migration tests

### Phase 2: Core Migration (2-3 weeks)  
- Migrate configuration structures
- Implement `UnifiedSummarySection` for existing sections
- Update report generation logic
- Integrate digital signature support

### Phase 3: Testing and Validation (1-2 weeks)
- Run comprehensive test suite
- Validate report generation functionality
- Test backward compatibility (if required)
- Performance testing

### Phase 4: Documentation and Cleanup (1 week)
- Update module documentation
- Remove legacy code (if not needed for compatibility)
- Update usage examples
- Final integration testing

## Support and Resources

- **Architecture Guide**: [unified-reporting-architecture.md](unified-reporting-architecture.md)
- **API Reference**: [api.md](api.md)
- **Best Practices**: [best-practices.md](best-practices.md)
- **Example Implementations**: See `src/datafold_node/key_rotation_compliance.rs`, `src/events/correlation.rs`, `src/tests/performance/mod.rs`

For additional support during migration, refer to the module-specific examples in the codebase that have already been successfully migrated to the unified reporting system.