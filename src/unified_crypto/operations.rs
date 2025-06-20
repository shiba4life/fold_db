//! # Operational Cryptographic Layer
//!
//! This module provides high-level cryptographic operations built on top of the primitives layer.
//! It coordinates complex cryptographic workflows and provides user-friendly APIs for database
//! operations, authentication, network security, backup operations, and CLI integration.
//!
//! ## Architecture
//!
//! The operational layer acts as a coordinator between different specialized operational modules:
//! - **Database Operations**: High-level database encryption and key management
//! - **Authentication Operations**: Multi-factor authentication and signature workflows  
//! - **Network Security Operations**: Secure communication and key exchange protocols
//! - **Backup Operations**: Encrypted backup creation and secure recovery
//! - **CLI Operations**: User-facing cryptographic command implementations
//!
//! ## Security Model
//!
//! The operational layer maintains strict security boundaries:
//! - All operations are built on verified primitives from the primitives layer
//! - Comprehensive input validation and sanitization at the operational boundary
//! - Session management and authentication state tracking
//! - Rate limiting and abuse protection mechanisms
//! - Comprehensive audit logging for all operational activities
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::unified_crypto::{UnifiedCrypto, CryptoOperations, CryptoConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize unified crypto system
//! let config = CryptoConfig::default();
//! let crypto = UnifiedCrypto::new(config)?;
//! let operations = CryptoOperations::new(crypto)?;
//!
//! // Perform high-level database encryption
//! let data = b"sensitive database record";
//! let encrypted_record = operations.database().encrypt_record(data, "user_table")?;
//!
//! // Perform authentication workflow
//! let auth_token = operations.auth().authenticate_user("user123", "password")?;
//!
//! // Create encrypted backup
//! let backup = operations.backup().create_encrypted_backup("/path/to/data")?;
//! # Ok(())
//! # }
//! ```

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

// Import operational modules
use super::database::DatabaseOperations;
use super::auth::AuthenticationOperations;
use super::network::NetworkSecurityOperations;
use super::backup::BackupOperations;
use super::cli::CliOperations;

/// Session information for tracking operational state
#[derive(Debug, Clone)]
pub struct OperationSession {
    /// Unique session identifier
    pub session_id: String,
    /// Session creation timestamp
    pub created_at: SystemTime,
    /// Last activity timestamp
    pub last_activity: SystemTime,
    /// Session expiration time
    pub expires_at: SystemTime,
    /// Associated user identifier (if applicable)
    pub user_id: Option<String>,
    /// Session privilege level
    pub privilege_level: PrivilegeLevel,
    /// Active operations count
    pub active_operations: u32,
}

/// Privilege levels for operational access control
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PrivilegeLevel {
    /// Read-only access (query operations only)
    ReadOnly,
    /// Standard user access (normal operations)
    Standard,
    /// Administrative access (management operations)
    Administrative,
    /// System-level access (emergency and maintenance operations)
    System,
}

/// Rate limiting configuration for abuse protection
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum operations per minute
    pub max_operations_per_minute: u32,
    /// Maximum failed operations before temporary lockout
    pub max_failed_operations: u32,
    /// Lockout duration for excessive failures
    pub lockout_duration: Duration,
    /// Rate limit reset interval
    pub reset_interval: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_operations_per_minute: 100,
            max_failed_operations: 10,
            lockout_duration: Duration::from_secs(300), // 5 minutes
            reset_interval: Duration::from_secs(60),    // 1 minute
        }
    }
}

/// Rate limiting state tracker
#[derive(Debug)]
struct RateLimitState {
    /// Operation count within current window
    operation_count: u32,
    /// Failed operation count
    failed_count: u32,
    /// Window start time
    window_start: SystemTime,
    /// Lockout end time (if in lockout)
    lockout_until: Option<SystemTime>,
}

