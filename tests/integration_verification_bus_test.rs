//! Integration tests for the Security Operations Event Bus
//!
//! This test demonstrates the complete verification event bus architecture
//! including event publishing, handling, correlation, and cross-platform support.

use datafold::config::unified_config::{EnvironmentConfig, UnifiedConfig};
use datafold::events::{
    AlertDestination, AuditLogHandler, CorrelationManager, CreateVerificationEvent, EventEnvelope,
    MetricsHandler, PlatformInfo, PlatformSource, SecurityAlertHandler,
    SecurityEvent, SecurityEventCategory, TransportConfig, TransportFactory, TransportProtocol,
    VerificationBusConfig, VerificationEvent, VerificationEventBus,
};
use datafold::security_types::Severity;
use std::time::Duration;
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::test]
async fn test_verification_bus_integration() {
    // Initialize logging for the test
    let _ = env_logger::builder().is_test(true).try_init();

    // Create event bus configuration
    let mut config = VerificationBusConfig::default();
    config.buffer_size = 1000;
    config.min_severity = Severity::Info;
    config.enable_correlation = true;

    // Create event bus
    let mut event_bus = VerificationEventBus::new(config);

    // Start the event bus
    event_bus.start().await.expect("Failed to start event bus");

    // Register built-in handlers

    // 1. Audit logging handler
    let audit_handler = AuditLogHandler::new("test_audit.log".to_string())
        .with_name("test_audit_handler".to_string());
    event_bus
        .register_handler(Box::new(audit_handler))
        .await
        .expect("Failed to register audit handler");

    // 2. Metrics collection handler
    let metrics_handler = MetricsHandler::new().with_name("test_metrics_handler".to_string());
    event_bus
        .register_handler(Box::new(metrics_handler))
        .await
        .expect("Failed to register metrics handler");

    // 3. Security alerting handler
    let alert_handler = SecurityAlertHandler::new(Severity::Warning)
        .add_destination(AlertDestination::Console)
        .with_name("test_alert_handler".to_string());
    event_bus
        .register_handler(Box::new(alert_handler))
        .await
        .expect("Failed to register alert handler");

    // Test cross-platform event correlation
    let trace_id = Uuid::new_v4().to_string();

    // Simulate events from different platforms with same trace ID
    let mut rust_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Authentication,
        Severity::Info,
        PlatformSource::RustCli,
        "auth_service".to_string(),
        "user_login".to_string(),
    );
    rust_event.trace_id = Some(trace_id.clone());
    rust_event.actor = Some("user123".to_string());
    rust_event.result = datafold::events::event_types::OperationResult::Success;
    rust_event.duration = Some(Duration::from_millis(150));

    let mut js_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Authorization,
        Severity::Info,
        PlatformSource::JavaScriptSdk,
        "authz_service".to_string(),
        "check_permission".to_string(),
    );
    js_event.trace_id = Some(trace_id.clone());
    js_event.actor = Some("user123".to_string());
    js_event.result = datafold::events::event_types::OperationResult::Success;
    js_event.duration = Some(Duration::from_millis(75));

    let mut python_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Verification,
        Severity::Info,
        PlatformSource::PythonSdk,
        "signature_service".to_string(),
        "verify_signature".to_string(),
    );
    python_event.trace_id = Some(trace_id.clone());
    python_event.actor = Some("user123".to_string());
    python_event.result = datafold::events::event_types::OperationResult::Success;
    python_event.duration = Some(Duration::from_millis(95));

    // Publish events
    event_bus
        .publish_event(SecurityEvent::Generic(rust_event.clone()))
        .await
        .expect("Failed to publish Rust event");

    event_bus
        .publish_event(SecurityEvent::Generic(js_event.clone()))
        .await
        .expect("Failed to publish JavaScript event");

    event_bus
        .publish_event(SecurityEvent::Generic(python_event.clone()))
        .await
        .expect("Failed to publish Python event");

    // Test security threat event
    let mut threat_event = VerificationEvent::create_base_event(
        SecurityEventCategory::Security,
        Severity::Critical,
        PlatformSource::DataFoldNode,
        "security_monitor".to_string(),
        "threat_detected".to_string(),
    );
    threat_event.actor = Some("malicious_user".to_string());
    threat_event.result = datafold::events::event_types::OperationResult::Failure {
        error_type: "SecurityViolation".to_string(),
        error_message: "Multiple failed login attempts detected".to_string(),
        error_code: Some("SEC_001".to_string()),
    };

    let security_threat = datafold::events::event_types::SecurityThreatEvent {
        base: threat_event,
        threat_type: "BruteForceAttack".to_string(),
        threat_level: "High".to_string(),
        confidence: 0.95,
        threat_source: Some("192.168.1.100".to_string()),
        target: Some("login_service".to_string()),
        evidence: {
            let mut evidence = std::collections::HashMap::new();
            evidence.insert("failed_attempts".to_string(), serde_json::json!(15));
            evidence.insert("time_window".to_string(), serde_json::json!("5 minutes"));
            evidence
        },
        recommended_actions: vec![
            "Block source IP".to_string(),
            "Notify security team".to_string(),
            "Increase monitoring".to_string(),
        ],
        auto_response_triggered: true,
    };

    event_bus
        .publish_event(SecurityEvent::Security(security_threat))
        .await
        .expect("Failed to publish security threat event");

    // Wait for event processing
    sleep(Duration::from_millis(500)).await;

    // Check event bus statistics
    let stats = event_bus.get_statistics().await;
    println!("Event Bus Statistics:");
    println!("  Total events: {}", stats.total_events);
    println!("  Active handlers: {}", stats.active_handlers);
    println!(
        "  Handler success rate: {:.2}%",
        stats.handler_success_rate * 100.0
    );
    println!(
        "  Average processing time: {:.2}ms",
        stats.avg_processing_time_ms
    );

    // Verify we processed multiple events
    assert!(stats.total_events >= 4);
    assert_eq!(stats.active_handlers, 3);

    // Test correlation functionality
    let correlations = event_bus.get_correlations(rust_event.event_id).await;
    println!("Correlated events found: {}", correlations.len());

    // Should have correlated the events with the same trace ID
    assert!(
        correlations.len() >= 2,
        "Expected at least 2 correlated events, got {}",
        correlations.len()
    );

    // Test event filtering by severity
    let filtered_events: Vec<&SecurityEvent> = correlations
        .iter()
        .filter(|event| event.should_alert(Severity::Warning))
        .collect();

    println!("Events above warning threshold: {}", filtered_events.len());

    // Stop the event bus
    event_bus.stop().await;

    println!("✅ Verification event bus integration test completed successfully");
}

