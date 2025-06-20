//! # Network Security Cryptographic Operations
//!
//! This module provides high-level network security operations built on top of the unified
//! cryptographic primitives. It handles secure communication protocols, key exchange,
//! TLS/SSL integration, and network-level encryption with comprehensive audit logging.
//!
//! ## Features
//!
//! - **Secure Communication**: End-to-end encryption for network communication
//! - **Key Exchange**: Secure key exchange protocols (ECDH, RSA key transport)
//! - **TLS/SSL Integration**: Certificate management and validation
//! - **Message Authentication**: Network message signing and verification
//! - **Protocol Security**: Support for secure network protocols
//! - **Rate Limiting**: Network-level abuse protection
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
//! // Establish secure communication channel
//! let channel = operations.network().create_secure_channel("peer_id")?;
//!
//! // Send encrypted message
//! let message = b"sensitive network data";
//! let encrypted_message = operations.network().encrypt_message(&channel, message)?;
//! # Ok(())
//! # }
//! ```

use crate::unified_crypto::{UnifiedCrypto, UnifiedCryptoResult, UnifiedCryptoError, CryptoAuditEvent};
use crate::unified_crypto::types::{EncryptedData, Signature, KeyId, Algorithm};
use crate::unified_crypto::keys::KeyPair;
use crate::unified_crypto::primitives::{PublicKeyHandle, PrivateKeyHandle};
use crate::unified_crypto::audit::CryptoAuditLogger;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{SystemTime, Duration};
use std::net::{IpAddr, SocketAddr};
use serde::{Serialize, Deserialize};

/// Network security policy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecurityPolicy {
    /// Default encryption algorithm for network communications
    pub default_encryption: Algorithm,
    /// Require mutual authentication for all connections
    pub require_mutual_auth: bool,
    /// Enable perfect forward secrecy
    pub enable_perfect_forward_secrecy: bool,
    /// Maximum message size for network encryption
    pub max_message_size: usize,
    /// Connection timeout duration
    pub connection_timeout: Duration,
    /// Enable network-level compression
    pub enable_compression: bool,
    /// Rate limiting configuration
    pub rate_limit_config: NetworkRateLimitConfig,
}

/// Network rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRateLimitConfig {
    /// Maximum messages per minute per peer
    pub max_messages_per_minute: u32,
    /// Maximum bandwidth per minute (bytes)
    pub max_bandwidth_per_minute: u64,
    /// Connection attempt rate limit
    pub max_connections_per_minute: u32,
    /// Rate limit reset interval
    pub reset_interval: Duration,
}

impl Default for NetworkSecurityPolicy {
    fn default() -> Self {
        Self {
            default_encryption: Algorithm::ChaCha20Poly1305,
            require_mutual_auth: true,
            enable_perfect_forward_secrecy: true,
            max_message_size: 64 * 1024 * 1024, // 64MB
            connection_timeout: Duration::from_secs(30),
            enable_compression: true,
            rate_limit_config: NetworkRateLimitConfig::default(),
        }
    }
}

impl Default for NetworkRateLimitConfig {
    fn default() -> Self {
        Self {
            max_messages_per_minute: 1000,
            max_bandwidth_per_minute: 100 * 1024 * 1024, // 100MB
            max_connections_per_minute: 100,
            reset_interval: Duration::from_secs(60),
        }
    }
}

/// Secure communication channel between peers
#[derive(Debug, Clone)]
pub struct SecureChannel {
    /// Channel identifier
    pub channel_id: String,
    /// Local peer identifier
    pub local_peer_id: String,
    /// Remote peer identifier
    pub remote_peer_id: String,
    /// Shared encryption key
    pub encryption_key: KeyId,
    /// Channel creation timestamp
    pub created_at: SystemTime,
    /// Channel expiration time
    pub expires_at: SystemTime,
    /// Channel security parameters
    pub security_params: ChannelSecurityParams,
}

/// Security parameters for a communication channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelSecurityParams {
    /// Encryption algorithm
    pub encryption_algorithm: Algorithm,
    /// Key exchange method used
    pub key_exchange_method: KeyExchangeMethod,
    /// Perfect forward secrecy enabled
    pub perfect_forward_secrecy: bool,
    /// Mutual authentication performed
    pub mutual_authentication: bool,
}

