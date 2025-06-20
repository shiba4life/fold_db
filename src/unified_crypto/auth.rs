//! # Authentication Cryptographic Operations
//!
//! This module provides high-level authentication operations built on top of the unified
//! cryptographic primitives. It handles multi-factor authentication, signature workflows,
//! session management, and certificate-based authentication with comprehensive audit logging.
//!
//! ## Features
//!
//! - **Multi-Factor Authentication**: Support for multiple authentication factors
//! - **Digital Signatures**: Message signing and verification workflows
//! - **Session Management**: Secure session creation and validation
//! - **Certificate Management**: X.509 certificate handling and validation
//! - **Token Authentication**: JWT and custom token support
//! - **Authentication Policies**: Configurable authentication requirements
//!
//! ## Example Usage
//!
//! ```rust
//! use datafold::unified_crypto::{UnifiedCrypto, CryptoOperations, CryptoConfig};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Initialize crypto operations
//! let config = CryptoConfig::default();
//! let crypto = UnifiedCrypto::new(config)?;
//! let operations = CryptoOperations::new(crypto)?;
//!
//! // Authenticate user with password
//! let auth_result = operations.auth().authenticate_password(
//!     "user123",
//!     "secure_password",
//! )?;
//!
//! // Create authentication token
//! let token = operations.auth().create_auth_token(&auth_result)?;
//! # Ok(())
//! # }
//! ```

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::types::{Signature, SignatureAlgorithm, KeyId, HashAlgorithm};
use crate::unified_crypto::{KeyPair, PrivateKeyHandle, PublicKeyHandle};
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use serde::{Serialize, Deserialize};

/// Authentication factor types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthenticationFactor {
    /// Password-based authentication
    Password(String),
    /// Public key authentication
    PublicKey(PublicKeyHandle),
    /// Biometric authentication (placeholder)
    Biometric(BiometricData),
    /// Hardware token authentication
    HardwareToken(String),
    /// One-time password (TOTP/HOTP)
    OneTimePassword(String),
}

/// Biometric authentication data (placeholder)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BiometricData {
    /// Biometric type (fingerprint, face, etc.)
    pub biometric_type: String,
    /// Encoded biometric template
    pub template: Vec<u8>,
}

/// Authentication result containing user identity and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationResult {
    /// Authenticated user identifier
    pub user_id: String,
    /// Authentication timestamp
    pub timestamp: SystemTime,
    /// Authentication factors used
    pub factors_used: Vec<String>,
    /// Authentication strength level
    pub strength_level: AuthenticationStrength,
    /// Session identifier
    pub session_id: String,
    /// Expiration time
    pub expires_at: SystemTime,
}

/// Authentication strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AuthenticationStrength {
    /// Single factor authentication
    SingleFactor,
    /// Two-factor authentication
    TwoFactor,
    /// Multi-factor authentication (3+ factors)
    MultiFactor,
    /// High-assurance authentication
    HighAssurance,
}

/// Authentication policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationPolicy {
    /// Required minimum authentication strength
    pub minimum_strength: AuthenticationStrength,
    /// Session timeout duration
    pub session_timeout: Duration,
    /// Maximum concurrent sessions per user
    pub max_concurrent_sessions: u32,
    /// Require re-authentication for sensitive operations
    pub require_reauth_for_sensitive: bool,
    /// Password complexity requirements
    pub password_policy: PasswordPolicy,
    /// Enable rate limiting for authentication attempts
    pub enable_rate_limiting: bool,
    /// Maximum failed attempts before lockout
    pub max_failed_attempts: u32,
    /// Lockout duration
    pub lockout_duration: Duration,
}

/// Password policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    /// Minimum password length
    pub min_length: usize,
    /// Require uppercase letters
    pub require_uppercase: bool,
    /// Require lowercase letters
    pub require_lowercase: bool,
    /// Require digits
    pub require_digits: bool,
    /// Require special characters
    pub require_special: bool,
    /// Password history size (prevent reuse)
    pub history_size: usize,
}

impl Default for AuthenticationPolicy {
    fn default() -> Self {
        Self {
            minimum_strength: AuthenticationStrength::SingleFactor,
            session_timeout: Duration::from_secs(3600), // 1 hour
            max_concurrent_sessions: 5,
            require_reauth_for_sensitive: true,
            password_policy: PasswordPolicy::default(),
            enable_rate_limiting: true,
            max_failed_attempts: 5,
            lockout_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
            history_size: 5,
        }
    }
}

