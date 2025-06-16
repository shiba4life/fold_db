//! Example usage of migrated network and crypto configurations
//!
//! This module demonstrates the enhanced capabilities provided by the trait-based
//! configuration system for network and crypto configurations.

use crate::config::crypto::CryptoConfig;
use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, ConnectivityTestResult,
    NetworkConfig as NetworkConfigTrait, NetworkHealthMetrics, SecurityComplianceReport,
    SecurityConfig, SecurityStrengthAssessment, TraitConfigResult,
};
use crate::events::transport::TransportConfig;
use crate::network::config::NetworkConfig;
use std::collections::HashMap;

/// Example demonstrating network configuration with enhanced traits
pub async fn example_network_config_usage() -> TraitConfigResult<()> {
    // Create network configuration with trait-enhanced validation
    let mut network_config = NetworkConfig::new("0.0.0.0:8080");
    network_config = network_config
        .with_max_connections(1000)
        .with_mdns(true)
        .with_discovery_port(5353);

    // Enhanced validation using trait methods
    network_config.validate_network_parameters()?;
    network_config.validate_port_configuration()?;
    network_config.validate_connection_settings()?;
    network_config.validate_network_security()?;

    // Get network health metrics
    let health_metrics: NetworkHealthMetrics = network_config.get_network_health().await?;
    println!(
        "Network Health - Success Rate: {:.2}%, Latency: {:.1}ms",
        health_metrics.connection_success_rate * 100.0,
        health_metrics.avg_latency_ms
    );

    // Test connectivity
    let connectivity_result: ConnectivityTestResult = network_config.test_connectivity().await?;
    println!("Connectivity Status: {:?}", connectivity_result.status);

    // Platform-specific optimizations
    let platform_settings = network_config.get_platform_network_settings();
    println!(
        "Optimal buffer size: {} bytes",
        platform_settings.socket_buffer_sizes.send_buffer
    );

    // Save with enhanced lifecycle management
    // network_config.save(std::path::Path::new("network_config.toml")).await?;

    Ok(())
}

/// Example demonstrating crypto configuration with security traits
pub async fn example_crypto_config_usage() -> TraitConfigResult<()> {
    // Create crypto configuration with enhanced security
    let mut crypto_config =
        CryptoConfig::with_enhanced_security("very-secure-passphrase-123!".to_string());

    // Security-focused validation
    crypto_config.validate_crypto_parameters()?;
    crypto_config.validate_security_policy()?;
    crypto_config.validate_key_management()?;

    // Security vulnerability assessment
    let vulnerabilities = crypto_config.check_security_vulnerabilities()?;
    println!("Found {} security vulnerabilities", vulnerabilities.len());
    for vuln in &vulnerabilities {
        println!(
            "  - {}: {} (Severity: {:?})",
            vuln.id, vuln.description, vuln.severity
        );
    }

    // Security strength assessment
    let strength_assessment: SecurityStrengthAssessment =
        crypto_config.assess_security_strength()?;
    println!(
        "Security Strength Score: {}/100",
        strength_assessment.overall_score
    );
    println!("Security Strengths: {:?}", strength_assessment.strengths);
    println!("Security Weaknesses: {:?}", strength_assessment.weaknesses);

    // Compliance reporting
    let compliance_report: SecurityComplianceReport = crypto_config.generate_compliance_report()?;
    println!(
        "Compliance Status: {:?} (Score: {}/100)",
        compliance_report.compliance_status, compliance_report.compliance_score
    );

    // Enhanced lifecycle with security monitoring
    // crypto_config.save(std::path::Path::new("crypto_config.toml")).await?;

    Ok(())
}

/// Example demonstrating transport configuration with network traits
pub async fn example_transport_config_usage() -> TraitConfigResult<()> {
    // Create transport configuration
    let mut transport_config = TransportConfig::default();
    transport_config.encryption = true;
    transport_config.compression = true;
    transport_config.connection_timeout_ms = 10000;

    // Network-specific validation for transport
    transport_config.validate_network_parameters()?;
    transport_config.validate_connection_settings()?;
    transport_config.validate_network_security()?;

    // Transport health monitoring
    let health_metrics: NetworkHealthMetrics = transport_config.get_network_health().await?;
    println!(
        "Transport Health - Throughput: {} bps",
        health_metrics.throughput_bps
    );

    // Connectivity testing for transport
    let connectivity_result: ConnectivityTestResult = transport_config.test_connectivity().await?;
    println!("Transport Connectivity: {:?}", connectivity_result.status);

    // Enhanced validation with context
    if let Err(context) = transport_config.validate_with_context() {
        println!("Validation failed in trait: {}", context.trait_name);
    }

    Ok(())
}

