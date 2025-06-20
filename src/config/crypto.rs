//! Legacy crypto configuration - DEPRECATED
//!
//! This module is deprecated in favor of src/unified_crypto/config.rs
//! All new code should use the unified cryptographic configuration.

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::config::CryptoConfig instead")]
pub use crate::unified_crypto::config::CryptoConfig;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::config::EncryptionConfig instead")]
pub use crate::unified_crypto::config::EncryptionConfig as MasterKeyConfig;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::config::KeyConfig instead")]
pub use crate::unified_crypto::config::KeyConfig as KeyDerivationConfig;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::error::UnifiedCryptoError instead")]
pub use crate::unified_crypto::error::UnifiedCryptoError as ConfigError;

#[deprecated(since = "1.0.0", note = "Use crate::unified_crypto::error::UnifiedCryptoResult instead")]
pub use crate::unified_crypto::error::UnifiedCryptoResult as ConfigResult;
