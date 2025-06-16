//! Database-specific configuration traits for the shared traits system
//!
//! This module provides domain-specific traits for database configurations,
//! implementing common patterns found across database operations including
//! backup, restoration, encryption, and connection management.

use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, TraitConfigError, TraitConfigResult,
    ValidationContext,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Domain-specific trait for database configurations
#[async_trait]
pub trait DatabaseConfig: BaseConfig + ConfigValidation + ConfigLifecycle {
    /// Associated type for connection configuration
    type ConnectionConfig: Clone + std::fmt::Debug;

    /// Associated type for backup configuration
    type BackupConfig: Clone + std::fmt::Debug;

    /// Associated type for encryption configuration
    type EncryptionConfig: Clone + std::fmt::Debug;

    /// Associated type for performance tuning
    type PerformanceConfig: Clone + std::fmt::Debug + Default;

    /// Get the connection configuration
    fn connection_config(&self) -> &Self::ConnectionConfig;

    /// Get backup configuration
    fn backup_config(&self) -> &Self::BackupConfig;

    /// Get encryption configuration
    fn encryption_config(&self) -> &Self::EncryptionConfig;

    /// Get performance tuning configuration
    fn performance_config(&self) -> &Self::PerformanceConfig;

    /// Validate database connectivity
    async fn validate_connectivity(&self) -> TraitConfigResult<()>;

    /// Apply environment variable overrides to the configuration
    async fn apply_env_overrides(&mut self) -> TraitConfigResult<()>;

    /// Validate backup settings
    fn validate_backup_settings(&self) -> TraitConfigResult<()>;

    /// Validate encryption settings
    fn validate_encryption_settings(&self) -> TraitConfigResult<()>;

    /// Validate performance settings
    fn validate_performance_settings(&self) -> TraitConfigResult<()>;
}

/// Trait for database connection configuration
pub trait ConnectionConfigTrait: Clone + std::fmt::Debug {
    /// Get the database path or connection string
    fn database_path(&self) -> &Path;

    /// Get connection timeout
    fn connection_timeout(&self) -> Duration;

    /// Get maximum number of connections
    fn max_connections(&self) -> u32;

    /// Whether to create database if it doesn't exist
    fn create_if_missing(&self) -> bool;

    /// Validate connection configuration
    fn validate(&self) -> TraitConfigResult<()>;
}

/// Trait for backup configuration
pub trait BackupConfigTrait: Clone + std::fmt::Debug {
    /// Associated type for backup mode
    type BackupMode: Clone + std::fmt::Debug;

    /// Get backup mode (full, incremental, etc.)
    fn backup_mode(&self) -> Self::BackupMode;

    /// Get backup destination path
    fn backup_path(&self) -> &Path;

    /// Get compression level (0-9)
    fn compression_level(&self) -> u8;

    /// Whether to verify integrity during backup
    fn verify_during_creation(&self) -> bool;

    /// Whether to include metadata in backup
    fn include_metadata(&self) -> bool;

    /// Get retention policy (number of backups to keep)
    fn retention_count(&self) -> u32;

    /// Validate backup configuration
    fn validate(&self) -> TraitConfigResult<()>;
}

/// Trait for encryption configuration
pub trait EncryptionConfigTrait: Clone + std::fmt::Debug {
    /// Whether encryption is enabled
    fn encryption_enabled(&self) -> bool;

    /// Get encryption algorithm identifier
    fn encryption_algorithm(&self) -> &str;

    /// Get key derivation method
    fn key_derivation_method(&self) -> &str;

    /// Whether to encrypt at rest
    fn encrypt_at_rest(&self) -> bool;

    /// Whether to encrypt backups
    fn encrypt_backups(&self) -> bool;

    /// Validate encryption configuration
    fn validate(&self) -> TraitConfigResult<()>;
}

/// Backup operation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackupMode {
    /// Full backup of entire database
    Full,
    /// Incremental backup since last backup
    Incremental,
    /// Differential backup since last full backup
    Differential,
}

impl Default for BackupMode {
    fn default() -> Self {
        Self::Full
    }
}

/// Standard backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardBackupConfig {
    /// Backup mode
    pub mode: BackupMode,
    /// Backup destination directory
    pub backup_directory: PathBuf,
    /// Compression level (0-9, 0 = no compression)
    pub compression_level: u8,
    /// Verify integrity during backup creation
    pub verify_during_creation: bool,
    /// Include metadata trees in backup
    pub include_metadata: bool,
    /// Number of backups to retain
    pub retention_count: u32,
    /// Optional filter for specific tree names
    pub tree_filter: Option<Vec<String>>,
    /// Optional key prefix filter
    pub key_prefix_filter: Option<String>,
}