/// Authentication token for session management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationToken {
    /// Token value
    pub token: String,
    /// User identifier
    pub user_id: String,
    /// Token creation timestamp
    pub issued_at: SystemTime,
    /// Token expiration
    pub expires_at: SystemTime,
    /// Token permissions/scopes
    pub scopes: Vec<String>,
    /// Digital signature for token integrity
    pub signature: Signature,
}

/// Failed authentication attempt tracking
#[derive(Debug, Clone)]
struct FailedAttemptTracker {
    /// Number of failed attempts
    attempt_count: u32,
    /// First failed attempt timestamp
    first_attempt: SystemTime,
    /// Last failed attempt timestamp
    last_attempt: SystemTime,
    /// Lockout end time (if locked out)
    lockout_until: Option<SystemTime>,
}

/// Authentication operations coordinator
///
/// This struct provides high-level authentication operations including multi-factor
/// authentication, session management, and secure token handling.
pub struct AuthenticationOperations {
    /// Reference to the unified crypto system
    crypto: Arc<UnifiedCrypto>,
    /// Authentication policy
    policy: AuthenticationPolicy,
    /// Active sessions tracking
    active_sessions: Arc<std::sync::RwLock<HashMap<String, AuthenticationResult>>>,
    /// Failed attempt tracking
    failed_attempts: Arc<std::sync::RwLock<HashMap<String, FailedAttemptTracker>>>,
    /// Audit logger for authentication operations
    audit_logger: Arc<CryptoAuditLogger>,
    /// Signing key for authentication tokens
    token_signing_key: Arc<PrivateKeyHandle>,
}

impl AuthenticationOperations {
    /// Create new authentication operations coordinator
    ///
    /// # Arguments
    /// * `crypto` - Unified crypto system reference
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New authentication operations coordinator or error
    ///
    /// # Security
    /// - Generates dedicated signing key for authentication tokens
    /// - Initializes secure session tracking
    /// - Establishes comprehensive audit logging
    pub fn new(crypto: Arc<UnifiedCrypto>) -> UnifiedCryptoResult<Self> {
        let policy = AuthenticationPolicy::default();
        let active_sessions = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let failed_attempts = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let audit_logger = crypto.audit_logger().clone();

        // Generate dedicated signing key for authentication tokens
        let token_keypair = crypto.generate_keypair()?;
        let token_signing_key = Arc::new(token_keypair.private_key);

        // Log authentication operations initialization
        audit_logger.log_crypto_event(CryptoAuditEvent::auth_operations_initialized())?;

        Ok(Self {
            crypto,
            policy,
            active_sessions,
            failed_attempts,
            audit_logger: audit_logger.clone(),
            token_signing_key,
        })
    }

    /// Authenticate user with password
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `password` - User password
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<AuthenticationResult>` - Authentication result or error
    ///
    /// # Security
    /// - Validates password against policy requirements
    /// - Implements rate limiting and lockout protection
    /// - Logs all authentication attempts for audit trail
    pub fn authenticate_password(&self, user_id: &str, password: &str) -> UnifiedCryptoResult<AuthenticationResult> {
        // Check rate limiting and lockouts
        self.check_authentication_rate_limit(user_id)?;

        // Log authentication attempt
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_attempt_start(user_id, "password")
        )?;

        // Validate password policy compliance
        self.validate_password_policy(password)?;

        // In a real implementation, verify password against stored hash
        // For this example, we'll simulate password verification
        let password_valid = self.verify_password_hash(user_id, password)?;

        if !password_valid {
            self.record_failed_authentication(user_id)?;
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::auth_attempt_failed(user_id, "password", "invalid_password")
            )?;
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Invalid password".to_string(),
            });
        }

        // Clear failed attempts on successful authentication
        self.clear_failed_attempts(user_id)?;

        // Create authentication result
        let auth_result = self.create_authentication_result(
            user_id,
            vec!["password".to_string()],
            AuthenticationStrength::SingleFactor,
        )?;

        // Log successful authentication
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_attempt_success(user_id, "password")
        )?;

        Ok(auth_result)
    }

    /// Authenticate user with public key
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `message` - Message to verify signature against
    /// * `signature` - Digital signature from user
    /// * `public_key` - User's public key
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<AuthenticationResult>` - Authentication result or error
    pub fn authenticate_public_key(
        &self,
        user_id: &str,
        message: &[u8],
        signature: &Signature,
        public_key: &PublicKeyHandle,
    ) -> UnifiedCryptoResult<AuthenticationResult> {
        // Check rate limiting
        self.check_authentication_rate_limit(user_id)?;

        // Log authentication attempt
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_attempt_start(user_id, "public_key")
        )?;

