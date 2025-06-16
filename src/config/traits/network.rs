//! Network-specific configuration traits
//!
//! This module provides network-specific configuration traits that extend the base
//! configuration system with network-specific validation, monitoring, and optimization.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use super::base::BaseConfig;
use super::error::{TraitConfigError, TraitConfigResult, ValidationContext};

/// Network configuration domain trait
///
/// Provides network-specific configuration capabilities including connection management,
/// protocol validation, and network performance optimization.
#[async_trait]
pub trait NetworkConfig: BaseConfig {
    /// Validate network connectivity parameters
    ///
    /// Performs comprehensive validation of network parameters including ports,
    /// addresses, protocols, and connection limits.
    fn validate_network_parameters(&self) -> TraitConfigResult<()>;

    /// Validate port ranges and accessibility
    ///
    /// Ensures ports are within valid ranges and available for binding.
    fn validate_port_configuration(&self) -> TraitConfigResult<()>;

    /// Validate connection and timeout settings
    ///
    /// Ensures timeout values are reasonable and connection limits are appropriate.
    fn validate_connection_settings(&self) -> TraitConfigResult<()>;

    /// Get network health metrics
    ///
    /// Returns current network health and performance metrics.
    async fn get_network_health(&self) -> TraitConfigResult<NetworkHealthMetrics>;

    /// Test network connectivity
    ///
    /// Performs actual network connectivity tests to validate configuration.
    async fn test_connectivity(&self) -> TraitConfigResult<ConnectivityTestResult>;

    /// Get platform-specific network optimization settings
    ///
    /// Returns network settings optimized for the current platform.
    fn get_platform_network_settings(&self) -> NetworkPlatformSettings;

    /// Validate network security settings
    ///
    /// Ensures network security configurations are properly set.
    fn validate_network_security(&self) -> TraitConfigResult<()>;
}

/// Security configuration domain trait
///
/// Provides security-specific configuration capabilities including cryptographic
/// parameter validation, security monitoring, and compliance checking.
pub trait SecurityConfig: BaseConfig {
    /// Validate cryptographic parameters
    ///
    /// Ensures cryptographic settings meet security standards and are compatible.
    fn validate_crypto_parameters(&self) -> TraitConfigResult<()>;

    /// Validate security policy compliance
    ///
    /// Checks that configuration meets security policy requirements.
    fn validate_security_policy(&self) -> TraitConfigResult<()>;

    /// Check for security vulnerabilities
    ///
    /// Scans configuration for known security vulnerabilities.
    fn check_security_vulnerabilities(&self) -> TraitConfigResult<Vec<SecurityVulnerability>>;

    /// Get security strength assessment
    ///
    /// Returns an assessment of the overall security strength of the configuration.
    fn assess_security_strength(&self) -> TraitConfigResult<SecurityStrengthAssessment>;

    /// Validate key management settings
    ///
    /// Ensures key management and rotation settings are secure.
    fn validate_key_management(&self) -> TraitConfigResult<()>;

    /// Generate security compliance report
    ///
    /// Creates a detailed report of security compliance status.
    fn generate_compliance_report(&self) -> TraitConfigResult<SecurityComplianceReport>;
}

/// Network health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    /// Connection success rate (0.0 to 1.0)
    pub connection_success_rate: f64,

    /// Average connection latency in milliseconds
    pub avg_latency_ms: f64,

    /// Current active connections
    pub active_connections: u32,

    /// Maximum concurrent connections reached
    pub max_connections_reached: u32,

    /// Number of connection failures
    pub connection_failures: u64,

    /// Number of timeout events
    pub timeout_events: u64,

    /// Network throughput in bytes per second
    pub throughput_bps: u64,

    /// Packet loss rate (0.0 to 1.0)
    pub packet_loss_rate: f64,

    /// DNS resolution time in milliseconds
    pub dns_resolution_ms: f64,

    /// Last health check timestamp
    pub last_check: chrono::DateTime<chrono::Utc>,
}

