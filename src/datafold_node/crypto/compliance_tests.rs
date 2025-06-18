//! Test suite for compliance functionality
//!
//! This module contains comprehensive tests for the compliance manager,
//! report generation, and integration tests.

#[cfg(test)]
mod tests {
    use super::super::{
        compliance_data::*,
        compliance_frameworks::*,
        compliance_manager::*,
        compliance_reporting::*,
    };
    // Removed unused imports after refactoring
    use crate::crypto::threat_monitor::RotationThreatMonitor;
    use crate::reporting::types::{TimeRange, UnifiedReportConfig, UnifiedReportFormat};
    use chrono::Datelike;
    use std::sync::Arc;

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

    #[test]
    fn test_compliance_control_types() {
        let controls = vec![
            ComplianceControl::AccessControls,
            ComplianceControl::DataIntegrity,
            ComplianceControl::SecurityMonitoring,
        ];
        
        // Test that controls can be compared and hashed
        let mut control_set = std::collections::HashSet::new();
        for control in controls {
            control_set.insert(control);
        }
        assert_eq!(control_set.len(), 3);
    }

    #[test]
    fn test_report_section_enum() {
        let sections = vec![
            ReportSection::ExecutiveSummary,
            ReportSection::RotationStatistics,
            ReportSection::SecurityIncidents,
            ReportSection::ControlAssessment,
            ReportSection::AuditTrail,
            ReportSection::RiskAssessment,
            ReportSection::PolicyCompliance,
            ReportSection::Recommendations,
            ReportSection::RawData,
        ];
        
        assert_eq!(sections.len(), 9);
    }

    #[test]
    fn test_compliance_status_enum() {
        let statuses = vec![
            ComplianceStatus::Compliant,
            ComplianceStatus::MostlyCompliant,
            ComplianceStatus::PartiallyCompliant,
            ComplianceStatus::NonCompliant,
            ComplianceStatus::UnderReview,
        ];
        
        assert_eq!(statuses.len(), 5);
        
        // Test that statuses can be compared
        assert_eq!(ComplianceStatus::Compliant, ComplianceStatus::Compliant);
        assert_ne!(ComplianceStatus::Compliant, ComplianceStatus::NonCompliant);
    }

    #[test]
    fn test_implementation_status_ordering() {
        let fully_implemented = ImplementationStatus::Implemented;
        let partially_implemented = ImplementationStatus::PartiallyImplemented;
        let not_implemented = ImplementationStatus::NotImplemented;
        
        assert_eq!(fully_implemented, ImplementationStatus::Implemented);
        assert_ne!(fully_implemented, partially_implemented);
        assert_ne!(partially_implemented, not_implemented);
    }
}