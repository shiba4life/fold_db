//! Security Operations Event Bus Demo
//!
//! This example demonstrates how to integrate the verification event bus
//! with existing DataFold systems for security operations monitoring.

use datafold::events::{
    VerificationEventBus, VerificationBusConfig, SecurityEvent, EventSeverity,
    CreateVerificationEvent, VerificationEvent, SecurityEventCategory, PlatformSource,
    AuditLogHandler, MetricsHandler, SecurityAlertHandler, AlertDestination,
    AuthenticationEvent, AuthorizationEvent, SecurityThreatEvent,
};
use datafold::crypto::audit_logger::{CryptoAuditLogger, AuditConfig};
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 DataFold Security Operations Event Bus Demo");
    println!("================================================================");
    
    // Initialize logging
    env_logger::init();
    
    // Create event bus with custom configuration
    let config = VerificationBusConfig {
        enabled: true,
        buffer_size: 5000,
        min_severity: EventSeverity::Info,
        enable_correlation: true,
        correlation_window_minutes: 30,
        graceful_degradation: true,
        ..Default::default()
    };
    
    let mut event_bus = VerificationEventBus::new(config);
    
    // Start the event bus
    event_bus.start().await?;
    
    println!("✅ Event bus started successfully");
    
    // Register event handlers
    setup_event_handlers(&event_bus).await?;
    
    // Demonstrate cross-platform security monitoring
    println!("\n📊 Demonstrating cross-platform security monitoring...");
    
    // Simulate a user authentication flow across platforms
    let trace_id = Uuid::new_v4().to_string();
    let user_id = "user_12345";
    
    // 1. Initial authentication (Rust CLI)
    simulate_authentication_event(&event_bus, &trace_id, user_id).await?;
    
    // 2. Authorization check (JavaScript SDK)
    simulate_authorization_event(&event_bus, &trace_id, user_id).await?;
    
    // 3. Signature verification (Python SDK)
    simulate_verification_event(&event_bus, &trace_id, user_id).await?;
    
    // 4. Security threat detection (DataFold Node)
    simulate_security_threat(&event_bus).await?;
    
    // Wait for event processing
    sleep(Duration::from_millis(1000)).await;
    
    // Display results
    display_event_bus_statistics(&event_bus).await;
    
    // Demonstrate correlation capabilities
    demonstrate_event_correlation(&event_bus).await;
    
    // Integration with existing crypto audit logger
    demonstrate_crypto_integration().await?;
    
    // Cleanup
    event_bus.stop().await;
    
    println!("\n🎉 Demo completed successfully!");
    println!("================================================================");
    
    Ok(())
}

async fn setup_event_handlers(event_bus: &VerificationEventBus) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔧 Setting up event handlers...");
    
    // Audit logging handler
    let audit_handler = AuditLogHandler::new("demo_security_audit.log".to_string())
        .with_name("demo_audit_handler".to_string());
    event_bus.register_handler(Box::new(audit_handler)).await?;
    
    // Metrics collection handler
    let metrics_handler = MetricsHandler::new()
        .with_name("demo_metrics_handler".to_string())
        .with_detailed(true);
    event_bus.register_handler(Box::new(metrics_handler)).await?;
    
    // Security alerting handler with multiple destinations
    let alert_handler = SecurityAlertHandler::new(EventSeverity::Warning)
        .add_destination(AlertDestination::Console)
        .add_destination(AlertDestination::File { 
            path: "demo_security_alerts.log".to_string() 
        })
        .with_name("demo_alert_handler".to_string());
    event_bus.register_handler(Box::new(alert_handler)).await?;
    
    println!("✅ Event handlers registered successfully");
    Ok(())
}

