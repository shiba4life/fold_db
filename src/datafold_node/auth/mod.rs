//! Authentication functionality for DataFold node
//!
//! This module provides comprehensive authentication capabilities for the DataFold node,
//! organized into specialized sub-modules for maintainability and security.

// Core authentication modules
pub mod auth_config;
pub mod auth_errors;
pub mod auth_types;
pub mod signature_verification;
pub mod auth_middleware;
pub mod key_management;

// Specialized functionality modules
pub mod nonce_operations;
pub mod rate_limiting;
pub mod attack_detection;
pub mod security_metrics;

// Main signature authentication coordinator (maintains backward compatibility)
pub mod signature_auth;

// Consolidated test module
#[cfg(test)]
pub mod auth_tests;

// Legacy test module for compatibility
#[cfg(test)]
pub mod signature_auth_tests;

// Re-export public API for backward compatibility
pub use signature_auth::{
    SignatureAuthConfig, SignatureVerificationState,
    SignatureVerificationMiddleware, AuthenticationError
};

// Re-export key types for convenience
pub use auth_config::{
    AttackDetectionConfig, RateLimitingConfig, ResponseSecurityConfig, SecurityLoggingConfig,
};
pub use auth_errors::{CustomAuthError, ErrorDetails, ErrorResponse};
pub use auth_middleware::{should_skip_verification, SecurityLogger};
pub use auth_types::{
    AuthenticatedClient, CacheStats, ClientInfo, RequestInfo, SecurityEvent,
    SecurityEventType, SecurityMetrics, SecurityProfile
};
pub use key_management::{KeyManager, DetailedCacheStats};
pub use signature_verification::SignatureVerifier;