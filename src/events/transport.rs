//! Event Transport and Serialization
//!
//! This module handles cross-platform event transport, serialization protocols,
//! and communication between different DataFold SDK implementations.

use super::event_types::SecurityEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;

/// Transport protocol for cross-platform events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportProtocol {
    /// In-memory transport (for single process)
    InMemory,
    /// HTTP/HTTPS transport
    Http {
        endpoint: String,
        headers: HashMap<String, String>,
    },
    /// WebSocket transport
    WebSocket { url: String },
    /// TCP socket transport
    Tcp { host: String, port: u16 },
    /// Unix domain socket transport
    UnixSocket { path: String },
    /// Message queue transport (Redis, RabbitMQ, etc.)
    MessageQueue {
        broker_url: String,
        queue_name: String,
    },
}

/// Configuration for event transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport protocol to use
    pub protocol: TransportProtocol,
    /// Enable compression for transport
    pub compression: bool,
    /// Serialization format
    pub serialization: SerializationFormat,
    /// Connection timeout in milliseconds
    pub connection_timeout_ms: u64,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Enable transport encryption
    pub encryption: bool,
    /// Buffer size for batching events
    pub buffer_size: usize,
    /// Batch timeout for sending events
    pub batch_timeout_ms: u64,
}

/// Serialization formats supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SerializationFormat {
    /// JSON serialization (human-readable, larger size)
    Json,
    /// MessagePack serialization (binary, compact)
    MessagePack,
    /// Protocol Buffers (binary, very compact, schema evolution)
    ProtocolBuffers,
    /// CBOR (binary, JSON-like but compact)
    Cbor,
}

/// Retry configuration for failed transports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Backoff strategy
    pub backoff: BackoffStrategy,
    /// Whether to retry on specific error types
    pub retry_on_errors: Vec<String>,
}

/// Backoff strategies for retry attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Exponential backoff with jitter
    Exponential,
    /// Linear backoff
    Linear,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            protocol: TransportProtocol::InMemory,
            compression: false,
            serialization: SerializationFormat::Json,
            connection_timeout_ms: 5000,
            retry_config: RetryConfig {
                max_retries: 3,
                base_delay_ms: 1000,
                max_delay_ms: 30000,
                backoff: BackoffStrategy::Exponential,
                retry_on_errors: vec![
                    "ConnectionError".to_string(),
                    "TimeoutError".to_string(),
                    "TemporaryFailure".to_string(),
                ],
            },
            encryption: false,
            buffer_size: 100,
            batch_timeout_ms: 5000,
        }
    }
}

/// Transport envelope for cross-platform events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Transport protocol version
    pub version: String,
    /// Source platform information
    pub source: PlatformInfo,
    /// Target platform (if specific)
    pub target: Option<PlatformInfo>,
    /// Event payload
    pub event: SecurityEvent,
    /// Transport metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp when envelope was created
    pub envelope_timestamp: chrono::DateTime<chrono::Utc>,
    /// Unique envelope identifier
    pub envelope_id: uuid::Uuid,
}

/// Platform information for transport
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    /// Platform type (rust-cli, js-sdk, python-sdk, etc.)
    pub platform_type: String,
    /// Platform version
    pub version: String,
    /// Host information
    pub host: Option<String>,
    /// Process or instance identifier
    pub instance_id: Option<String>,
    /// Additional platform metadata
    pub metadata: HashMap<String, String>,
}

/// Result of transport operation
#[derive(Debug, Clone)]
pub struct TransportResult {
    /// Whether transport was successful
    pub success: bool,
    /// Transport latency
    pub latency: std::time::Duration,
    /// Error message if failed
    pub error: Option<String>,
    /// Number of bytes sent/received
    pub bytes_transferred: Option<usize>,
    /// Transport metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Statistics about transport operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportStatistics {
    /// Total events sent
    pub events_sent: u64,
    /// Total events received
    pub events_received: u64,
    /// Failed send attempts
    pub send_failures: u64,
    /// Failed receive attempts
    pub receive_failures: u64,
    /// Average send latency in milliseconds
    pub avg_send_latency_ms: f64,
    /// Average receive latency in milliseconds
    pub avg_receive_latency_ms: f64,
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Connection failures
    pub connection_failures: u64,
    /// Successful connections
    pub successful_connections: u64,
    /// Current connection status
    pub connected: bool,
}