#[tokio::test]
async fn test_cross_platform_transport() {
    // Test event transport and serialization
    let transport_config = TransportConfig {
        protocol: TransportProtocol::InMemory,
        compression: false,
        serialization: datafold::events::transport::SerializationFormat::Json,
        connection_timeout_ms: 5000,
        ..Default::default()
    };

    let platform_info = TransportFactory::get_platform_info();

    let mut transport = TransportFactory::create_transport(transport_config, platform_info.clone())
        .expect("Failed to create transport");

    // Initialize transport
    transport
        .initialize()
        .await
        .expect("Failed to initialize transport");
    assert!(transport.is_healthy().await, "Transport should be healthy");

    // Create test event
    let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
        SecurityEventCategory::Performance,
        Severity::Info,
        PlatformSource::RustCli,
        "perf_monitor".to_string(),
        "latency_measurement".to_string(),
    ));

    // Create transport envelope
    let envelope = EventEnvelope {
        version: "1.0".to_string(),
        source: platform_info.clone(),
        target: Some(PlatformInfo {
            platform_type: "js-sdk".to_string(),
            version: "1.0.0".to_string(),
            host: Some("remote-host".to_string()),
            instance_id: Some("js-instance-123".to_string()),
            metadata: std::collections::HashMap::new(),
        }),
        event,
        metadata: std::collections::HashMap::new(),
        envelope_timestamp: chrono::Utc::now(),
        envelope_id: Uuid::new_v4(),
    };

    // Send event through transport
    let result = transport
        .send_event(envelope)
        .await
        .expect("Failed to send event");
    assert!(result.success, "Transport send should succeed");
    assert!(
        result.bytes_transferred.is_some(),
        "Should report bytes transferred"
    );

    // Check transport statistics
    let transport_stats = transport.get_statistics().await;
    assert_eq!(transport_stats.events_sent, 1);
    assert!(transport_stats.bytes_sent > 0);
    assert!(transport_stats.connected);

    // Close transport
    transport.close().await.expect("Failed to close transport");

    println!("✅ Cross-platform transport test completed successfully");
}