/// Connectivity test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityTestResult {
    /// Overall connectivity status
    pub status: ConnectivityStatus,

    /// Individual test results
    pub tests: Vec<ConnectivityTest>,

    /// Total test duration in milliseconds
    pub total_duration_ms: f64,

    /// Test timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Connectivity status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectivityStatus {
    /// All connectivity tests passed
    Healthy,

    /// Some tests failed but core connectivity works
    Degraded,

    /// Significant connectivity issues
    Unhealthy,

    /// No connectivity
    Failed,
}

/// Individual connectivity test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectivityTest {
    /// Test name
    pub name: String,

    /// Test result
    pub success: bool,

    /// Test duration in milliseconds
    pub duration_ms: f64,

    /// Error message if test failed
    pub error: Option<String>,

    /// Additional test metadata
    pub metadata: HashMap<String, String>,
}

/// Platform-specific network settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlatformSettings {
    /// Optimal socket buffer sizes
    pub socket_buffer_sizes: SocketBufferSizes,

    /// TCP-specific settings
    pub tcp_settings: TcpSettings,

    /// UDP-specific settings
    pub udp_settings: UdpSettings,

    /// Platform-specific optimizations
    pub platform_optimizations: HashMap<String, bool>,
}

/// Socket buffer size configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketBufferSizes {
    /// Send buffer size
    pub send_buffer: usize,

    /// Receive buffer size
    pub receive_buffer: usize,

    /// Maximum segment size
    pub max_segment_size: Option<usize>,
}

/// TCP-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpSettings {
    /// Enable TCP keepalive
    pub keepalive: bool,

    /// Keepalive idle time in seconds
    pub keepalive_idle_secs: u32,

    /// Keepalive interval in seconds
    pub keepalive_interval_secs: u32,

    /// Number of keepalive probes
    pub keepalive_probes: u32,

    /// Enable Nagle's algorithm
    pub nagle_algorithm: bool,
}

/// UDP-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UdpSettings {
    /// Enable broadcast
    pub broadcast: bool,

    /// Enable multicast
    pub multicast: bool,

    /// Multicast TTL
    pub multicast_ttl: Option<u32>,
}

/// Security vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    /// Vulnerability identifier
    pub id: String,

    /// Vulnerability severity
    pub severity: SecuritySeverity,

    /// Vulnerability description
    pub description: String,

    /// Affected configuration fields
    pub affected_fields: Vec<String>,

    /// Remediation recommendations
    pub remediation: Vec<String>,

    /// CVSS score if applicable
    pub cvss_score: Option<f64>,
}

/// Security severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    /// Critical security issue
    Critical,

    /// High security issue
    High,

    /// Medium security issue
    Medium,

    /// Low security issue
    Low,

    /// Informational security note
    Info,
}

/// Security strength assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStrengthAssessment {
    /// Overall security score (0-100)
    pub overall_score: u8,

    /// Individual security component scores
    pub component_scores: HashMap<String, u8>,

    /// Security strengths
    pub strengths: Vec<String>,

    /// Security weaknesses
    pub weaknesses: Vec<String>,

    /// Recommendations for improvement
    pub recommendations: Vec<String>,

    /// Assessment timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Security compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityComplianceReport {
    /// Compliance framework (e.g., "NIST", "ISO27001")
    pub framework: String,

    /// Overall compliance status
    pub compliance_status: ComplianceStatus,

    /// Compliance score (0-100)
    pub compliance_score: u8,

    /// Individual compliance checks
    pub checks: Vec<ComplianceCheck>,

    /// Non-compliant items
    pub non_compliant_items: Vec<String>,

    /// Recommendations for compliance
    pub recommendations: Vec<String>,

    /// Report generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
}

/// Compliance status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceStatus {
    /// Fully compliant
    Compliant,

    /// Mostly compliant with minor issues
    PartiallyCompliant,

    /// Significant compliance issues
    NonCompliant,

    /// Compliance assessment not applicable
    NotApplicable,
}

/// Individual compliance check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    /// Check identifier
    pub check_id: String,

    /// Check description
    pub description: String,

    /// Check result
    pub result: ComplianceCheckResult,

    /// Evidence or justification
    pub evidence: Option<String>,

    /// Remediation if non-compliant
    pub remediation: Option<String>,
}