/// Main operational cryptographic coordinator
///
/// This struct coordinates high-level cryptographic operations across all operational domains.
/// It provides unified access to database operations, authentication workflows, network security,
/// backup operations, and CLI integration while maintaining strict security boundaries.
pub struct CryptoOperations {
    /// Core unified crypto instance
    crypto: Arc<UnifiedCrypto>,
    /// Database operations module
    database_ops: Arc<DatabaseOperations>,
    /// Authentication operations module
    auth_ops: Arc<AuthenticationOperations>,
    /// Network security operations module
    network_ops: Arc<NetworkSecurityOperations>,
    /// Backup operations module
    backup_ops: Arc<BackupOperations>,
    /// CLI operations module
    cli_ops: Arc<CliOperations>,
    /// Active sessions for state tracking
    sessions: Arc<RwLock<HashMap<String, OperationSession>>>,
    /// Rate limiting configuration
    rate_limit_config: RateLimitConfig,
    /// Rate limiting state
    rate_limit_state: Arc<RwLock<HashMap<String, RateLimitState>>>,
    /// Audit logger for operational events
    audit_logger: Arc<CryptoAuditLogger>,
}

impl CryptoOperations {
    /// Create a new crypto operations coordinator
    ///
    /// # Arguments
    /// * `crypto` - Unified crypto instance for primitives access
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New operations coordinator or error
    ///
    /// # Security
    /// - Initializes all operational modules with proper security boundaries
    /// - Establishes audit logging for all operational activities
    /// - Configures rate limiting and abuse protection mechanisms
    pub fn new(crypto: UnifiedCrypto) -> UnifiedCryptoResult<Self> {
        let crypto_arc = Arc::new(crypto);
        let audit_logger = crypto_arc.audit_logger().clone();

        // Initialize operational modules
        let database_ops = Arc::new(DatabaseOperations::new(crypto_arc.clone())?);
        let auth_ops = Arc::new(AuthenticationOperations::new(crypto_arc.clone())?);
        let network_ops = Arc::new(NetworkSecurityOperations::new(crypto_arc.clone())?);
        let backup_ops = Arc::new(BackupOperations::new(crypto_arc.clone())?);
        let cli_ops = Arc::new(CliOperations::new(crypto_arc.clone())?);

        // Initialize session and rate limiting tracking
        let sessions = Arc::new(RwLock::new(HashMap::new()));
        let rate_limit_state = Arc::new(RwLock::new(HashMap::new()));
        let rate_limit_config = RateLimitConfig::default();

        let operations = Self {
            crypto: crypto_arc,
            database_ops,
            auth_ops,
            network_ops,
            backup_ops,
            cli_ops,
            sessions,
            rate_limit_config,
            rate_limit_state,
            audit_logger: audit_logger.clone(),
        };

        // Log operations initialization
        let logger = audit_logger.clone();
        logger.log_crypto_event(CryptoAuditEvent::operations_initialized())?;

        Ok(operations)
    }

    /// Get access to database operations
    ///
    /// # Returns
    /// * `&DatabaseOperations` - Reference to database operations module
    pub fn database(&self) -> &DatabaseOperations {
        &self.database_ops
    }

    /// Get access to authentication operations
    ///
    /// # Returns
    /// * `&AuthenticationOperations` - Reference to authentication operations module
    pub fn auth(&self) -> &AuthenticationOperations {
        &self.auth_ops
    }

    /// Get access to network security operations
    ///
    /// # Returns
    /// * `&NetworkSecurityOperations` - Reference to network security operations module
    pub fn network(&self) -> &NetworkSecurityOperations {
        &self.network_ops
    }

    /// Get access to backup operations
    ///
    /// # Returns
    /// * `&BackupOperations` - Reference to backup operations module
    pub fn backup(&self) -> &BackupOperations {
        &self.backup_ops
    }

    /// Get access to CLI operations
    ///
    /// # Returns
    /// * `&CliOperations` - Reference to CLI operations module
    pub fn cli(&self) -> &CliOperations {
        &self.cli_ops
    }