/// Trait for implementing event transport
#[async_trait::async_trait]
pub trait EventTransport: Send + Sync {
    /// Initialize the transport
    async fn initialize(&mut self) -> Result<(), TransportError>;

    /// Send an event through the transport
    async fn send_event(&self, envelope: EventEnvelope) -> Result<TransportResult, TransportError>;

    /// Send multiple events in batch
    async fn send_batch(
        &self,
        envelopes: Vec<EventEnvelope>,
    ) -> Result<Vec<TransportResult>, TransportError>;

    /// Receive events from transport (for subscribers)
    async fn receive_events(&self) -> Result<Vec<EventEnvelope>, TransportError>;

    /// Subscribe to events (returns a receiver)
    async fn subscribe(&self) -> Result<broadcast::Receiver<EventEnvelope>, TransportError>;

    /// Get transport statistics
    async fn get_statistics(&self) -> TransportStatistics;

    /// Check if transport is connected/healthy
    async fn is_healthy(&self) -> bool;

    /// Close the transport connection
    async fn close(&mut self) -> Result<(), TransportError>;
}

/// Transport errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Transport timeout")]
    Timeout,

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Transport not initialized")]
    NotInitialized,

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Buffer overflow")]
    BufferOverflow,

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Transport closed")]
    TransportClosed,
}

/// In-memory transport implementation (for single process)
pub struct InMemoryTransport {
    /// Configuration
    config: TransportConfig,
    /// Event sender
    sender: broadcast::Sender<EventEnvelope>,
    /// Platform information
    #[allow(dead_code)]
    platform_info: PlatformInfo,
    /// Transport statistics
    statistics: std::sync::Arc<tokio::sync::RwLock<TransportStatistics>>,
}

impl InMemoryTransport {
    /// Create new in-memory transport
    pub fn new(config: TransportConfig, platform_info: PlatformInfo) -> Self {
        let (sender, _) = broadcast::channel(config.buffer_size);

        Self {
            config,
            sender,
            platform_info,
            statistics: std::sync::Arc::new(tokio::sync::RwLock::new(TransportStatistics {
                events_sent: 0,
                events_received: 0,
                send_failures: 0,
                receive_failures: 0,
                avg_send_latency_ms: 0.0,
                avg_receive_latency_ms: 0.0,
                bytes_sent: 0,
                bytes_received: 0,
                connection_failures: 0,
                successful_connections: 1, // Always connected for in-memory
                connected: true,
            })),
        }
    }
}

#[async_trait::async_trait]
impl EventTransport for InMemoryTransport {
    async fn initialize(&mut self) -> Result<(), TransportError> {
        // In-memory transport is always ready
        Ok(())
    }

