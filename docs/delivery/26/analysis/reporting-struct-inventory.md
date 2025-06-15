# Reporting and Summary Struct Inventory

This document catalogs all reporting and summary structs and enums found across security and related modules as part of PBI 26-1.

| File/Module | Struct/Enum Name | Type | Brief Description |
|-------------|------------------|------|-------------------|
| src/datafold_node/key_rotation_compliance.rs | ComplianceReportType | enum | Types of compliance reports (daily, weekly, incident, etc.) |
| src/datafold_node/key_rotation_compliance.rs | ComplianceReportConfig | struct | Configuration for generating compliance reports |
| src/datafold_node/key_rotation_compliance.rs | TimeRange | struct | Time range for reports (start/end) |
| src/datafold_node/key_rotation_compliance.rs | ReportSection | enum | Sections to include in a report |
| src/datafold_node/key_rotation_compliance.rs | ReportFormat | enum | Output formats for reports (PDF, JSON, etc.) |
| src/datafold_node/key_rotation_compliance.rs | ComplianceReport | struct | Main compliance report data structure |
| src/datafold_node/key_rotation_compliance.rs | ReportMetadata | struct | Metadata for reports (ID, type, period, etc.) |
| src/datafold_node/key_rotation_compliance.rs | SecurityIncidentSummary | struct | Summary of security incidents for a report |
| src/datafold_node/key_rotation_compliance.rs | ResolutionStatistics | struct | Statistics on incident resolution |
| src/datafold_node/key_rotation_compliance.rs | ThreatSummary | struct | Summary of a specific threat |
| src/datafold_node/key_rotation_compliance.rs | ControlAssessment | struct | Results of compliance control assessment |
| src/datafold_node/key_rotation_compliance.rs | AuditTrailSummary | struct | Summary of audit trail data |
| src/datafold_node/key_rotation_compliance.rs | RiskAssessmentSummary | struct | Summary of risk assessment results |
| src/datafold_node/key_rotation_compliance.rs | RiskArea | struct | Details of a specific risk area |
| src/datafold_node/key_rotation_compliance.rs | RiskTrends | struct | Analysis of risk trends over time |
| src/datafold_node/key_rotation_compliance.rs | PolicyComplianceSummary | struct | Summary of policy compliance |
| src/datafold_node/key_rotation_compliance.rs | PolicyViolation | struct | Details of a policy violation |
| src/datafold_node/key_rotation_compliance.rs | ComplianceTrends | struct | Analysis of compliance trends |
| src/datafold_node/key_rotation_compliance.rs | RecommendationsSummary | struct | Summary of recommendations in a report |
| src/datafold_node/key_rotation_compliance.rs | Recommendation | struct | Individual recommendation details |
| src/events/correlation.rs | CorrelationSummary | struct | Summary of an event correlation group |
| src/tests/performance/mod.rs | PerformanceAnalysisConfig | struct | Config for performance analysis |
| src/tests/performance/mod.rs | ReportConfig | struct | Config for report generation in performance tests |
| src/tests/performance/mod.rs | ReportFormat | enum | Output formats for performance reports |
| src/tests/performance/mod.rs | ChartOptions | struct | Chart rendering options for reports |
| src/tests/performance/mod.rs | PerformanceMeasurement | struct | Results of a performance measurement |
| tests/security/cross_platform_validation.rs | CrossPlatformReport | struct | Report for cross-platform validation results |
| tests/security/cross_platform_validation.rs | SecurityEffectivenessSummary | struct | Summary of security effectiveness in validation |
| tests/security/cross_platform_validation.rs | CrossPlatformPerformanceAnalysis | struct | Performance analysis for cross-platform validation |
| tools/protocol-validation/src/lib.rs | ValidationSummary | struct | Summary of protocol validation results |
| tools/protocol-validation/src/report.rs | generate_html_report | fn/logic | Generates HTML report for protocol validation |
| tests/e2e_monitoring_and_reporting.rs | generate_comprehensive_report | fn/logic | Generates comprehensive E2E test report |
| tests/e2e_monitoring_and_reporting.rs | generate_executive_summary | fn/logic | Generates executive summary for E2E tests |

> This table will be expanded and refined as the audit continues. Duplicates, inconsistencies, and usage patterns will be documented in subsequent sections or tables as needed. 