async fn simulate_authentication_event(
    event_bus: &VerificationEventBus,
    trace_id: &str,
    user_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔐 Simulating authentication event (Rust CLI)...");
    
    let mut base_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Authentication,
        EventSeverity::Info,
        PlatformSource::RustCli,
        "auth_service".to_string(),
        "user_login".to_string(),
    );
    
    base_event.trace_id = Some(trace_id.to_string());
    base_event.actor = Some(user_id.to_string());
    base_event.result = datafold::events::event_types::OperationResult::Success;
    base_event.duration = Some(Duration::from_millis(245));
    base_event.environment = Some("production".to_string());
    
    let auth_event = AuthenticationEvent {
        base: base_event,
        auth_type: "password_login".to_string(),
        method: "pbkdf2".to_string(),
        key_id: Some("key_auth_001".to_string()),
        source_ip: Some("192.168.1.100".to_string()),
        user_agent: Some("DataFold-CLI/1.0".to_string()),
        mfa_used: true,
    };
    
    event_bus.publish_event(SecurityEvent::Authentication(auth_event)).await?;
    println!("  ✅ Authentication event published");
    
    Ok(())
}

async fn simulate_authorization_event(
    event_bus: &VerificationEventBus,
    trace_id: &str,
    user_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔑 Simulating authorization event (JavaScript SDK)...");
    
    let mut base_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Authorization,
        EventSeverity::Info,
        PlatformSource::JavaScriptSdk,
        "authz_service".to_string(),
        "check_resource_access".to_string(),
    );
    
    base_event.trace_id = Some(trace_id.to_string());
    base_event.actor = Some(user_id.to_string());
    base_event.result = datafold::events::event_types::OperationResult::Success;
    base_event.duration = Some(Duration::from_millis(89));
    
    let authz_event = AuthorizationEvent {
        base: base_event,
        resource: "/api/v1/secure-data".to_string(),
        action: "read".to_string(),
        policy: Some("resource_access_policy_v2".to_string()),
        decision: "allow".to_string(),
        reason: Some("User has required permissions".to_string()),
    };
    
    event_bus.publish_event(SecurityEvent::Authorization(authz_event)).await?;
    println!("  ✅ Authorization event published");
    
    Ok(())
}

async fn simulate_verification_event(
    event_bus: &VerificationEventBus,
    trace_id: &str,
    user_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("✅ Simulating verification event (Python SDK)...");
    
    let mut base_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Verification,
        EventSeverity::Info,
        PlatformSource::PythonSdk,
        "signature_service".to_string(),
        "verify_request_signature".to_string(),
    );
    
    base_event.trace_id = Some(trace_id.to_string());
    base_event.actor = Some(user_id.to_string());
    base_event.result = datafold::events::event_types::OperationResult::Success;
    base_event.duration = Some(Duration::from_millis(156));
    
    // Add verification-specific metadata
    base_event.metadata.insert(
        "signature_algorithm".to_string(),
        serde_json::Value::String("EdDSA".to_string())
    );
    base_event.metadata.insert(
        "key_id".to_string(),
        serde_json::Value::String("key_sign_001".to_string())
    );
    
    event_bus.publish_event(SecurityEvent::Generic(base_event)).await?;
    println!("  ✅ Verification event published");
    
    Ok(())
}

async fn simulate_security_threat(event_bus: &VerificationEventBus) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚨 Simulating security threat detection...");
    
    let mut threat_base = VerificationEvent::create_base_event(
        SecurityEventCategory::Security,
        EventSeverity::Critical,
        PlatformSource::DataFoldNode,
        "security_monitor".to_string(),
        "suspicious_activity_detected".to_string(),
    );
    
    threat_base.actor = Some("suspicious_user_456".to_string());
    threat_base.result = datafold::events::event_types::OperationResult::Failure {
        error_type: "SecurityViolation".to_string(),
        error_message: "Multiple failed authentication attempts from same IP".to_string(),
        error_code: Some("SEC_BRUTE_FORCE".to_string()),
    };
    
    let security_threat = SecurityThreatEvent {
        base: threat_base,
        threat_type: "BruteForceAttack".to_string(),
        threat_level: "High".to_string(),
        confidence: 0.89,
        threat_source: Some("203.0.113.50".to_string()),
        target: Some("auth_service".to_string()),
        evidence: {
            let mut evidence = std::collections::HashMap::new();
            evidence.insert("failed_attempts".to_string(), serde_json::json!(12));
            evidence.insert("time_window_minutes".to_string(), serde_json::json!(5));
            evidence.insert("source_country".to_string(), serde_json::json!("Unknown"));
            evidence
        },
        recommended_actions: vec![
            "Block source IP address".to_string(),
            "Notify security operations center".to_string(),
            "Increase monitoring for this user account".to_string(),
            "Review recent access patterns".to_string(),
        ],
        auto_response_triggered: true,
    };
    
    event_bus.publish_event(SecurityEvent::Security(security_threat)).await?;
    println!("  🚨 Security threat event published - CRITICAL ALERT!");
    
    Ok(())
}

