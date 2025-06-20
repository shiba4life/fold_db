//! # CLI Cryptographic Operations
//!
//! This module provides CLI-specific cryptographic operations that integrate with the
//! existing CLI infrastructure while leveraging the unified cryptographic system.

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::auth::{AuthenticationOperations, AuthenticationResult};
use crate::unified_crypto::keys::KeyPair;
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::Arc;

/// CLI operations coordinator for cryptographic commands
pub struct CliOperations {
    /// Reference to the unified crypto system
    crypto: Arc<UnifiedCrypto>,
    /// Authentication operations for CLI
    auth_ops: Arc<AuthenticationOperations>,
    /// Audit logger for CLI operations
    audit_logger: Arc<CryptoAuditLogger>,
}

impl CliOperations {
    /// Create new CLI operations coordinator
    pub fn new(crypto: Arc<UnifiedCrypto>) -> UnifiedCryptoResult<Self> {
        let auth_ops = Arc::new(AuthenticationOperations::new(crypto.clone())?);
        let audit_logger = crypto.audit_logger().clone();

        audit_logger.log_crypto_event(CryptoAuditEvent::cli_operations_initialized())?;

        Ok(Self {
            crypto,
            auth_ops,
            audit_logger: audit_logger,
        })
    }

    /// Generate a new keypair for CLI usage
    pub fn generate_cli_keypair(&self) -> UnifiedCryptoResult<KeyPair> {
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::cli_keypair_generation_start()
        )?;

        let keypair = self.crypto.generate_keypair()?;

        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::cli_keypair_generation_success(&keypair.public_key.id())
        )?;

        Ok(keypair)
    }

    /// Authenticate CLI user
    pub fn authenticate_cli_user(&self, user_id: &str, password: &str) -> UnifiedCryptoResult<AuthenticationResult> {
        self.auth_ops.authenticate_password(user_id, password)
    }

    /// Get access to authentication operations
    pub fn auth(&self) -> &AuthenticationOperations {
        &self.auth_ops
    }
}