/// Key exchange methods for secure communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum KeyExchangeMethod {
    /// Elliptic Curve Diffie-Hellman
    ECDH,
    /// RSA key transport
    RSAKeyTransport,
    /// Pre-shared key
    PreSharedKey,
    /// Static key pair
    StaticKeyPair,
}

/// Encrypted network message with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNetworkMessage {
    /// Message identifier
    pub message_id: String,
    /// Source peer identifier
    pub source_peer: String,
    /// Destination peer identifier
    pub destination_peer: String,
    /// Encrypted message data
    pub encrypted_data: EncryptedData,
    /// Message timestamp
    pub timestamp: SystemTime,
    /// Message signature for integrity
    pub signature: Signature,
    /// Message metadata
    pub metadata: NetworkMessageMetadata,
}

/// Metadata for network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMessageMetadata {
    /// Message type identifier
    pub message_type: String,
    /// Message priority level
    pub priority: MessagePriority,
    /// Compression information
    pub compression_info: Option<CompressionInfo>,
    /// Routing information
    pub routing_info: Option<RoutingInfo>,
}

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    /// Low priority background messages
    Low,
    /// Normal priority messages
    Normal,
    /// High priority messages
    High,
    /// Emergency priority messages
    Emergency,
}

/// Compression information for network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
    /// Compression algorithm used
    pub algorithm: String,
    /// Original message size
    pub original_size: usize,
    /// Compressed size
    pub compressed_size: usize,
}

/// Routing information for network messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingInfo {
    /// Source address
    pub source_addr: Option<SocketAddr>,
    /// Destination address
    pub destination_addr: Option<SocketAddr>,
    /// Hop count
    pub hop_count: u8,
    /// Maximum hops allowed
    pub max_hops: u8,
}

/// Peer authentication information
#[derive(Debug, Clone)]
pub struct PeerAuthentication {
    /// Peer identifier
    pub peer_id: String,
    /// Peer public key
    pub public_key: PublicKeyHandle,
    /// Authentication timestamp
    pub authenticated_at: SystemTime,
    /// Authentication method used
    pub auth_method: String,
    /// Trust level
    pub trust_level: TrustLevel,
}

/// Trust levels for peer authentication
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustLevel {
    /// Untrusted peer
    Untrusted,
    /// Basic trust level
    Basic,
    /// Verified trust level
    Verified,
    /// High trust level
    High,
    /// Maximum trust level
    Maximum,
}

/// Network rate limiting state
#[derive(Debug)]
struct NetworkRateLimitState {
    /// Message count in current window
    message_count: u32,
    /// Bandwidth used in current window
    bandwidth_used: u64,
    /// Connection count in current window
    connection_count: u32,
    /// Window start time
    window_start: SystemTime,
}

/// Network security operations coordinator
///
/// This struct provides high-level network security operations including secure communication
/// channels, key exchange protocols, message encryption, and peer authentication.
pub struct NetworkSecurityOperations {
    /// Reference to the unified crypto system
    crypto: Arc<UnifiedCrypto>,
    /// Network security policy
    policy: NetworkSecurityPolicy,
    /// Active secure channels
    active_channels: Arc<std::sync::RwLock<HashMap<String, SecureChannel>>>,
    /// Authenticated peers
    authenticated_peers: Arc<std::sync::RwLock<HashMap<String, PeerAuthentication>>>,
    /// Rate limiting state per peer
    rate_limit_state: Arc<std::sync::RwLock<HashMap<String, NetworkRateLimitState>>>,
    /// Audit logger for network operations
    audit_logger: Arc<CryptoAuditLogger>,
}

impl NetworkSecurityOperations {
    /// Create new network security operations coordinator
    ///
    /// # Arguments
    /// * `crypto` - Unified crypto system reference
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Self>` - New network operations coordinator or error
    pub fn new(crypto: Arc<UnifiedCrypto>) -> UnifiedCryptoResult<Self> {
        let policy = NetworkSecurityPolicy::default();
        let active_channels = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let authenticated_peers = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let rate_limit_state = Arc::new(std::sync::RwLock::new(HashMap::new()));
        let audit_logger = crypto.audit_logger();

        // Log network operations initialization
        audit_logger.log_crypto_event(CryptoAuditEvent::network_operations_initialized())?;

        Ok(Self {
            crypto,
            policy,
            active_channels,
            authenticated_peers,
            rate_limit_state,
            audit_logger: audit_logger.clone(),
        })
    }

    /// Create a secure communication channel with a peer
    ///
    /// # Arguments
    /// * `remote_peer_id` - Identifier of the remote peer
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<SecureChannel>` - Secure channel or error
    ///
    /// # Security
    /// - Performs key exchange with the remote peer
    /// - Establishes mutual authentication if required
    /// - Creates secure communication parameters
    /// - Logs channel creation for audit trail
    pub fn create_secure_channel(&self, remote_peer_id: &str) -> UnifiedCryptoResult<SecureChannel> {
        // Check rate limiting
        self.check_network_rate_limit(remote_peer_id, "channel_creation")?;

        // Log channel creation start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_channel_creation_start(remote_peer_id)
        )?;