impl Default for StandardBackupConfig {
    fn default() -> Self {
        Self {
            mode: BackupMode::Full,
            backup_directory: PathBuf::from("./backups"),
            compression_level: 6,
            verify_during_creation: true,
            include_metadata: true,
            retention_count: 10,
            tree_filter: None,
            key_prefix_filter: None,
        }
    }
}

impl BackupConfigTrait for StandardBackupConfig {
    type BackupMode = BackupMode;

    fn backup_mode(&self) -> Self::BackupMode {
        self.mode
    }

    fn backup_path(&self) -> &Path {
        &self.backup_directory
    }

    fn compression_level(&self) -> u8 {
        self.compression_level
    }

    fn verify_during_creation(&self) -> bool {
        self.verify_during_creation
    }

    fn include_metadata(&self) -> bool {
        self.include_metadata
    }

    fn retention_count(&self) -> u32 {
        self.retention_count
    }

    fn validate(&self) -> TraitConfigResult<()> {
        if self.compression_level > 9 {
            return Err(TraitConfigError::ValidationError {
                field: "compression_level".to_string(),
                message: "Compression level must be between 0 and 9".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.retention_count == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "retention_count".to_string(),
                message: "Retention count must be at least 1".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.retention_count > 1000 {
            return Err(TraitConfigError::ValidationError {
                field: "retention_count".to_string(),
                message: "Retention count should not exceed 1000".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Standard database connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardConnectionConfig {
    /// Database file path or connection string
    pub database_path: PathBuf,
    /// Connection timeout in seconds
    pub connection_timeout_seconds: u64,
    /// Maximum number of concurrent connections
    pub max_connections: u32,
    /// Create database if it doesn't exist
    pub create_if_missing: bool,
    /// Enable write-ahead logging
    pub enable_wal: bool,
    /// Sync mode for durability
    pub sync_mode: SyncMode,
}

/// Database synchronization modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    /// No synchronization (fastest, least safe)
    Off,
    /// Normal synchronization
    Normal,
    /// Full synchronization (slowest, safest)
    Full,
}

impl Default for SyncMode {
    fn default() -> Self {
        Self::Normal
    }
}

impl Default for StandardConnectionConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from("./database"),
            connection_timeout_seconds: 30,
            max_connections: 100,
            create_if_missing: true,
            enable_wal: true,
            sync_mode: SyncMode::Normal,
        }
    }
}

impl ConnectionConfigTrait for StandardConnectionConfig {
    fn database_path(&self) -> &Path {
        &self.database_path
    }

    fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_seconds)
    }

    fn max_connections(&self) -> u32 {
        self.max_connections
    }

    fn create_if_missing(&self) -> bool {
        self.create_if_missing
    }

    fn validate(&self) -> TraitConfigResult<()> {
        if self.connection_timeout_seconds == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "connection_timeout_seconds".to_string(),
                message: "Connection timeout must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.connection_timeout_seconds > 3600 {
            return Err(TraitConfigError::ValidationError {
                field: "connection_timeout_seconds".to_string(),
                message: "Connection timeout should not exceed 3600 seconds".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_connections == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Maximum connections must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_connections > 10000 {
            return Err(TraitConfigError::ValidationError {
                field: "max_connections".to_string(),
                message: "Maximum connections should not exceed 10000".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Standard encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StandardEncryptionConfig {
    /// Whether encryption is enabled
    pub enabled: bool,
    /// Encryption algorithm (e.g., "AES-256-GCM")
    pub algorithm: String,
    /// Key derivation method (e.g., "PBKDF2", "Argon2")
    pub key_derivation: String,
    /// Whether to encrypt data at rest
    pub encrypt_at_rest: bool,
    /// Whether to encrypt backups
    pub encrypt_backups: bool,
    /// Key rotation interval in days
    pub key_rotation_days: u32,
    /// Whether to use hardware security modules
    pub use_hsm: bool,
}

impl Default for StandardEncryptionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            algorithm: "AES-256-GCM".to_string(),
            key_derivation: "Argon2".to_string(),
            encrypt_at_rest: true,
            encrypt_backups: true,
            key_rotation_days: 90,
            use_hsm: false,
        }
    }
}

impl EncryptionConfigTrait for StandardEncryptionConfig {
    fn encryption_enabled(&self) -> bool {
        self.enabled
    }

    fn encryption_algorithm(&self) -> &str {
        &self.algorithm
    }

    fn key_derivation_method(&self) -> &str {
        &self.key_derivation
    }

    fn encrypt_at_rest(&self) -> bool {
        self.encrypt_at_rest
    }

    fn encrypt_backups(&self) -> bool {
        self.encrypt_backups
    }

    fn validate(&self) -> TraitConfigResult<()> {
        if self.enabled {
            if self.algorithm.is_empty() {
                return Err(TraitConfigError::ValidationError {
                    field: "algorithm".to_string(),
                    message: "Encryption algorithm must be specified when encryption is enabled"
                        .to_string(),
                    context: ValidationContext::default(),
                });
            }

            if self.key_derivation.is_empty() {
                return Err(TraitConfigError::ValidationError {
                    field: "key_derivation".to_string(),
                    message: "Key derivation method must be specified when encryption is enabled"
                        .to_string(),
                    context: ValidationContext::default(),
                });
            }
        }

        if self.key_rotation_days == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "key_rotation_days".to_string(),
                message: "Key rotation interval must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.key_rotation_days > 3650 {
            // 10 years
            return Err(TraitConfigError::ValidationError {
                field: "key_rotation_days".to_string(),
                message: "Key rotation interval should not exceed 3650 days".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

/// Database performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabasePerformanceConfig {
    /// Cache size in MB
    pub cache_size_mb: u64,
    /// Number of background threads
    pub background_threads: u32,
    /// Enable automatic compaction
    pub auto_compaction: bool,
    /// Compaction interval in hours
    pub compaction_interval_hours: u32,
    /// Maximum batch size for operations
    pub max_batch_size: usize,
    /// Enable statistics collection
    pub enable_statistics: bool,
}

impl Default for DatabasePerformanceConfig {
    fn default() -> Self {
        Self {
            cache_size_mb: 256,
            background_threads: 4,
            auto_compaction: true,
            compaction_interval_hours: 24,
            max_batch_size: 1000,
            enable_statistics: true,
        }
    }
}

impl DatabasePerformanceConfig {
    /// Validate performance configuration
    pub fn validate(&self) -> TraitConfigResult<()> {
        if self.cache_size_mb == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "cache_size_mb".to_string(),
                message: "Cache size must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.cache_size_mb > 32768 {
            // 32GB
            return Err(TraitConfigError::ValidationError {
                field: "cache_size_mb".to_string(),
                message: "Cache size should not exceed 32GB".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.background_threads == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "background_threads".to_string(),
                message: "Background threads must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.background_threads > 64 {
            return Err(TraitConfigError::ValidationError {
                field: "background_threads".to_string(),
                message: "Background threads should not exceed 64".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.max_batch_size == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "max_batch_size".to_string(),
                message: "Maximum batch size must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        if self.compaction_interval_hours == 0 {
            return Err(TraitConfigError::ValidationError {
                field: "compaction_interval_hours".to_string(),
                message: "Compaction interval must be greater than 0".to_string(),
                context: ValidationContext::default(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backup_config() {
        let config = StandardBackupConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.backup_mode(), BackupMode::Full);
        assert_eq!(config.compression_level(), 6);
        assert!(config.verify_during_creation());
        assert!(config.include_metadata());
        assert_eq!(config.retention_count(), 10);

        let mut invalid_config = config.clone();
        invalid_config.compression_level = 10;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.retention_count = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_connection_config() {
        let config = StandardConnectionConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.create_if_missing());
        assert_eq!(config.max_connections(), 100);
        assert_eq!(config.connection_timeout(), Duration::from_secs(30));

        let mut invalid_config = config.clone();
        invalid_config.connection_timeout_seconds = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.max_connections = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_encryption_config() {
        let config = StandardEncryptionConfig::default();
        assert!(config.validate().is_ok());
        assert!(config.encryption_enabled());
        assert_eq!(config.encryption_algorithm(), "AES-256-GCM");
        assert_eq!(config.key_derivation_method(), "Argon2");
        assert!(config.encrypt_at_rest());
        assert!(config.encrypt_backups());

        let mut invalid_config = config.clone();
        invalid_config.key_rotation_days = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.algorithm = String::new();
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_performance_config() {
        let config = DatabasePerformanceConfig::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.cache_size_mb, 256);
        assert_eq!(config.background_threads, 4);
        assert!(config.auto_compaction);
        assert_eq!(config.max_batch_size, 1000);

        let mut invalid_config = config.clone();
        invalid_config.cache_size_mb = 0;
        assert!(invalid_config.validate().is_err());

        let mut invalid_config = config.clone();
        invalid_config.background_threads = 0;
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_backup_modes() {
        assert_eq!(BackupMode::default(), BackupMode::Full);

        let modes = [
            BackupMode::Full,
            BackupMode::Incremental,
            BackupMode::Differential,
        ];
        for mode in modes {
            let config = StandardBackupConfig {
                mode,
                ..Default::default()
            };
            assert_eq!(config.backup_mode(), mode);
        }
    }

    #[test]
    fn test_sync_modes() {
        assert_eq!(SyncMode::default(), SyncMode::Normal);

        let modes = [SyncMode::Off, SyncMode::Normal, SyncMode::Full];
        for mode in modes {
            let config = StandardConnectionConfig {
                sync_mode: mode,
                ..Default::default()
            };
            assert_eq!(config.sync_mode, mode);
        }
    }
}