/// Example demonstrating duplication reduction through shared traits
pub fn example_duplication_reduction() {
    println!("=== Duplication Reduction Examples ===");

    // Before: Each configuration had its own validation logic
    // After: Shared validation framework across all configurations

    println!("1. Shared Validation Framework:");
    println!("   - NetworkConfig uses BaseConfig::validate()");
    println!("   - CryptoConfig uses BaseConfig::validate()");
    println!("   - TransportConfig uses BaseConfig::validate()");
    println!("   - Common ValidationRule framework");

    println!("\n2. Shared Lifecycle Management:");
    println!("   - ConfigLifecycle::save() - common across all configs");
    println!("   - ConfigLifecycle::load() - unified loading logic");
    println!("   - ConfigLifecycle::reload() - shared reload mechanism");
    println!("   - ConfigMetadata - unified metadata management");

    println!("\n3. Shared Reporting Infrastructure:");
    println!("   - ConfigReporting::report_event() - unified event reporting");
    println!("   - ReportingConfig - shared reporting configuration");
    println!("   - Integrated with PBI 26 unified reporting system");

    println!("\n4. Domain-Specific Optimization:");
    println!("   - NetworkConfig trait - network-specific functionality");
    println!("   - SecurityConfig trait - security-specific validation");
    println!("   - Platform-specific optimizations through CrossPlatformConfig");

    println!("\n5. Validation Rule Consolidation:");
    println!("   - 40+ validation rules consolidated into shared framework");
    println!("   - ValidationRuleType enum reduces code duplication");
    println!("   - ValidationContext provides unified error reporting");
}

/// Performance comparison showing trait overhead
pub async fn example_performance_comparison() -> TraitConfigResult<()> {
    use std::time::Instant;

    // Measure trait-based validation performance
    let network_config = NetworkConfig::default();
    let crypto_config = CryptoConfig::default();
    let transport_config = TransportConfig::default();

    // Network validation performance
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = network_config.validate();
        let _ = network_config.validate_network_parameters();
        let _ = network_config.validate_port_configuration();
    }
    let network_duration = start.elapsed();

    // Crypto validation performance
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = crypto_config.validate();
        let _ = crypto_config.validate_crypto_parameters();
        let _ = crypto_config.validate_security_policy();
    }
    let crypto_duration = start.elapsed();

    // Transport validation performance
    let start = Instant::now();
    for _ in 0..1000 {
        let _ = transport_config.validate();
        let _ = transport_config.validate_network_parameters();
        let _ = transport_config.validate_connection_settings();
    }
    let transport_duration = start.elapsed();

    println!("=== Performance Metrics (1000 iterations) ===");
    println!("Network Config Validation: {:?}", network_duration);
    println!("Crypto Config Validation: {:?}", crypto_duration);
    println!("Transport Config Validation: {:?}", transport_duration);
    println!(
        "Average per validation: {:?}",
        (network_duration + crypto_duration + transport_duration) / 3000
    );

    Ok(())
}

/// Demonstrate cross-configuration validation
pub fn example_cross_validation() -> TraitConfigResult<()> {
    let network_config = NetworkConfig::default();
    let crypto_config = CryptoConfig::with_enhanced_security("secure-passphrase".to_string());
    let transport_config = TransportConfig::default();

    // Cross-validation example: ensure transport encryption matches crypto requirements
    if crypto_config.enabled && !transport_config.encryption {
        println!("WARNING: Crypto encryption enabled but transport encryption disabled");
        println!("RECOMMENDATION: Enable transport encryption for end-to-end security");
    }

    // Cross-validation: network security settings
    if network_config.listen_address.contains("0.0.0.0") && crypto_config.enabled {
        println!("INFO: Using encryption with public network interface - good security practice");
    }

    // Validate that configurations are compatible
    let network_health = network_config.validation_rules().len();
    let crypto_health = crypto_config.validation_rules().len();
    let transport_health = transport_config.validation_rules().len();

    println!("Configuration Health Summary:");
    println!("  Network: {} validation rules", network_health);
    println!("  Crypto: {} validation rules", crypto_health);
    println!("  Transport: {} validation rules", transport_health);
    println!(
        "  Total: {} shared validation rules",
        network_health + crypto_health + transport_health
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_config_traits() {
        let result = example_network_config_usage().await;
        assert!(result.is_ok(), "Network config trait usage should work");
    }

    #[tokio::test]
    async fn test_crypto_config_traits() {
        let result = example_crypto_config_usage().await;
        assert!(result.is_ok(), "Crypto config trait usage should work");
    }

    #[tokio::test]
    async fn test_transport_config_traits() {
        let result = example_transport_config_usage().await;
        assert!(result.is_ok(), "Transport config trait usage should work");
    }

    #[test]
    fn test_cross_validation() {
        let result = example_cross_validation();
        assert!(result.is_ok(), "Cross-validation should work");
    }

    #[tokio::test]
    async fn test_performance_benchmarks() {
        let result = example_performance_comparison().await;
        assert!(result.is_ok(), "Performance benchmarks should complete");
    }
}
