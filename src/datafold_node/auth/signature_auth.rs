//! Ed25519 signature verification middleware for DataFold HTTP server (Refactored)
//!
//! This module serves as the main coordinator for the signature authentication system,
//! delegating to specialized modules while maintaining backward compatibility with
//! the existing public API.

// Re-export all public types and functions for backward compatibility
pub use super::{
    auth_config::{
        AttackDetectionConfig, RateLimitingConfig, ResponseSecurityConfig, SecurityLoggingConfig,
        SignatureAuthConfig,
    },
    auth_errors::{AuthenticationError, CustomAuthError, ErrorDetails, ErrorResponse},
    auth_middleware::{
        should_skip_verification, SecurityLogger, SignatureVerificationMiddleware,
        SignatureVerificationService, SignatureVerificationState,
    },
    auth_types::{
        AttackDetector, AttackPatternType, AuthenticatedClient, CacheStats, CacheWarmupResult,
        CachedPublicKey, ClientInfo, EnhancedSecurityMetricsCollector, LatencyHistogram,
        NonceStore, NonceStorePerformanceStats, NonceStoreStats, PerformanceAlert,
        PerformanceAlertType, PerformanceBreakdown, PerformanceMetrics, PerformanceMonitor,
        PublicKeyCache, RateLimiter, RequestInfo, SecurityEvent, SecurityEventType,
        SecurityMetrics, SecurityProfile, SignatureComponents, SlidingWindowStats,
        SuspiciousPattern, SystemHealthStatus,
    },
    key_management::{DetailedCacheStats, KeyManager},
    signature_verification::{verify_request_signature, SignatureVerifier},
    // Import the specialized modules that contain the implementations
    nonce_operations,
    rate_limiting,
    attack_detection,
    security_metrics,
};

// Legacy test module for backward compatibility
#[cfg(test)]
mod tests {
    // Legacy test module for backward compatibility
    // Note: The actual tests are now in the signature_auth_tests module
}