    /// Create a new operational session
    ///
    /// # Arguments
    /// * `user_id` - Optional user identifier for the session
    /// * `privilege_level` - Privilege level for the session
    /// * `duration` - Session duration before expiration
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<String>` - Session ID or error
    ///
    /// # Security
    /// - Generates cryptographically secure session identifier
    /// - Enforces session timeout and privilege level constraints
    /// - Logs session creation for audit trail
    pub fn create_session(
        &self,
        user_id: Option<String>,
        privilege_level: PrivilegeLevel,
        duration: Duration,
    ) -> UnifiedCryptoResult<String> {
        let now = SystemTime::now();
        let session_id = self.generate_session_id()?;
        
        let session = OperationSession {
            session_id: session_id.clone(),
            created_at: now,
            last_activity: now,
            expires_at: now + duration,
            user_id: user_id.clone(),
            privilege_level: privilege_level.clone(),
            active_operations: 0,
        };

        // Store session
        {
            let mut sessions = self.sessions.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire session lock".to_string(),
            })?;
            sessions.insert(session_id.clone(), session);
        }

        // Log session creation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::session_created(&session_id, &user_id, &privilege_level)
        )?;

        Ok(session_id)
    }

    /// Validate and refresh a session
    ///
    /// # Arguments
    /// * `session_id` - Session identifier to validate
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<OperationSession>` - Valid session or error
    ///
    /// # Security
    /// - Validates session existence and expiration
    /// - Updates last activity timestamp
    /// - Logs session validation for audit trail
    pub fn validate_session(&self, session_id: &str) -> UnifiedCryptoResult<OperationSession> {
        let now = SystemTime::now();
        
        let mut sessions = self.sessions.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire session lock".to_string(),
        })?;

        let session = sessions.get_mut(session_id).ok_or_else(|| UnifiedCryptoError::AuthenticationError {
            message: "Invalid session ID".to_string(),
        })?;

        // Check session expiration
        if now > session.expires_at {
            sessions.remove(session_id);
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Session expired".to_string(),
            });
        }

        // Update last activity
        session.last_activity = now;
        let session_copy = session.clone();

        Ok(session_copy)
    }

    /// Check rate limiting for a session or identifier
    ///
    /// # Arguments
    /// * `identifier` - Session ID or other identifier for rate limiting
    /// * `operation_type` - Type of operation for rate limiting
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<()>` - Success if within rate limits, error otherwise
    ///
    /// # Security
    /// - Enforces rate limits to prevent abuse
    /// - Implements temporary lockouts for excessive failures
    /// - Logs rate limit violations for audit trail
    pub fn check_rate_limit(&self, identifier: &str, operation_type: &str) -> UnifiedCryptoResult<()> {
        let now = SystemTime::now();
        
        let mut rate_limits = self.rate_limit_state.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire rate limit lock".to_string(),
        })?;

        let state = rate_limits.entry(identifier.to_string()).or_insert_with(|| RateLimitState {
            operation_count: 0,
            failed_count: 0,
            window_start: now,
            lockout_until: None,
        });

        // Check if currently in lockout
        if let Some(lockout_until) = state.lockout_until {
            if now < lockout_until {
                self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::rate_limit_violation(identifier, operation_type)
                )?;
                return Err(UnifiedCryptoError::RateLimitExceeded {
                    message: "Rate limit exceeded, temporarily locked out".to_string(),
                });
            } else {
                // Reset lockout
                state.lockout_until = None;
                state.failed_count = 0;
            }
        }

        // Check if window needs reset
        if now.duration_since(state.window_start).unwrap_or(Duration::ZERO) > self.rate_limit_config.reset_interval {
            state.operation_count = 0;
            state.window_start = now;
        }

        // Check rate limit
        if state.operation_count >= self.rate_limit_config.max_operations_per_minute {
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::rate_limit_violation(identifier, operation_type)
            )?;
            return Err(UnifiedCryptoError::RateLimitExceeded {
                message: "Rate limit exceeded".to_string(),
            });
        }

        // Increment operation count
        state.operation_count += 1;

        Ok(())
    }

    /// Record a failed operation for rate limiting
    ///
    /// # Arguments
    /// * `identifier` - Session ID or other identifier
    ///
    /// # Security
    /// - Tracks failed operations for abuse detection
    /// - Implements lockouts for excessive failures
    pub fn record_failed_operation(&self, identifier: &str) -> UnifiedCryptoResult<()> {
        let now = SystemTime::now();
        
        let mut rate_limits = self.rate_limit_state.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire rate limit lock".to_string(),
        })?;

        let state = rate_limits.entry(identifier.to_string()).or_insert_with(|| RateLimitState {
            operation_count: 0,
            failed_count: 0,
            window_start: now,
            lockout_until: None,
        });

        state.failed_count += 1;

        // Check if lockout threshold reached
        if state.failed_count >= self.rate_limit_config.max_failed_operations {
            state.lockout_until = Some(now + self.rate_limit_config.lockout_duration);
            
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::excessive_failures_lockout(identifier)
            )?;
        }

        Ok(())
    }

    /// Generate a cryptographically secure session ID
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<String>` - Secure session ID or error
    ///
    /// # Security
    /// - Uses cryptographically secure random number generation
    /// - Generates sufficient entropy for session security
    fn generate_session_id(&self) -> UnifiedCryptoResult<String> {
        use crate::unified_crypto::types::HashAlgorithm;
        
        // Generate 32 bytes of secure random data
        let random_bytes = self.crypto.primitives.generate_random_bytes(32)?;
        
        // Hash the random bytes to create a session ID
        let session_id_bytes = self.crypto.hash(&random_bytes, HashAlgorithm::Sha256)?;
        
        // Convert to hex string
        Ok(hex::encode(session_id_bytes))
    }

    /// Clean up expired sessions
    ///
    /// # Security
    /// - Removes expired sessions to prevent resource exhaustion
    /// - Logs session cleanup for audit trail
    pub fn cleanup_expired_sessions(&self) -> UnifiedCryptoResult<usize> {
        let now = SystemTime::now();
        let mut removed_count = 0;
        
        {
            let mut sessions = self.sessions.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire session lock".to_string(),
            })?;

            let expired_sessions: Vec<String> = sessions
                .iter()
                .filter(|(_, session)| now > session.expires_at)
                .map(|(id, _)| id.clone())
                .collect();

            for session_id in expired_sessions {
                sessions.remove(&session_id);
                removed_count += 1;
                
                // Log session expiration
                let _ = self.audit_logger.log_crypto_event(
                    CryptoAuditEvent::session_expired(&session_id)
                );
            }
        }

        Ok(removed_count)
    }
}