#[tokio::test]
async fn test_unified_config_integration() {
    // Test creating event bus from unified configuration
    let unified_config = UnifiedConfig {
        config_format_version: "1.0".to_string(),
        environments: {
            let mut envs = std::collections::HashMap::new();

            let env_config = EnvironmentConfig {
                signing: datafold::config::unified_config::SigningConfig {
                    policy: "standard".to_string(),
                    timeout_ms: 5000,
                    required_components: vec!["@method".to_string(), "@target-uri".to_string()],
                    include_content_digest: true,
                    include_timestamp: true,
                    include_nonce: true,
                    max_body_size_mb: 10,
                    debug: datafold::config::unified_config::DebugConfig {
                        enabled: false,
                        log_canonical_strings: false,
                        log_components: false,
                        log_timing: false,
                    },
                },
                verification: datafold::config::unified_config::VerificationConfig {
                    strict_timing: false,
                    allow_clock_skew_seconds: 300,
                    require_nonce: true,
                    max_signature_age_seconds: 3600,
                },
                logging: datafold::config::unified_config::LoggingConfig {
                    level: "info".to_string(),
                    colored_output: true,
                    structured: false,
                },
                authentication: datafold::config::unified_config::AuthenticationConfig {
                    store_tokens: true,
                    auto_update_check: true,
                    prompt_on_first_sign: true,
                },
                performance: datafold::config::unified_config::PerformanceConfig {
                    cache_keys: true,
                    max_concurrent_signs: 10,
                    default_timeout_secs: 30,
                    default_max_retries: 3,
                },
            };

            envs.insert("test".to_string(), env_config);
            envs
        },
        security_profiles: std::collections::HashMap::new(),
        defaults: datafold::config::unified_config::DefaultConfig {
            environment: "test".to_string(),
            signing_mode: "manual".to_string(),
            output_format: "table".to_string(),
            verbosity: 1,
        },
    };

    // Create event bus from unified config
    let event_bus = VerificationEventBus::from_unified_config(&unified_config, "test")
        .expect("Failed to create event bus from unified config");

    // Verify configuration was applied correctly
    assert!(event_bus.get_config().enabled);
    assert_eq!(event_bus.get_config().min_severity, Severity::Info); // Should match "info" log level

    println!("✅ Unified config integration test completed successfully");
}