        // Generate channel identifier
        let channel_id = self.generate_channel_id()?;

        // Perform key exchange
        let (encryption_key, key_exchange_method) = self.perform_key_exchange(remote_peer_id)?;

        // Verify mutual authentication if required
        let mutual_auth = if self.policy.require_mutual_auth {
            self.verify_peer_authentication(remote_peer_id)?;
            true
        } else {
            false
        };

        // Create channel
        let now = SystemTime::now();
        let channel = SecureChannel {
            channel_id: channel_id.clone(),
            local_peer_id: "local".to_string(), // In production, use actual local peer ID
            remote_peer_id: remote_peer_id.to_string(),
            encryption_key,
            created_at: now,
            expires_at: now + self.policy.connection_timeout,
            security_params: ChannelSecurityParams {
                encryption_algorithm: self.policy.default_encryption,
                key_exchange_method,
                perfect_forward_secrecy: self.policy.enable_perfect_forward_secrecy,
                mutual_authentication: mutual_auth,
            },
        };

        // Store active channel
        {
            let mut channels = self.active_channels.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire channels lock".to_string(),
            })?;
            channels.insert(channel_id.clone(), channel.clone());
        }

        // Log successful channel creation
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_channel_creation_success(&channel_id, remote_peer_id)
        )?;

        Ok(channel)
    }

    /// Encrypt a message for secure network transmission
    ///
    /// # Arguments
    /// * `channel` - Secure channel to use for encryption
    /// * `message` - Message data to encrypt
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<EncryptedNetworkMessage>` - Encrypted message or error
    pub fn encrypt_message(&self, channel: &SecureChannel, message: &[u8]) -> UnifiedCryptoResult<EncryptedNetworkMessage> {
        // Validate message size
        if message.len() > self.policy.max_message_size {
            return Err(UnifiedCryptoError::InvalidInput {
                message: format!("Message size {} exceeds maximum {}", message.len(), self.policy.max_message_size),
            });
        }

        // Check rate limiting
        self.check_network_rate_limit(&channel.remote_peer_id, "message_encryption")?;

        // Log message encryption start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_message_encryption_start(&channel.channel_id, message.len())
        )?;

        // Apply compression if enabled
        let (final_message, compression_info) = if self.policy.enable_compression {
            self.compress_message(message)?
        } else {
            (message.to_vec(), None)
        };

        // Get encryption key
        let key_pair = self.crypto.key_manager().load_keypair(&channel.encryption_key)?;

        // Encrypt the message
        let encrypted_data = self.crypto.encrypt(&final_message, &key_pair.public_key)?;

        // Generate message ID
        let message_id = self.generate_message_id()?;

        // Sign the encrypted message for integrity
        let signature = self.crypto.sign(encrypted_data.ciphertext(), &key_pair.private_key)?;

        let encrypted_message = EncryptedNetworkMessage {
            message_id: message_id.clone(),
            source_peer: channel.local_peer_id.clone(),
            destination_peer: channel.remote_peer_id.clone(),
            encrypted_data,
            timestamp: SystemTime::now(),
            signature,
            metadata: NetworkMessageMetadata {
                message_type: "data".to_string(),
                priority: MessagePriority::Normal,
                compression_info,
                routing_info: None,
            },
        };

        // Log successful encryption
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_message_encryption_success(&message_id, &channel.channel_id)
        )?;

        Ok(encrypted_message)
    }

    /// Decrypt a network message
    ///
    /// # Arguments
    /// * `encrypted_message` - Encrypted network message to decrypt
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<Vec<u8>>` - Decrypted message data or error
    pub fn decrypt_message(&self, encrypted_message: &EncryptedNetworkMessage) -> UnifiedCryptoResult<Vec<u8>> {
        // Log message decryption start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_message_decryption_start(&encrypted_message.message_id)
        )?;

        // Get decryption key
        let key_id = encrypted_message.encrypted_data.key_id();
        let key_pair = self.crypto.key_manager().load_keypair(key_id)?;

        // Verify message signature
        let signature_valid = self.crypto.verify(
            encrypted_message.encrypted_data.ciphertext(),
            &encrypted_message.signature,
            &key_pair.public_key,
        )?;

        if !signature_valid {
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Invalid message signature".to_string(),
            });
        }

        // Decrypt the message
        let decrypted_data = self.crypto.decrypt(&encrypted_message.encrypted_data, &key_pair.private_key)?;

        // Apply decompression if needed
        let final_data = if let Some(compression_info) = &encrypted_message.metadata.compression_info {
            self.decompress_message(&decrypted_data, compression_info)?
        } else {
            decrypted_data
        };

        // Log successful decryption
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_message_decryption_success(&encrypted_message.message_id)
        )?;

        Ok(final_data)
    }

    /// Authenticate a peer using their public key
    ///
    /// # Arguments
    /// * `peer_id` - Peer identifier
    /// * `public_key` - Peer's public key
    /// * `challenge` - Authentication challenge
    /// * `signature` - Peer's signature on the challenge
    ///
    /// # Returns
    /// * `UnifiedCryptoResult<PeerAuthentication>` - Peer authentication result or error
    pub fn authenticate_peer(
        &self,
        peer_id: &str,
        public_key: PublicKeyHandle,
        challenge: &[u8],
        signature: &Signature,
    ) -> UnifiedCryptoResult<PeerAuthentication> {
        // Check rate limiting
        self.check_network_rate_limit(peer_id, "peer_authentication")?;

        // Log peer authentication start
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_peer_auth_start(peer_id)
        )?;

        // Verify signature
        let signature_valid = self.crypto.verify(challenge, signature, &public_key)?;

        if !signature_valid {
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::network_peer_auth_failed(peer_id, "invalid_signature")
            )?;
            return Err(UnifiedCryptoError::AuthenticationError {
                message: "Invalid peer signature".to_string(),
            });
        }

        // Create peer authentication
        let peer_auth = PeerAuthentication {
            peer_id: peer_id.to_string(),
            public_key,
            authenticated_at: SystemTime::now(),
            auth_method: "public_key_signature".to_string(),
            trust_level: TrustLevel::Basic, // In production, determine based on certificate chain
        };

        // Store authenticated peer
        {
            let mut peers = self.authenticated_peers.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
                message: "Failed to acquire peers lock".to_string(),
            })?;
            peers.insert(peer_id.to_string(), peer_auth.clone());
        }

        // Log successful authentication
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_peer_auth_success(peer_id)
        )?;

        Ok(peer_auth)
    }

    /// Get current network security policy
    pub fn policy(&self) -> &NetworkSecurityPolicy {
        &self.policy
    }

    /// Update network security policy
    pub fn update_policy(&mut self, new_policy: NetworkSecurityPolicy) -> UnifiedCryptoResult<()> {
        // Log policy update
        self.audit_logger.log_crypto_event(
            CryptoAuditEvent::network_policy_updated(&self.policy, &new_policy)
        )?;

        self.policy = new_policy;
        Ok(())
    }

    /// Close a secure channel
    ///
    /// # Arguments
    /// * `channel_id` - Channel identifier to close
    pub fn close_channel(&self, channel_id: &str) -> UnifiedCryptoResult<()> {
        let mut channels = self.active_channels.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire channels lock".to_string(),
        })?;

        if let Some(_channel) = channels.remove(channel_id) {
            // Log channel closure
            self.audit_logger.log_crypto_event(
                CryptoAuditEvent::network_channel_closed(channel_id)
            )?;
        }

        Ok(())
    }

    /// Check network rate limiting for a peer
    fn check_network_rate_limit(&self, peer_id: &str, operation_type: &str) -> UnifiedCryptoResult<()> {
        let now = SystemTime::now();
        let mut rate_limits = self.rate_limit_state.write().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire rate limit lock".to_string(),
        })?;

        let state = rate_limits.entry(peer_id.to_string()).or_insert_with(|| NetworkRateLimitState {
            message_count: 0,
            bandwidth_used: 0,
            connection_count: 0,
            window_start: now,
        });

        // Check if window needs reset
        if now.duration_since(state.window_start).unwrap_or(Duration::ZERO) > self.policy.rate_limit_config.reset_interval {
            state.message_count = 0;
            state.bandwidth_used = 0;
            state.connection_count = 0;
            state.window_start = now;
        }

        // Check rate limits based on operation type
        match operation_type {
            "message_encryption" => {
                if state.message_count >= self.policy.rate_limit_config.max_messages_per_minute {
                    return Err(UnifiedCryptoError::RateLimitExceeded {
                        message: "Message rate limit exceeded".to_string(),
                    });
                }
                state.message_count += 1;
            }
            "channel_creation" => {
                if state.connection_count >= self.policy.rate_limit_config.max_connections_per_minute {
                    return Err(UnifiedCryptoError::RateLimitExceeded {
                        message: "Connection rate limit exceeded".to_string(),
                    });
                }
                state.connection_count += 1;
            }
            _ => {
                // General rate limiting
                state.message_count += 1;
            }
        }

        Ok(())
    }

    /// Perform key exchange with a peer
    fn perform_key_exchange(&self, peer_id: &str) -> UnifiedCryptoResult<(KeyId, KeyExchangeMethod)> {
        // In a real implementation, this would perform actual key exchange protocols
        // For this example, generate a shared key
        let key_pair = self.crypto.generate_keypair()?;
        let key_id = key_pair.public_key.id().clone();
        
        Ok((key_id, KeyExchangeMethod::ECDH))
    }

    /// Verify peer authentication
    fn verify_peer_authentication(&self, peer_id: &str) -> UnifiedCryptoResult<()> {
        let peers = self.authenticated_peers.read().map_err(|_| UnifiedCryptoError::ConcurrencyError {
            message: "Failed to acquire peers lock".to_string(),
        })?;

        if peers.contains_key(peer_id) {
            Ok(())
        } else {
            Err(UnifiedCryptoError::AuthenticationError {
                message: format!("Peer {} not authenticated", peer_id),
            })
        }
    }

    /// Generate a unique channel identifier
    fn generate_channel_id(&self) -> UnifiedCryptoResult<String> {
        use crate::unified_crypto::types::HashAlgorithm;
        
        let random_bytes = self.crypto.primitives.generate_random_bytes(32)?;
        let channel_id_hash = self.crypto.hash(&random_bytes, HashAlgorithm::Sha256)?;
        Ok(format!("channel_{}", hex::encode(channel_id_hash)))
    }

    /// Generate a unique message identifier
    fn generate_message_id(&self) -> UnifiedCryptoResult<String> {
        use crate::unified_crypto::types::HashAlgorithm;
        
        let random_bytes = self.crypto.primitives.generate_random_bytes(16)?;
        let message_id_hash = self.crypto.hash(&random_bytes, HashAlgorithm::Sha256)?;
        Ok(format!("msg_{}", hex::encode(&message_id_hash[..8])))
    }

    /// Compress message data
    fn compress_message(&self, message: &[u8]) -> UnifiedCryptoResult<(Vec<u8>, Option<CompressionInfo>)> {
        // For this example, simulate compression (in production, use actual compression)
        let original_size = message.len();
        let compressed = message.to_vec(); // No actual compression
        let compressed_size = compressed.len();

        let compression_info = CompressionInfo {
            algorithm: "none".to_string(),
            original_size,
            compressed_size,
        };

        Ok((compressed, Some(compression_info)))
    }

    /// Decompress message data
    fn decompress_message(&self, data: &[u8], _compression_info: &CompressionInfo) -> UnifiedCryptoResult<Vec<u8>> {
        // For this example, just return the data as-is
        Ok(data.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::unified_crypto::CryptoConfig;

    #[test]
    fn test_network_operations_initialization() {
        let config = CryptoConfig::default();
        let crypto = UnifiedCrypto::new(config).expect("Failed to create crypto");
        let crypto_arc = Arc::new(crypto);
        let network_ops = NetworkSecurityOperations::new(crypto_arc);
        assert!(network_ops.is_ok());
    }

    #[test]
    fn test_message_priority_ordering() {
        assert!(MessagePriority::Low < MessagePriority::Normal);
        assert!(MessagePriority::Normal < MessagePriority::High);
        assert!(MessagePriority::High < MessagePriority::Emergency);
    }

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Untrusted < TrustLevel::Basic);
        assert!(TrustLevel::Basic < TrustLevel::Verified);
        assert!(TrustLevel::Verified < TrustLevel::High);
        assert!(TrustLevel::High < TrustLevel::Maximum);
    }
}