    async fn send_event(&self, envelope: EventEnvelope) -> Result<TransportResult, TransportError> {
        let start_time = std::time::Instant::now();

        // Serialize envelope to calculate size
        let serialized = match self.config.serialization {
            SerializationFormat::Json => serde_json::to_vec(&envelope)
                .map_err(|e| TransportError::SerializationError(e.to_string()))?,
            _ => {
                // For now, fallback to JSON for other formats
                serde_json::to_vec(&envelope)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))?
            }
        };

        let bytes_size = serialized.len();

        // Send through broadcast channel
        match self.sender.send(envelope) {
            Ok(_) => {
                let latency = start_time.elapsed();

                // Update statistics
                let mut stats = self.statistics.write().await;
                stats.events_sent += 1;
                stats.bytes_sent += bytes_size as u64;

                // Update average latency (exponential moving average)
                let latency_ms = latency.as_millis() as f64;
                stats.avg_send_latency_ms = if stats.avg_send_latency_ms == 0.0 {
                    latency_ms
                } else {
                    0.9 * stats.avg_send_latency_ms + 0.1 * latency_ms
                };

                Ok(TransportResult {
                    success: true,
                    latency,
                    error: None,
                    bytes_transferred: Some(bytes_size),
                    metadata: HashMap::new(),
                })
            }
            Err(broadcast::error::SendError(_)) => {
                // Channel has no receivers - this is okay for a broadcast channel
                // We'll still consider it a successful send
                let latency = start_time.elapsed();

                let mut stats = self.statistics.write().await;
                stats.events_sent += 1;
                stats.bytes_sent += bytes_size as u64;

                let latency_ms = latency.as_millis() as f64;
                stats.avg_send_latency_ms = if stats.avg_send_latency_ms == 0.0 {
                    latency_ms
                } else {
                    0.9 * stats.avg_send_latency_ms + 0.1 * latency_ms
                };

                Ok(TransportResult {
                    success: true,
                    latency,
                    error: None,
                    bytes_transferred: Some(bytes_size),
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("no_receivers".to_string(), serde_json::Value::Bool(true));
                        metadata
                    },
                })
            }
        }
    }

    async fn send_batch(
        &self,
        envelopes: Vec<EventEnvelope>,
    ) -> Result<Vec<TransportResult>, TransportError> {
        let mut results = Vec::new();

        for envelope in envelopes {
            let result = self.send_event(envelope).await;
            results.push(result?);
        }

        Ok(results)
    }

    async fn receive_events(&self) -> Result<Vec<EventEnvelope>, TransportError> {
        // For in-memory transport, receiving is handled through subscription
        // This method could be used for polling-based receivers
        Ok(Vec::new())
    }

    async fn subscribe(&self) -> Result<broadcast::Receiver<EventEnvelope>, TransportError> {
        Ok(self.sender.subscribe())
    }

    async fn get_statistics(&self) -> TransportStatistics {
        self.statistics.read().await.clone()
    }

    async fn is_healthy(&self) -> bool {
        true // In-memory transport is always healthy
    }

    async fn close(&mut self) -> Result<(), TransportError> {
        // Nothing to close for in-memory transport
        let mut stats = self.statistics.write().await;
        stats.connected = false;
        Ok(())
    }
}

/// Event serializer for different formats
pub struct EventSerializer;