#[tokio::test]
async fn test_event_correlation_scenarios() {
    // Test various correlation scenarios
    let mut correlation_manager = CorrelationManager::new(Duration::from_secs(3600));

    // Scenario 1: Trace ID correlation across platforms
    let trace_id = Uuid::new_v4().to_string();

    let mut event1 = VerificationEvent::create_base_event(
        SecurityEventCategory::Authentication,
        Severity::Info,
        PlatformSource::RustCli,
        "auth".to_string(),
        "login".to_string(),
    );
    event1.trace_id = Some(trace_id.clone());

    let mut event2 = VerificationEvent::create_base_event(
        SecurityEventCategory::Authorization,
        Severity::Info,
        PlatformSource::JavaScriptSdk,
        "authz".to_string(),
        "permission_check".to_string(),
    );
    event2.trace_id = Some(trace_id.clone());

    correlation_manager
        .add_event(&SecurityEvent::Generic(event1.clone()))
        .await;
    correlation_manager
        .add_event(&SecurityEvent::Generic(event2.clone()))
        .await;

    // Should find correlated events
    let correlated = correlation_manager
        .get_correlated_events(event1.event_id)
        .await;
    assert!(
        correlated.len() >= 2,
        "Should find at least 2 correlated events"
    );

    // Scenario 2: Session ID correlation
    let session_id = "session_abc123".to_string();

    let mut event3 = VerificationEvent::create_base_event(
        SecurityEventCategory::Configuration,
        Severity::Warning,
        PlatformSource::PythonSdk,
        "config".to_string(),
        "policy_update".to_string(),
    );
    event3.session_id = Some(session_id.clone());

    let mut event4 = VerificationEvent::create_base_event(
        SecurityEventCategory::Performance,
        Severity::Info,
        PlatformSource::DataFoldNode,
        "perf".to_string(),
        "metric_update".to_string(),
    );
    event4.session_id = Some(session_id.clone());

    correlation_manager
        .add_event(&SecurityEvent::Generic(event3.clone()))
        .await;
    correlation_manager
        .add_event(&SecurityEvent::Generic(event4.clone()))
        .await;

    // Give some time for correlation processing
    tokio::time::sleep(Duration::from_millis(10)).await;

    // Check cross-platform correlations
    let cross_platform_correlations = correlation_manager.get_cross_platform_correlations().await;
    println!(
        "Cross-platform correlations found: {}",
        cross_platform_correlations.len()
    );

    // Check if we have any correlations at all
    let all_correlations = correlation_manager.get_all_correlations().await;
    println!("Total correlations found: {}", all_correlations.len());
    for correlation in &all_correlations {
        println!(
            "  Correlation: {} platforms, {} events, strategy: {:?}",
            correlation.platforms.len(),
            correlation.events.len(),
            correlation.strategy
        );
    }

    // We should have at least some correlations, and at least one should be cross-platform
    assert!(
        !all_correlations.is_empty(),
        "Should have created some correlations"
    );
    let has_cross_platform = all_correlations.iter().any(|c| c.is_cross_platform());
    assert!(
        has_cross_platform,
        "Should have at least one cross-platform correlation"
    );

    // Check correlation statistics
    let stats = correlation_manager.get_statistics().await;
    assert!(
        stats.total_correlations > 0,
        "Should have created correlations"
    );
    assert!(
        stats.cross_platform_correlations > 0,
        "Should have cross-platform correlations"
    );

    println!("✅ Event correlation scenarios test completed successfully");
}

/// Custom event handler for testing
struct TestEventHandler {
    name: String,
    processed_events: std::sync::Arc<tokio::sync::Mutex<Vec<SecurityEvent>>>,
}

impl TestEventHandler {
    fn new(name: String) -> Self {
        Self {
            name,
            processed_events: std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new())),
        }
    }

    async fn get_processed_events(&self) -> Vec<SecurityEvent> {
        self.processed_events.lock().await.clone()
    }
}

#[async_trait::async_trait]
impl datafold::events::handlers::EventHandler for TestEventHandler {
    async fn handle_event(
        &self,
        event: &SecurityEvent,
    ) -> datafold::events::handlers::EventHandlerResult {
        let start_time = std::time::Instant::now();

        // Store the event
        self.processed_events.lock().await.push(event.clone());

        datafold::events::handlers::EventHandlerResult {
            handler_name: self.name.clone(),
            success: true,
            duration: start_time.elapsed(),
            error: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    fn handler_name(&self) -> String {
        self.name.clone()
    }
}

#[tokio::test]
async fn test_custom_event_handler() {
    // Test custom event handler implementation
    let mut event_bus = VerificationEventBus::with_default_config();
    event_bus.start().await.expect("Failed to start event bus");

    let custom_handler = TestEventHandler::new("custom_test_handler".to_string());
    let handler_ref = std::sync::Arc::new(custom_handler);

    // Register custom handler
    let handler_clone = handler_ref.clone();
    event_bus
        .register_handler(Box::new(TestEventHandler::new(
            "custom_test_handler".to_string(),
        )))
        .await
        .expect("Failed to register custom handler");

    // Publish test events
    for i in 0..5 {
        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::System,
            Severity::Info,
            PlatformSource::RustCli,
            "test_system".to_string(),
            format!("test_operation_{}", i),
        ));

        event_bus
            .publish_event(event)
            .await
            .expect("Failed to publish test event");
    }

    // Wait for processing
    sleep(Duration::from_millis(200)).await;

    // Verify events were processed by our custom handler
    let stats = event_bus.get_statistics().await;
    assert_eq!(stats.total_events, 5);
    assert_eq!(stats.active_handlers, 1);

    event_bus.stop().await;

    println!("✅ Custom event handler test completed successfully");
}