// Implement secure cleanup for CryptoOperations
impl Drop for CryptoOperations {
    fn drop(&mut self) {
        // Clear sensitive session data
        if let Ok(mut sessions) = self.sessions.write() {
            sessions.clear();
        }
        
        // Log operations shutdown (ignore errors during shutdown)
        let _ = self.audit_logger.log_crypto_event(
            CryptoAuditEvent::operations_shutdown()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::CryptoConfig;

    #[test]
    fn test_operations_initialization() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let operations = CryptoOperations::new(crypto);
        assert!(operations.is_ok());
    }

    #[test]
    fn test_session_creation_and_validation() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let operations = CryptoOperations::new(crypto).expect("Failed to create operations");
        
        let session_id = operations.create_session(
            Some("user123".to_string()),
            PrivilegeLevel::Standard,
            Duration::from_secs(3600),
        ).expect("Failed to create session");
        
        let session = operations.validate_session(&session_id).expect("Failed to validate session");
        assert_eq!(session.user_id, Some("user123".to_string()));
        assert_eq!(session.privilege_level, PrivilegeLevel::Standard);
    }

    #[test]
    fn test_rate_limiting() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let operations = CryptoOperations::new(crypto).expect("Failed to create operations");
        
        // First operation should succeed
        assert!(operations.check_rate_limit("test_user", "test_op").is_ok());
        
        // Record many failed operations
        for _ in 0..15 {
            let _ = operations.record_failed_operation("test_user");
        }
        
        // Next operation should fail due to lockout
        assert!(operations.check_rate_limit("test_user", "test_op").is_err());
    }
}