//! Tests for the Verification Event Bus
//!
//! This module contains all test code for the verification event bus functionality.

#[cfg(test)]
mod tests {
    use super::super::event_types::{
        CreateVerificationEvent, PlatformSource, SecurityEvent, SecurityEventCategory,
        VerificationEvent,
    };
    use super::super::verification_bus::VerificationEventBus;
    use super::super::verification_bus_config::VerificationBusConfig;
    use crate::security_types::Severity;

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = VerificationEventBus::with_default_config();
        assert!(bus.get_config().enabled);
        assert_eq!(bus.get_config().buffer_size, 10000);
    }

    #[tokio::test]
    async fn test_event_publishing() {
        let bus = VerificationEventBus::with_default_config();

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        let result = bus.publish_event(event).await;
        assert!(result.is_ok());

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 1);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let config = VerificationBusConfig {
            min_severity: Severity::Error,
            ..Default::default()
        };

        let bus = VerificationEventBus::new(config);

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        // This should be filtered out
        let info_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Performance,
            Severity::Info,
            PlatformSource::JavaScriptSdk,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        // This should pass through
        let error_event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Error,
            PlatformSource::PythonSdk,
            "security_component".to_string(),
            "security_operation".to_string(),
        ));

        bus.publish_event(info_event).await.unwrap();
        bus.publish_event(error_event).await.unwrap();

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 1); // Only error event should be counted
    }

    #[tokio::test]
    async fn test_statistics_tracking() {
        let bus = VerificationEventBus::with_default_config();

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        let event1 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "auth".to_string(),
            "login".to_string(),
        ));

        let event2 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Critical,
            PlatformSource::DataFoldNode,
            "security".to_string(),
            "threat_detected".to_string(),
        ));

        bus.publish_event(event1).await.unwrap();
        bus.publish_event(event2).await.unwrap();

        let stats = bus.get_statistics().await;
        assert_eq!(stats.total_events, 2);
        assert_eq!(stats.events_by_severity.get("Info"), Some(&1));
        assert_eq!(stats.events_by_severity.get("Critical"), Some(&1));
        assert_eq!(stats.events_by_category.get("Authentication"), Some(&1));
        assert_eq!(stats.events_by_category.get("Security"), Some(&1));
    }

    #[tokio::test]
    async fn test_config_from_default() {
        let config = VerificationBusConfig::default();
        assert!(config.enabled);
        assert_eq!(config.buffer_size, 10000);
        assert_eq!(config.processing_timeout_ms, 5000);
        assert_eq!(config.max_concurrent_handlers, 10);
        assert!(config.enable_persistence);
        assert_eq!(config.retention_hours, 24);
        assert_eq!(config.min_severity, Severity::Info);
        assert!(config.enable_correlation);
        assert_eq!(config.correlation_window_minutes, 60);
        assert!(config.graceful_degradation);
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.handler_timeout_ms, 3000);
    }

    #[tokio::test]
    async fn test_statistics_clearing() {
        let bus = VerificationEventBus::with_default_config();

        // Create a receiver to ensure the channel has subscribers
        let _receiver = bus.subscribe();

        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        bus.publish_event(event).await.unwrap();

        let stats_before = bus.get_statistics().await;
        assert_eq!(stats_before.total_events, 1);

        bus.clear_statistics().await;

        let stats_after = bus.get_statistics().await;
        assert_eq!(stats_after.total_events, 0);
        assert!(stats_after.events_by_severity.is_empty());
        assert!(stats_after.events_by_category.is_empty());
        assert!(stats_after.events_by_platform.is_empty());
    }
}