async fn display_event_bus_statistics(event_bus: &VerificationEventBus) {
    println!("\n📈 Event Bus Statistics:");
    println!("----------------------------------------");
    
    let stats = event_bus.get_statistics().await;
    
    println!("Total Events Processed: {}", stats.total_events);
    println!("Active Handlers: {}", stats.active_handlers);
    println!("Handler Success Rate: {:.1}%", stats.handler_success_rate * 100.0);
    println!("Average Processing Time: {:.2}ms", stats.avg_processing_time_ms);
    println!("Events Dropped: {}", stats.dropped_events);
    println!("Uptime: {}s", stats.uptime_seconds);
    
    if !stats.events_by_severity.is_empty() {
        println!("\nEvents by Severity:");
        for (severity, count) in &stats.events_by_severity {
            println!("  {}: {}", severity, count);
        }
    }
    
    if !stats.events_by_category.is_empty() {
        println!("\nEvents by Category:");
        for (category, count) in &stats.events_by_category {
            println!("  {}: {}", category, count);
        }
    }
    
    if !stats.events_by_platform.is_empty() {
        println!("\nEvents by Platform:");
        for (platform, count) in &stats.events_by_platform {
            println!("  {}: {}", platform, count);
        }
    }
}

async fn demonstrate_event_correlation(event_bus: &VerificationEventBus) {
    println!("\n🔗 Event Correlation Analysis:");
    println!("----------------------------------------");
    
    // In a real implementation, we would get the first event ID from our trace
    // For this demo, we'll create a simple correlation demonstration
    
    let stats = event_bus.get_statistics().await;
    
    if stats.total_events > 1 {
        println!("Cross-platform correlation enabled: ✅");
        println!("Events can be correlated by:");
        println!("  • Trace ID (across all platforms)");
        println!("  • Session ID (within user sessions)");
        println!("  • Actor (user-based correlation)");
        println!("  • Temporal proximity (time-based patterns)");
        println!("  • Operation sequences (workflow analysis)");
        
        println!("\nBenefits of correlation:");
        println!("  • Complete security incident timelines");
        println!("  • Cross-platform attack pattern detection");
        println!("  • Faster incident response and investigation");
        println!("  • Comprehensive audit trails for compliance");
    } else {
        println!("Insufficient events for correlation analysis");
    }
}

async fn demonstrate_crypto_integration() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("\n🔐 Integration with Existing Crypto Systems:");
    println!("----------------------------------------");
    
    // Show how the event bus integrates with existing crypto audit logger
    let audit_config = AuditConfig::default();
    let crypto_logger = CryptoAuditLogger::new(audit_config);
    
    // Simulate crypto operation logging
    crypto_logger.log_encryption_operation(
        "aes_256_gcm_encrypt",
        "user_data_encryption",
        1024, // data size
        Duration::from_millis(45),
        datafold::crypto::audit_logger::OperationResult::Success,
        Some(Uuid::new_v4()),
    ).await;
    
    println!("✅ Crypto audit logger integration demonstrated");
    println!("The event bus can receive events from:");
    println!("  • Existing crypto audit logger");
    println!("  • Security monitor components");
    println!("  • Verification systems");
    println!("  • All SDK implementations");
    
    println!("\nThis creates a unified security operations view!");
    
    Ok(())
}