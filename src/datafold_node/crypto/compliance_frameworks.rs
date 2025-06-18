//! Compliance framework definitions and requirements
//!
//! This module defines the various compliance frameworks supported by the system
//! and their specific requirements and controls.

use serde::{Deserialize, Serialize};

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