        // Verify signature
        let signature_valid = self.crypto.verify(message, signature, public_key)?;

        if !signature_valid {
            self.record_failed_authentication(user_id)?;
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::auth_attempt_failed(user_id, "public_key", "invalid_signature")
            )?;
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Invalid signature".to_string(),
            });
        }

        // Clear failed attempts on successful authentication
        self.clear_failed_attempts(user_id)?;

        // Create authentication result
        let auth_result = self.create_authentication_result(
            user_id,
            vec!["public_key".to_string()],
            AuthenticationStrength::SingleFactor,
        )?;

        // Log successful authentication
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_attempt_success(user_id, "public_key")
        )?;

        Ok(auth_result)
    }

    /// Perform multi-factor authentication
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `factors` - Authentication factors to verify
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<AuthenticationResult>` - Authentication result or error
    pub fn authenticate_multi_factor(
        &self,
        user_id: &str,
        factors: Vec<AuthenticationFactor>,
    ) -> UnifiedCryptoResult<AuthenticationResult> {
        if factors.is_empty() {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "At least one authentication factor required".to_string(),
            });
        }

        // Check rate limiting
        self.check_authentication_rate_limit(user_id)?;

        // Log MFA attempt
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_mfa_attempt_start(user_id, factors.len())
        )?;

        let mut verified_factors = Vec::new();
        let mut all_factors_valid = true;

        // Verify each factor
        for factor in factors {
            match self.verify_authentication_factor(user_id, &factor) {
                Ok(factor_name) => {
                    verified_factors.push(factor_name);
                }
                Err(_) => {
                    all_factors_valid = false;
                    break;
                }
            }
        }

        if !all_factors_valid {
            self.record_failed_authentication(user_id)?;
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::auth_mfa_attempt_failed(user_id, &verified_factors)
            )?;
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Multi-factor authentication failed".to_string(),
            });
        }

        // Determine authentication strength
        let strength = match verified_factors.len() {
            1 => AuthenticationStrength::SingleFactor,
            2 => AuthenticationStrength::TwoFactor,
            3.. => AuthenticationStrength::MultiFactor,
            _ => AuthenticationStrength::SingleFactor,
        };

        // Check if strength meets policy requirements
        if strength < self.policy.minimum_strength {
            return Err(UnifiedCryptoError::AuthenticationError {
                message: format!("Authentication strength {:?} does not meet minimum requirement {:?}", 
                    strength, self.policy.minimum_strength),
            });
        }

        // Clear failed attempts on successful authentication
        self.clear_failed_attempts(user_id)?;

        // Create authentication result
        let auth_result = self.create_authentication_result(user_id, verified_factors, strength)?;

        // Log successful MFA
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_mfa_attempt_success(user_id, &auth_result.factors_used)
        )?;

        Ok(auth_result)
    }

    /// Create an authentication token from authentication result
    ///
    /// # Arguments
    /// * `auth_result` - Authentication result to create token from
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<AuthenticationToken>` - Authentication token or error
    pub fn create_auth_token(&self, auth_result: &AuthenticationResult) -> UnifiedCryptoResult<AuthenticationToken> {
        // Create token payload
        let token_data = format!("{}:{}:{}", 
            auth_result.user_id, 
            auth_result.timestamp.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs(),
            auth_result.session_id
        );

        // Sign the token
        let signature = self.crypto.sign(token_data.as_bytes(), &self.token_signing_key)?;

        let token = AuthenticationToken {
            token: token_data,
            user_id: auth_result.user_id.clone(),
            issued_at: auth_result.timestamp,
            expires_at: auth_result.expires_at,
            scopes: vec!["default".to_string()], // In production, determine from user roles
            signature,
        };

        // Log token creation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_token_created(&auth_result.user_id, &auth_result.session_id)
        )?;

        Ok(token)
    }

    /// Validate an authentication token
    ///
    /// # Arguments
    /// * `token` - Authentication token to validate
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<AuthenticationResult>` - Valid authentication result or error
    pub fn validate_auth_token(&self, token: &AuthenticationToken) -> UnifiedCryptoResult<AuthenticationResult> {
        // Check token expiration
        if SystemTime::now() > token.expires_at {
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Token expired".to_string(),
            });
        }

        // Verify token signature
        let token_signing_public_key = self.crypto.key_manager()
            .get_keypair(self.token_signing_key.id())?
            .public_key;
        
        let signature_valid = self.crypto.verify(
            token.token.as_bytes(),
            &token.signature,
            &token_signing_public_key,
        )?;

        if !signature_valid {
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Invalid token signature".to_string(),
            });
        }

        // Look up active session
        let sessions = self.active_sessions.read().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire sessions lock".to_string(),
        })?;

        // In a real implementation, extract session ID from token
        let session_id = "session_placeholder"; // Extract from token
        
        if let Some(auth_result) = sessions.get(session_id) {
            Ok(auth_result.clone())
        } else {
            Err(UnifiedCryptoError::AuthenticationError {
                message: "Session not found".to_string(),
            })
        }
    }

    /// Get current authentication policy
    pub fn policy(&self) -> &AuthenticationPolicy {
        &self.policy
    }

    /// Update authentication policy
    pub fn update_policy(&mut self, new_policy: AuthenticationPolicy) -> UnifiedCryptoResult<()> {
        // Log policy update
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::auth_policy_updated(&self.policy, &new_policy)
        )?;

        self.policy = new_policy;
        Ok(())
    }

    /// Check authentication rate limiting for a user
    fn check_authentication_rate_limit(&self, user_id: &str) -> UnifiedCryptoResult<()> {
        if !self.policy.enable_rate_limiting {
            return Ok(());
        }

        let now = SystemTime::now();
        let failed_attempts = self.failed_attempts.read().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire failed attempts lock".to_string(),
        })?;

        if let Some(tracker) = failed_attempts.get(user_id) {
            // Check if currently locked out
            if let Some(lockout_until) = tracker.lockout_until {
                if now < lockout_until {
                    return Err(UnifiedCryptoError::RateLimitExceeded {
                        message: "Account temporarily locked due to excessive failed attempts".to_string(),
                    });
                }
            }

            // Check if too many attempts in recent period
            if tracker.attempt_count >= self.policy.max_failed_attempts {
                let time_since_first = now.duration_since(tracker.first_attempt)
                    .unwrap_or(Duration::ZERO);
                
                if time_since_first < self.policy.lockout_duration {
                    return Err(UnifiedCryptoError::RateLimitExceeded {
                        message: "Too many failed authentication attempts".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Record a failed authentication attempt
    fn record_failed_authentication(&self, user_id: &str) -> UnifiedCryptoResult<()> {
        let now = SystemTime::now();
        let mut failed_attempts = self.failed_attempts.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire failed attempts lock".to_string(),
        })?;

        let tracker = failed_attempts.entry(user_id.to_string()).or_insert_with(|| FailedAttemptTracker {
            attempt_count: 0,
            first_attempt: now,
            last_attempt: now,
            lockout_until: None,
        });

        tracker.attempt_count += 1;
        tracker.last_attempt = now;

        // Set lockout if threshold reached
        if tracker.attempt_count >= self.policy.max_failed_attempts {
            tracker.lockout_until = Some(now + self.policy.lockout_duration);
        }

        Ok(())
    }

    /// Clear failed authentication attempts for a user
    fn clear_failed_attempts(&self, user_id: &str) -> UnifiedCryptoResult<()> {
        let mut failed_attempts = self.failed_attempts.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire failed attempts lock".to_string(),
        })?;

        failed_attempts.remove(user_id);
        Ok(())
    }

    /// Validate password against policy requirements
    fn validate_password_policy(&self, password: &str) -> UnifiedCryptoResult<()> {
        let policy = &self.policy.password_policy;

        if password.len() < policy.min_length {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Password must be at least {} characters", policy.min_length),
            });
        }

        if policy.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Password must contain uppercase letters".to_string(),
            });
        }

        if policy.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Password must contain lowercase letters".to_string(),
            });
        }

        if policy.require_digits && !password.chars().any(|c| c.is_ascii_digit()) {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Password must contain digits".to_string(),
            });
        }

        if policy.require_special && !password.chars().any(|c| !c.is_alphanumeric()) {
            return Err(UnifiedCryptoError::InvalidInput {
                message: "Password must contain special characters".to_string(),
            });
        }

        Ok(())
    }

    /// Verify password hash (placeholder implementation)
    fn verify_password_hash(&self, user_id: &str, password: &str) -> UnifiedCryptoResult<bool> {
        // In a real implementation, this would:
        // 1. Retrieve stored password hash for user
        // 2. Use appropriate key derivation function (Argon2, etc.)
        // 3. Compare hashes in constant time
        
        // For this example, simulate password verification
        Ok(!password.is_empty() && !user_id.is_empty())
    }

    /// Verify an individual authentication factor
    fn verify_authentication_factor(&self, user_id: &str, factor: &AuthenticationFactor) -> UnifiedCryptoResult<String> {
        match factor {
            AuthenticationFactor::Password(password) => {
                if self.verify_password_hash(user_id, password)? {
                    Ok("password".to_string())
                } else {
                    Err(UnifiedCryptoError::AuthenticationError {
                        message: "Invalid password".to_string(),
                    })
                }
            }
            AuthenticationFactor::PublicKey(_) => {
                // In production, verify public key signature
                Ok("public_key".to_string())
            }
            AuthenticationFactor::Biometric(_) => {
                // In production, verify biometric template
                Ok("biometric".to_string())
            }
            AuthenticationFactor::HardwareToken(_) => {
                // In production, verify hardware token
                Ok("hardware_token".to_string())
            }
            AuthenticationFactor::OneTimePassword(_) => {
                // In production, verify TOTP/HOTP
                Ok("otp".to_string())
            }
        }
    }

    /// Create authentication result with session management
    fn create_authentication_result(
        &self,
        user_id: &str,
        factors_used: Vec<String>,
        strength: AuthenticationStrength,
    ) -> UnifiedCryptoResult<AuthenticationResult> {
        let now = SystemTime::now();
        let session_id = self.generate_session_id()?;
        
        let auth_result = AuthenticationResult {
            user_id: user_id.to_string(),
            timestamp: now,
            factors_used,
            strength_level: strength,
            session_id: session_id.clone(),
            expires_at: now + self.policy.session_timeout,
        };

        // Store in active sessions
        {
            let mut sessions = self.active_sessions.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire sessions lock".to_string(),
            })?;
            sessions.insert(session_id, auth_result.clone());
        }

        Ok(auth_result)
    }

    /// Generate secure session ID
    fn generate_session_id(&self) -> UnifiedCryptoResult<String> {
        // Generate random session ID using crypto primitives
        let random_bytes = self.crypto.primitives.generate_random_bytes(32)?;
        let session_id_hash = self.crypto.hash(&random_bytes, HashAlgorithm::Sha256)?;
        Ok(hex::encode(session_id_hash))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::CryptoConfig;

    #[test]
    fn test_auth_operations_initialization() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let crypto_arc = Arc::new(crypto);
        let auth_ops = AuthenticationOperations::new(crypto_arc);
        assert!(auth_ops.is_ok());
    }

    #[test]
    fn test_password_policy_validation() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let crypto_arc = Arc::new(crypto);
        let auth_ops = AuthenticationOperations::new(crypto_arc).expect("Failed to create auth ops");

        // Valid password
        let result = auth_ops.validate_password_policy("StrongPass123!");
        assert!(result.is_ok());

        // Too short
        let result = auth_ops.validate_password_policy("short");
        assert!(result.is_err());

        // Missing uppercase
        let result = auth_ops.validate_password_policy("weakpass123!");
        assert!(result.is_err());
    }

    #[test]
    fn test_authentication_strength_ordering() {
        assert!(AuthenticationStrength::SingleFactor < AuthenticationStrength::TwoFactor);
        assert!(AuthenticationStrength::TwoFactor < AuthenticationStrength::MultiFactor);
        assert!(AuthenticationStrength::MultiFactor < AuthenticationStrength::HighAssurance);
    }
}