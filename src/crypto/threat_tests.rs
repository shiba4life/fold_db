//! Test suite for rotation threat monitoring
//!
//! This module contains comprehensive tests for the threat detection algorithms,
//! integration tests, and validation of the threat monitoring system.

#[cfg(test)]
mod tests {
    use super::super::threat_monitor::RotationThreatMonitor;
    use super::super::threat_types::{RotationThreatPattern};
    use super::super::threat_detection::RotationActivity;
    use super::super::key_rotation::{KeyRotationRequest, RotationReason};
    use super::super::key_rotation_audit::KeyRotationSecurityMetadata;
    use crate::crypto::generate_master_keypair;
    use crate::security_types::ThreatLevel;
    use std::collections::HashMap;
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::{Timelike, Utc};

    #[tokio::test]
    #[ignore = "Disabled due to admin functionality removal"]
    async fn test_frequent_requests_detection() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        let user_id = "test-user";
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Simulate multiple rotation requests
        for i in 0..15 {
            let old_keypair = generate_master_keypair().unwrap();
            let new_keypair = generate_master_keypair().unwrap();
            let old_private_key = old_keypair.private_key();

            let request = KeyRotationRequest::new(
                &old_private_key,
                new_keypair.public_key().clone(),
                RotationReason::UserInitiated,
                Some("test-client".to_string()),
                HashMap::new(),
            )
            .unwrap();

            let security_metadata = KeyRotationSecurityMetadata {
                source_ip: Some(ip),
                user_agent: Some("DataFold-CLI/1.0".to_string()),
                geolocation: None,
                session_info: None,
                device_fingerprint: Some("test-device".to_string()),
                auth_method: Some("signature".to_string()),
                risk_score: Some(0.1),
                request_source: Some("CLI".to_string()),
            };

            let detections = threat_monitor
                .monitor_rotation_request(
                    Uuid::new_v4(),
                    &request,
                    &security_metadata,
                    Some(user_id),
                    true,
                    0.1,
                )
                .await;

            // Should detect frequent requests after threshold
            if i >= 10 {
                assert!(!detections.is_empty());
                let detection = &detections[0];
                assert_eq!(
                    detection.rotation_pattern,
                    RotationThreatPattern::FrequentRotationRequests
                );
            }
        }
    }

    #[tokio::test]
    async fn test_off_hours_detection() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let _threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        // Create activity during off-hours (2 AM)
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::UserInitiated,
            Some("test-client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        let security_metadata = KeyRotationSecurityMetadata {
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            user_agent: Some("DataFold-CLI/1.0".to_string()),
            geolocation: None,
            session_info: None,
            device_fingerprint: Some("test-device".to_string()),
            auth_method: Some("signature".to_string()),
            risk_score: Some(0.1),
            request_source: Some("CLI".to_string()),
        };

        let activity = RotationActivity::new(
            Uuid::new_v4(),
            &request,
            &security_metadata,
            Some("test-user"),
            true,
            0.1,
        );

        // Manually adjust timestamp to 2 AM
        let activity = RotationActivity {
            timestamp: Utc::now().with_hour(2).unwrap(),
            ..activity
        };

        let detection_engine = super::super::threat_detection::ThreatDetectionEngine::new(0.7);
        let detection = detection_engine.detect_off_hours_rotation(&activity).await;
        assert!(detection.is_some());

        let detection = detection.unwrap();
        assert_eq!(
            detection.rotation_pattern,
            RotationThreatPattern::OffHoursRotation
        );
        assert_eq!(detection.base_detection.threat_level, ThreatLevel::Low);
    }

    #[tokio::test]
    async fn test_threat_status_summary() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        let status = threat_monitor.get_threat_status().await;
        assert_eq!(status.active_threat_count, 0);
        assert_eq!(status.overall_threat_level, ThreatLevel::Low);
        assert_eq!(status.recent_activity_count, 0);
    }

    #[tokio::test]
    async fn test_repeated_failures_detection() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let mut config = super::super::threat_config::RotationThreatMonitorConfig::default();
        config.min_confidence_threshold = 0.5; // Lower threshold for testing
        
        let threat_monitor = RotationThreatMonitor::new(
            config,
            base_monitor,
            audit_logger,
            security_manager,
        );

        let user_id = "test-user";
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Simulate multiple failed rotation requests
        for _i in 0..12 {
            let old_keypair = generate_master_keypair().unwrap();
            let new_keypair = generate_master_keypair().unwrap();
            let old_private_key = old_keypair.private_key();

            let request = KeyRotationRequest::new(
                &old_private_key,
                new_keypair.public_key().clone(),
                RotationReason::UserInitiated,
                Some("test-client".to_string()),
                HashMap::new(),
            )
            .unwrap();

            let security_metadata = KeyRotationSecurityMetadata {
                source_ip: Some(ip),
                user_agent: Some("DataFold-CLI/1.0".to_string()),
                geolocation: None,
                session_info: None,
                device_fingerprint: Some("test-device".to_string()),
                auth_method: Some("signature".to_string()),
                risk_score: Some(0.1),
                request_source: Some("CLI".to_string()),
            };

            let detections = threat_monitor
                .monitor_rotation_request(
                    Uuid::new_v4(),
                    &request,
                    &security_metadata,
                    Some(user_id),
                    false, // Simulate failure
                    0.1,
                )
                .await;

            // Should detect repeated failures after threshold
            if _i >= 9 {
                assert!(!detections.is_empty());
                let detection = &detections[0];
                assert_eq!(
                    detection.rotation_pattern,
                    RotationThreatPattern::RepeatedRotationFailures
                );
                assert_eq!(detection.base_detection.threat_level, ThreatLevel::High);
            }
        }
    }

    #[tokio::test]
    async fn test_cleanup_old_data() {
        let base_logger =
            Arc::new(super::super::audit_logger::CryptoAuditLogger::with_default_config());
        let audit_logger =
            Arc::new(super::super::key_rotation_audit::KeyRotationAuditLogger::new(base_logger));
        let base_monitor =
            Arc::new(super::super::security_monitor::CryptoSecurityMonitor::with_default_config());
        let security_manager = Arc::new(
            super::super::key_rotation_security::KeyRotationSecurityManager::with_default_policy(
                audit_logger.clone(),
            ),
        );

        let threat_monitor = RotationThreatMonitor::with_default_config(
            base_monitor,
            audit_logger,
            security_manager,
        );

        // Add some test activity
        let old_keypair = generate_master_keypair().unwrap();
        let new_keypair = generate_master_keypair().unwrap();
        let old_private_key = old_keypair.private_key();

        let request = KeyRotationRequest::new(
            &old_private_key,
            new_keypair.public_key().clone(),
            RotationReason::UserInitiated,
            Some("test-client".to_string()),
            HashMap::new(),
        )
        .unwrap();

        let security_metadata = KeyRotationSecurityMetadata {
            source_ip: Some(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))),
            user_agent: Some("DataFold-CLI/1.0".to_string()),
            geolocation: None,
            session_info: None,
            device_fingerprint: Some("test-device".to_string()),
            auth_method: Some("signature".to_string()),
            risk_score: Some(0.1),
            request_source: Some("CLI".to_string()),
        };

        threat_monitor
            .monitor_rotation_request(
                Uuid::new_v4(),
                &request,
                &security_metadata,
                Some("test-user"),
                true,
                0.1,
            )
            .await;

        let status_before = threat_monitor.get_threat_status().await;
        assert_eq!(status_before.recent_activity_count, 1);

        // Clean up data older than 0 hours (should remove everything)
        threat_monitor.cleanup_old_data(0).await;

        let status_after = threat_monitor.get_threat_status().await;
        assert_eq!(status_after.recent_activity_count, 0);
    }
}