/// Compliance check result
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceCheckResult {
    /// Check passed
    Pass,

    /// Check failed
    Fail,

    /// Check not applicable
    NotApplicable,

    /// Check skipped
    Skipped,
}

impl Default for NetworkPlatformSettings {
    fn default() -> Self {
        Self {
            socket_buffer_sizes: SocketBufferSizes {
                send_buffer: 65536,
                receive_buffer: 65536,
                max_segment_size: None,
            },
            tcp_settings: TcpSettings {
                keepalive: true,
                keepalive_idle_secs: 7200,
                keepalive_interval_secs: 75,
                keepalive_probes: 9,
                nagle_algorithm: true,
            },
            udp_settings: UdpSettings {
                broadcast: false,
                multicast: false,
                multicast_ttl: None,
            },
            platform_optimizations: HashMap::new(),
        }
    }
}

impl NetworkHealthMetrics {
    /// Create a new network health metrics instance
    pub fn new() -> Self {
        Self {
            connection_success_rate: 0.0,
            avg_latency_ms: 0.0,
            active_connections: 0,
            max_connections_reached: 0,
            connection_failures: 0,
            timeout_events: 0,
            throughput_bps: 0,
            packet_loss_rate: 0.0,
            dns_resolution_ms: 0.0,
            last_check: chrono::Utc::now(),
        }
    }

    /// Check if network health is acceptable
    pub fn is_healthy(&self) -> bool {
        self.connection_success_rate >= 0.95
            && self.packet_loss_rate <= 0.01
            && self.avg_latency_ms <= 100.0
    }
}

impl ConnectivityTestResult {
    /// Create a new connectivity test result
    pub fn new() -> Self {
        Self {
            status: ConnectivityStatus::Healthy,
            tests: Vec::new(),
            total_duration_ms: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add a test result
    pub fn add_test(&mut self, test: ConnectivityTest) {
        self.total_duration_ms += test.duration_ms;
        if !test.success {
            self.status = match self.status {
                ConnectivityStatus::Healthy => ConnectivityStatus::Degraded,
                ConnectivityStatus::Degraded => ConnectivityStatus::Unhealthy,
                _ => ConnectivityStatus::Failed,
            };
        }
        self.tests.push(test);
    }
}

impl SecurityStrengthAssessment {
    /// Create a new security strength assessment
    pub fn new() -> Self {
        Self {
            overall_score: 0,
            component_scores: HashMap::new(),
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            recommendations: Vec::new(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Check if security strength is acceptable
    pub fn is_secure(&self) -> bool {
        self.overall_score >= 80
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_health_metrics() {
        let mut metrics = NetworkHealthMetrics::new();
        assert!(!metrics.is_healthy());

        metrics.connection_success_rate = 0.98;
        metrics.packet_loss_rate = 0.005;
        metrics.avg_latency_ms = 50.0;
        assert!(metrics.is_healthy());
    }

    #[test]
    fn test_connectivity_test_result() {
        let mut result = ConnectivityTestResult::new();
        assert_eq!(result.status, ConnectivityStatus::Healthy);

        result.add_test(ConnectivityTest {
            name: "dns_test".to_string(),
            success: false,
            duration_ms: 100.0,
            error: Some("DNS resolution failed".to_string()),
            metadata: HashMap::new(),
        });

        assert_eq!(result.status, ConnectivityStatus::Degraded);
        assert_eq!(result.total_duration_ms, 100.0);
    }

    #[test]
    fn test_security_strength_assessment() {
        let mut assessment = SecurityStrengthAssessment::new();
        assert!(!assessment.is_secure());

        assessment.overall_score = 85;
        assert!(assessment.is_secure());
    }

    #[test]
    fn test_security_severity_ordering() {
        assert!(SecuritySeverity::Critical > SecuritySeverity::High);
        assert!(SecuritySeverity::High > SecuritySeverity::Medium);
        assert!(SecuritySeverity::Medium > SecuritySeverity::Low);
        assert!(SecuritySeverity::Low > SecuritySeverity::Info);
    }
}