impl EventSerializer {
    /// Serialize event envelope to bytes
    pub fn serialize(
        envelope: &EventEnvelope,
        format: &SerializationFormat,
    ) -> Result<Vec<u8>, TransportError> {
        match format {
            SerializationFormat::Json => serde_json::to_vec(envelope)
                .map_err(|e| TransportError::SerializationError(e.to_string())),
            SerializationFormat::MessagePack => {
                // Placeholder - would use rmp_serde crate
                serde_json::to_vec(envelope)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
            SerializationFormat::ProtocolBuffers => {
                // Placeholder - would use prost crate
                serde_json::to_vec(envelope)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
            SerializationFormat::Cbor => {
                // Placeholder - would use serde_cbor crate
                serde_json::to_vec(envelope)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
        }
    }

    /// Deserialize bytes to event envelope
    pub fn deserialize(
        bytes: &[u8],
        format: &SerializationFormat,
    ) -> Result<EventEnvelope, TransportError> {
        match format {
            SerializationFormat::Json => serde_json::from_slice(bytes)
                .map_err(|e| TransportError::SerializationError(e.to_string())),
            SerializationFormat::MessagePack => {
                // Placeholder - would use rmp_serde crate
                serde_json::from_slice(bytes)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
            SerializationFormat::ProtocolBuffers => {
                // Placeholder - would use prost crate
                serde_json::from_slice(bytes)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
            SerializationFormat::Cbor => {
                // Placeholder - would use serde_cbor crate
                serde_json::from_slice(bytes)
                    .map_err(|e| TransportError::SerializationError(e.to_string()))
            }
        }
    }
}

/// Transport factory for creating transport instances
pub struct TransportFactory;

impl TransportFactory {
    /// Create a transport instance based on configuration
    pub fn create_transport(
        config: TransportConfig,
        platform_info: PlatformInfo,
    ) -> Result<Box<dyn EventTransport>, TransportError> {
        match config.protocol {
            TransportProtocol::InMemory => {
                Ok(Box::new(InMemoryTransport::new(config, platform_info)))
            }
            TransportProtocol::Http { .. } => {
                // Placeholder for HTTP transport implementation
                Err(TransportError::ConfigurationError(
                    "HTTP transport not yet implemented".to_string(),
                ))
            }
            TransportProtocol::WebSocket { .. } => {
                // Placeholder for WebSocket transport implementation
                Err(TransportError::ConfigurationError(
                    "WebSocket transport not yet implemented".to_string(),
                ))
            }
            TransportProtocol::Tcp { .. } => {
                // Placeholder for TCP transport implementation
                Err(TransportError::ConfigurationError(
                    "TCP transport not yet implemented".to_string(),
                ))
            }
            TransportProtocol::UnixSocket { .. } => {
                // Placeholder for Unix socket transport implementation
                Err(TransportError::ConfigurationError(
                    "Unix socket transport not yet implemented".to_string(),
                ))
            }
            TransportProtocol::MessageQueue { .. } => {
                // Placeholder for message queue transport implementation
                Err(TransportError::ConfigurationError(
                    "Message queue transport not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Get default platform info for current platform
    pub fn get_platform_info() -> PlatformInfo {
        PlatformInfo {
            platform_type: "rust-cli".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            host: std::env::var("HOSTNAME")
                .ok()
                .or_else(|| std::env::var("COMPUTERNAME").ok()),
            instance_id: Some(uuid::Uuid::new_v4().to_string()),
            metadata: {
                let mut metadata = HashMap::new();
                metadata.insert("arch".to_string(), std::env::consts::ARCH.to_string());
                metadata.insert("os".to_string(), std::env::consts::OS.to_string());
                metadata.insert("family".to_string(), std::env::consts::FAMILY.to_string());
                metadata
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{
        CreateVerificationEvent, PlatformSource, SecurityEvent,
        SecurityEventCategory, VerificationEvent,
    };
    use crate::security_types::Severity;

    #[tokio::test]
    async fn test_in_memory_transport() {
        let config = TransportConfig::default();
        let platform_info = TransportFactory::get_platform_info();

        let mut transport = InMemoryTransport::new(config, platform_info.clone());

        // Initialize transport
        transport.initialize().await.unwrap();
        assert!(transport.is_healthy().await);

        // Create test event
        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "test_component".to_string(),
            "test_operation".to_string(),
        ));

        let envelope = EventEnvelope {
            version: "1.0".to_string(),
            source: platform_info,
            target: None,
            event,
            metadata: HashMap::new(),
            envelope_timestamp: chrono::Utc::now(),
            envelope_id: uuid::Uuid::new_v4(),
        };

        // Send event
        let result = transport.send_event(envelope).await.unwrap();
        assert!(result.success);
        assert!(result.bytes_transferred.is_some());

        // Check statistics
        let stats = transport.get_statistics().await;
        assert_eq!(stats.events_sent, 1);
        assert!(stats.bytes_sent > 0);
    }

    #[tokio::test]
    async fn test_event_serialization() {
        let event = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Critical,
            PlatformSource::DataFoldNode,
            "security_monitor".to_string(),
            "threat_detected".to_string(),
        ));

        let envelope = EventEnvelope {
            version: "1.0".to_string(),
            source: TransportFactory::get_platform_info(),
            target: None,
            event,
            metadata: HashMap::new(),
            envelope_timestamp: chrono::Utc::now(),
            envelope_id: uuid::Uuid::new_v4(),
        };

        // Test JSON serialization
        let serialized = EventSerializer::serialize(&envelope, &SerializationFormat::Json).unwrap();
        assert!(!serialized.is_empty());

        let deserialized =
            EventSerializer::deserialize(&serialized, &SerializationFormat::Json).unwrap();
        assert_eq!(envelope.envelope_id, deserialized.envelope_id);
        assert_eq!(envelope.version, deserialized.version);
    }

    #[test]
    fn test_transport_factory() {
        let config = TransportConfig::default();
        let platform_info = TransportFactory::get_platform_info();

        let transport = TransportFactory::create_transport(config, platform_info);
        assert!(transport.is_ok());

        // Test platform info
        let platform_info = TransportFactory::get_platform_info();
        assert_eq!(platform_info.platform_type, "rust-cli");
        assert!(!platform_info.version.is_empty());
        assert!(platform_info.instance_id.is_some());
    }
}
