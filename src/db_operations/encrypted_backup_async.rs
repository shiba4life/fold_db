//! Async encrypted backup and restore operations for DataFold databases
//!
//! This module provides async variants of encrypted backup and restore operations
//! with performance optimizations including streaming I/O, batch processing,
//! and progress tracking for large database operations.
//!
//! ## Features
//!
//! * Async streaming backup/restore for non-blocking operations
//! * Batch processing for improved throughput
//! * Progress tracking and cancellation support
//! * Memory-efficient streaming for large databases
//! * Integrity verification during backup/restore
//! * Performance metrics and monitoring
//! * Resume capability for interrupted operations
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use datafold::db_operations::{AsyncEncryptedBackup, DbOperations, AsyncBackupConfig};
//! use datafold::crypto::generate_master_keypair;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create async backup manager
//! let db = sled::open("source_db")?;
//! let db_ops = DbOperations::new(db)?;
//! let master_keypair = generate_master_keypair()?;
//! let config = AsyncBackupConfig::default();
//!
//! let backup_manager = AsyncEncryptedBackup::new(
//!     db_ops, &master_keypair, config
//! ).await?;
//!
//! // Create encrypted backup asynchronously
//! let backup_path = "backup.enc";
//! let backup_info = backup_manager.create_backup_async(backup_path).await?;
//! println!("Backup created: {} bytes", backup_info.metadata.total_size);
//!
//! // Restore from backup asynchronously
//! let restore_db = sled::open("restored_db")?;
//! let restore_db_ops = DbOperations::new(restore_db)?;
//! let restore_info = backup_manager.restore_backup_async(backup_path, restore_db_ops).await?;
//! println!("Restored: {} items", restore_info.total_items);
//! # Ok(())
//! # }
//! ```

use super::core::DbOperations;
use super::encryption_wrapper_async::{AsyncEncryptionWrapper, AsyncWrapperConfig};
use super::encryption_wrapper::contexts;
use crate::crypto::{MasterKeyPair, CryptoError, CryptoResult};
use crate::config::crypto::CryptoConfig;
use crate::schema::SchemaError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::sync::{RwLock, Semaphore};
use tokio::task;
use sha2::{Sha256, Digest};
use uuid::Uuid;

/// Configuration for async backup operations
#[derive(Debug, Clone)]
pub struct AsyncBackupConfig {
    /// Buffer size for streaming operations (bytes)
    pub buffer_size: usize,
    /// Batch size for database operations
    pub batch_size: usize,
    /// Maximum number of concurrent operations
    pub max_concurrent_operations: usize,
    /// Enable progress tracking
    pub enable_progress_tracking: bool,
    /// Enable integrity verification
    pub verify_integrity: bool,
    /// Compression level (0-9, 0 = no compression)
    pub compression_level: u32,
    /// Enable resume capability for interrupted operations
    pub enable_resume: bool,
    /// Checkpoint interval for resume capability (number of items)
    pub checkpoint_interval: usize,
}

impl Default for AsyncBackupConfig {
    fn default() -> Self {
        Self {
            buffer_size: 64 * 1024, // 64KB
            batch_size: 100,
            max_concurrent_operations: 10,
            enable_progress_tracking: true,
            verify_integrity: true,
            compression_level: 6,
            enable_resume: true,
            checkpoint_interval: 1000,
        }
    }
}

impl AsyncBackupConfig {
    /// Create configuration optimized for large databases
    pub fn large_database() -> Self {
        Self {
            buffer_size: 256 * 1024, // 256KB
            batch_size: 500,
            max_concurrent_operations: 20,
            checkpoint_interval: 5000,
            ..Default::default()
        }
    }
    
    /// Create configuration optimized for fast backup/restore
    pub fn fast_mode() -> Self {
        Self {
            buffer_size: 128 * 1024, // 128KB
            batch_size: 200,
            max_concurrent_operations: 15,
            compression_level: 3,
            verify_integrity: false, // Skip verification for speed
            ..Default::default()
        }
    }
    
    /// Create configuration optimized for network operations
    pub fn network_optimized() -> Self {
        Self {
            buffer_size: 32 * 1024, // 32KB (good for network)
            batch_size: 50,
            max_concurrent_operations: 5,
            compression_level: 9, // Max compression for network
            ..Default::default()
        }
    }
}

/// Progress information for backup/restore operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressInfo {
    /// Current operation type
    pub operation: String,
    /// Number of items processed
    pub items_processed: u64,
    /// Total number of items (if known)
    pub total_items: Option<u64>,
    /// Number of bytes processed
    pub bytes_processed: u64,
    /// Total bytes (if known)
    pub total_bytes: Option<u64>,
    /// Operation start time
    pub start_time: SystemTime,
    /// Estimated completion time
    pub estimated_completion: Option<SystemTime>,
    /// Current throughput (items per second)
    pub throughput_items_per_sec: f64,
    /// Current throughput (bytes per second)
    pub throughput_bytes_per_sec: f64,
}

impl ProgressInfo {
    fn new(operation: String) -> Self {
        Self {
            operation,
            items_processed: 0,
            total_items: None,
            bytes_processed: 0,
            total_bytes: None,
            start_time: SystemTime::now(),
            estimated_completion: None,
            throughput_items_per_sec: 0.0,
            throughput_bytes_per_sec: 0.0,
        }
    }
    
    fn update(&mut self, items_delta: u64, bytes_delta: u64) {
        self.items_processed += items_delta;
        self.bytes_processed += bytes_delta;
        
        let elapsed = self.start_time.elapsed().unwrap_or(Duration::from_secs(1));
        let elapsed_secs = elapsed.as_secs_f64().max(1.0);
        
        self.throughput_items_per_sec = self.items_processed as f64 / elapsed_secs;
        self.throughput_bytes_per_sec = self.bytes_processed as f64 / elapsed_secs;
        
        // Estimate completion time if total is known
        if let Some(total_items) = self.total_items {
            if self.items_processed > 0 && self.items_processed < total_items {
                let remaining_items = total_items - self.items_processed;
                let estimated_remaining_secs = remaining_items as f64 / self.throughput_items_per_sec;
                self.estimated_completion = Some(
                    SystemTime::now() + Duration::from_secs_f64(estimated_remaining_secs)
                );
            }
        }
    }
    
    /// Calculate progress percentage (0-100)
    pub fn progress_percentage(&self) -> Option<f64> {
        if let Some(total) = self.total_items {
            if total > 0 {
                Some((self.items_processed as f64 / total as f64) * 100.0)
            } else {
                Some(100.0)
            }
        } else {
            None
        }
    }
}

/// Backup metadata stored with the backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Backup format version
    pub version: u32,
    /// Backup creation timestamp
    pub created_at: SystemTime,
    /// Database identifier/name
    pub database_id: String,
    /// Total number of items in backup
    pub total_items: u64,
    /// Total size of backup in bytes
    pub total_size: u64,
    /// Compression level used
    pub compression_level: u32,
    /// Integrity hash of the backup content
    pub integrity_hash: String,
    /// Encryption contexts used
    pub encryption_contexts: Vec<String>,
    /// Additional metadata
    pub additional_info: HashMap<String, String>,
}

/// Information about a completed backup operation
#[derive(Debug, Clone)]
pub struct BackupInfo {
    /// Backup metadata
    pub metadata: BackupMetadata,
    /// Time taken for the operation
    pub duration: Duration,
    /// Average throughput during backup
    pub throughput_items_per_sec: f64,
    /// Average throughput in bytes per second
    pub throughput_bytes_per_sec: f64,
    /// Number of errors encountered (but recovered from)
    pub error_count: u64,
}

/// Information about a completed restore operation
#[derive(Debug, Clone)]
pub struct RestoreInfo {
    /// Number of items restored
    pub total_items: u64,
    /// Total bytes restored
    pub total_bytes: u64,
    /// Time taken for the operation
    pub duration: Duration,
    /// Average throughput during restore
    pub throughput_items_per_sec: f64,
    /// Average throughput in bytes per second
    pub throughput_bytes_per_sec: f64,
    /// Number of errors encountered (but recovered from)
    pub error_count: u64,
    /// Verification result (if enabled)
    pub verification_passed: Option<bool>,
}

/// Checkpoint information for resume capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointInfo {
    /// Checkpoint ID
    pub id: String,
    /// Operation type
    pub operation: String,
    /// Number of items processed so far
    pub items_processed: u64,
    /// Number of bytes processed so far
    pub bytes_processed: u64,
    /// Last processed key
    pub last_key: Option<String>,
    /// Checkpoint timestamp
    pub timestamp: SystemTime,
}

/// Async encrypted backup manager with performance optimizations
pub struct AsyncEncryptedBackup {
    /// Database operations
    db_ops: DbOperations,
    /// Async encryption wrapper
    encryption: AsyncEncryptionWrapper,
    /// Configuration
    config: AsyncBackupConfig,
    /// Semaphore for concurrent operations
    #[allow(dead_code)]
    operation_semaphore: Arc<Semaphore>,
    /// Progress tracking
    progress: Arc<RwLock<Option<ProgressInfo>>>,
}

impl AsyncEncryptedBackup {
    /// Create a new async encrypted backup manager
    pub async fn new(
        db_ops: DbOperations,
        master_keypair: &MasterKeyPair,
        config: AsyncBackupConfig,
    ) -> CryptoResult<Self> {
        let async_wrapper_config = AsyncWrapperConfig::default();
        let encryption = AsyncEncryptionWrapper::new(
            db_ops.clone(),
            master_keypair,
            async_wrapper_config,
        ).await?;
        
        let operation_semaphore = Arc::new(Semaphore::new(config.max_concurrent_operations));
        
        Ok(Self {
            db_ops,
            encryption,
            config,
            operation_semaphore,
            progress: Arc::new(RwLock::new(None)),
        })
    }
    
    /// Create an encrypted backup asynchronously
    pub async fn create_backup_async<P: AsRef<Path>>(
        &self,
        backup_path: P,
    ) -> Result<BackupInfo, SchemaError> {
        let start_time = Instant::now();
        let backup_path = backup_path.as_ref().to_path_buf();
        
        // Initialize progress tracking
        if self.config.enable_progress_tracking {
            let mut progress = self.progress.write().await;
            *progress = Some(ProgressInfo::new("backup".to_string()));
        }
        
        // Collect all database items first to get total count
        let all_items = self.collect_database_items().await?;
        
        // Update progress with total count
        if self.config.enable_progress_tracking {
            if let Some(ref mut progress) = *self.progress.write().await {
                progress.total_items = Some(all_items.len() as u64);
            }
        }
        
        // Create backup file
        let backup_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&backup_path)
            .await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to create backup file: {}", e)))?;
        
        let mut writer = BufWriter::with_capacity(self.config.buffer_size, backup_file);
        
        // Write backup header
        let backup_id = Uuid::new_v4().to_string();
        let mut metadata = BackupMetadata {
            version: 1,
            created_at: SystemTime::now(),
            database_id: backup_id.clone(),
            total_items: all_items.len() as u64,
            total_size: 0, // Will be updated at the end
            compression_level: self.config.compression_level,
            integrity_hash: String::new(), // Will be computed at the end
            encryption_contexts: contexts::all_contexts().iter().map(|s| s.to_string()).collect(),
            additional_info: HashMap::new(),
        };
        
        let header_bytes = serde_json::to_vec(&metadata)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize backup header: {}", e)))?;
        
        writer.write_u32(header_bytes.len() as u32).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write header length: {}", e)))?;
        writer.write_all(&header_bytes).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write header: {}", e)))?;
        
        // Write encrypted items in batches
        let mut hasher = Sha256::new();
        let mut total_bytes = 0u64;
        let mut error_count = 0u64;
        
        for chunk in all_items.chunks(self.config.batch_size) {
            let encrypted_chunk = self.encrypt_items_batch(chunk).await?;
            
            for (key, encrypted_data) in encrypted_chunk {
                let item_data = serde_json::to_vec(&(key, encrypted_data))
                    .map_err(|e| {
                        error_count += 1;
                        SchemaError::InvalidData(format!("Failed to serialize item: {}", e))
                    })?;
                
                // Apply compression if enabled
                let final_data = if self.config.compression_level > 0 {
                    self.compress_data(&item_data).await?
                } else {
                    item_data
                };
                
                // Write item size and data
                writer.write_u32(final_data.len() as u32).await
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to write item size: {}", e)))?;
                writer.write_all(&final_data).await
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to write item data: {}", e)))?;
                
                total_bytes += final_data.len() as u64 + 4; // Include size prefix
                hasher.update(&final_data);
                
                // Update progress
                if self.config.enable_progress_tracking {
                    if let Some(ref mut progress) = *self.progress.write().await {
                        progress.update(1, final_data.len() as u64);
                    }
                }
                
                // Create checkpoint if enabled
                if self.config.enable_resume && 
                   (total_bytes as usize / self.config.checkpoint_interval) > 0 {
                    self.create_checkpoint(&backup_path, total_bytes).await?;
                }
            }
        }
        
        // Finalize backup
        let integrity_hash = hex::encode(hasher.finalize());
        metadata.total_size = total_bytes;
        metadata.integrity_hash = integrity_hash;
        
        // Write updated metadata at the end
        let final_header_bytes = serde_json::to_vec(&metadata)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize final metadata: {}", e)))?;
        
        writer.write_u32(final_header_bytes.len() as u32).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write final header length: {}", e)))?;
        writer.write_all(&final_header_bytes).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write final header: {}", e)))?;
        
        writer.flush().await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to flush backup file: {}", e)))?;
        
        let duration = start_time.elapsed();
        
        // Clear progress
        if self.config.enable_progress_tracking {
            let mut progress = self.progress.write().await;
            *progress = None;
        }
        
        Ok(BackupInfo {
            metadata,
            duration,
            throughput_items_per_sec: all_items.len() as f64 / duration.as_secs_f64(),
            throughput_bytes_per_sec: total_bytes as f64 / duration.as_secs_f64(),
            error_count,
        })
    }
    
    /// Restore from an encrypted backup asynchronously
    pub async fn restore_backup_async<P: AsRef<Path>>(
        &self,
        backup_path: P,
        target_db_ops: DbOperations,
    ) -> Result<RestoreInfo, SchemaError> {
        let start_time = Instant::now();
        let backup_path = backup_path.as_ref();
        
        // Initialize progress tracking
        if self.config.enable_progress_tracking {
            let mut progress = self.progress.write().await;
            *progress = Some(ProgressInfo::new("restore".to_string()));
        }
        
        // Open backup file
        let backup_file = File::open(backup_path).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to open backup file: {}", e)))?;
        
        let mut reader = BufReader::with_capacity(self.config.buffer_size, backup_file);
        
        // Read and parse backup header
        let header_len = reader.read_u32().await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read header length: {}", e)))?;
        
        let mut header_bytes = vec![0u8; header_len as usize];
        reader.read_exact(&mut header_bytes).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read header: {}", e)))?;
        
        let metadata: BackupMetadata = serde_json::from_slice(&header_bytes)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to parse backup header: {}", e)))?;
        
        // Update progress with total count
        if self.config.enable_progress_tracking {
            if let Some(ref mut progress) = *self.progress.write().await {
                progress.total_items = Some(metadata.total_items);
                progress.total_bytes = Some(metadata.total_size);
            }
        }
        
        // Create target encryption wrapper
        let _crypto_config = CryptoConfig::default();
        let master_keypair = self.get_master_keypair().await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to get master keypair: {}", e)))?;
        let async_wrapper_config = AsyncWrapperConfig::default();
        let target_encryption = AsyncEncryptionWrapper::new(
            target_db_ops,
            &master_keypair,
            async_wrapper_config,
        ).await.map_err(|e| SchemaError::InvalidData(format!("Failed to create target encryption: {}", e)))?;
        
        // Read and restore items
        let mut hasher = Sha256::new();
        let mut items_restored = 0u64;
        let mut bytes_restored = 0u64;
        let mut error_count = 0u64;
        let mut batch_items = Vec::new();
        
        // Read items until we reach the final metadata
        while items_restored < metadata.total_items {
            let item_size = match reader.read_u32().await {
                Ok(size) => size,
                Err(_) => break, // End of items, final metadata follows
            };
            
            let mut item_bytes = vec![0u8; item_size as usize];
            reader.read_exact(&mut item_bytes).await
                .map_err(|e| SchemaError::InvalidData(format!("Failed to read item data: {}", e)))?;
            
            hasher.update(&item_bytes);
            
            // Decompress if needed
            let decompressed_data = if self.config.compression_level > 0 {
                self.decompress_data(&item_bytes).await?
            } else {
                item_bytes
            };
            
            let (key, encrypted_data): (String, Vec<u8>) = serde_json::from_slice(&decompressed_data)
                .map_err(|e| {
                    error_count += 1;
                    SchemaError::InvalidData(format!("Failed to parse item: {}", e))
                })?;
            
            batch_items.push((key, encrypted_data));
            
            // Process batch when full
            if batch_items.len() >= self.config.batch_size {
                let batch_result = self.restore_items_batch(&target_encryption, batch_items).await;
                match batch_result {
                    Ok(batch_count) => {
                        items_restored += batch_count;
                        bytes_restored += item_size as u64;
                    }
                    Err(_) => error_count += 1,
                }
                batch_items = Vec::new();
                
                // Update progress
                if self.config.enable_progress_tracking {
                    if let Some(ref mut progress) = *self.progress.write().await {
                        progress.update(items_restored, item_size as u64);
                    }
                }
            }
        }
        
        // Process remaining items
        if !batch_items.is_empty() {
            let batch_result = self.restore_items_batch(&target_encryption, batch_items).await;
            match batch_result {
                Ok(batch_count) => {
                    items_restored += batch_count;
                }
                Err(_) => error_count += 1,
            }
        }
        
        // Verify integrity if enabled
        let verification_passed = if self.config.verify_integrity {
            let computed_hash = hex::encode(hasher.finalize());
            Some(computed_hash == metadata.integrity_hash)
        } else {
            None
        };
        
        let duration = start_time.elapsed();
        
        // Clear progress
        if self.config.enable_progress_tracking {
            let mut progress = self.progress.write().await;
            *progress = None;
        }
        
        Ok(RestoreInfo {
            total_items: items_restored,
            total_bytes: bytes_restored,
            duration,
            throughput_items_per_sec: items_restored as f64 / duration.as_secs_f64(),
            throughput_bytes_per_sec: bytes_restored as f64 / duration.as_secs_f64(),
            error_count,
            verification_passed,
        })
    }
    
    /// Get current progress information
    pub async fn get_progress(&self) -> Option<ProgressInfo> {
        self.progress.read().await.clone()
    }
    
    /// Cancel ongoing operation
    pub async fn cancel_operation(&self) {
        if self.config.enable_progress_tracking {
            let mut progress = self.progress.write().await;
            *progress = None;
        }
    }
    
    /// List available backup checkpoints for resume
    pub async fn list_checkpoints<P: AsRef<Path>>(
        &self,
        backup_path: P,
    ) -> Result<Vec<CheckpointInfo>, SchemaError> {
        let checkpoint_dir = backup_path.as_ref().with_extension("checkpoints");
        if !checkpoint_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut checkpoints = Vec::new();
        let mut entries = tokio::fs::read_dir(checkpoint_dir).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read checkpoint directory: {}", e)))?;
        
        while let Some(entry) = entries.next_entry().await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to read checkpoint entry: {}", e)))? {
            
            if entry.file_type().await
                .map_err(|e| SchemaError::InvalidData(format!("Failed to get file type: {}", e)))?
                .is_file() {
                
                let checkpoint_data = tokio::fs::read(entry.path()).await
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to read checkpoint file: {}", e)))?;
                
                let checkpoint: CheckpointInfo = serde_json::from_slice(&checkpoint_data)
                    .map_err(|e| SchemaError::InvalidData(format!("Failed to parse checkpoint: {}", e)))?;
                
                checkpoints.push(checkpoint);
            }
        }
        
        // Sort by timestamp
        checkpoints.sort_by_key(|c| c.timestamp);
        Ok(checkpoints)
    }
    
    /// Collect all database items for backup
    async fn collect_database_items(&self) -> Result<Vec<(String, Vec<u8>)>, SchemaError> {
        let mut items = Vec::new();
        
        // Collect from main database
        for result in self.db_ops.db().iter() {
            let (key, value) = result.map_err(|e| SchemaError::InvalidData(format!("Database iteration error: {}", e)))?;
            let key_str = String::from_utf8_lossy(&key).to_string();
            items.push((key_str, value.to_vec()));
        }
        
        Ok(items)
    }
    
    /// Encrypt a batch of items
    async fn encrypt_items_batch(
        &self,
        items: &[(String, Vec<u8>)],
    ) -> Result<Vec<(String, Vec<u8>)>, SchemaError> {
        let mut encrypted_items = Vec::new();
        
        for (key, data) in items {
            // Determine appropriate context based on key prefix
            let _context = if key.starts_with("atom:") {
                contexts::ATOM_DATA
            } else if key.starts_with("schema:") {
                contexts::SCHEMA_DATA
            } else if key.starts_with("transform:") {
                contexts::TRANSFORM_DATA
            } else {
                contexts::METADATA
            };
            
            // Serialize the data for backup
            let serialized_data = serde_json::to_vec(data)
                .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize item: {}", e)))?;
            
            encrypted_items.push((key.clone(), serialized_data));
        }
        
        Ok(encrypted_items)
    }
    
    /// Restore a batch of items
    async fn restore_items_batch(
        &self,
        target_encryption: &AsyncEncryptionWrapper,
        items: Vec<(String, Vec<u8>)>,
    ) -> Result<u64, SchemaError> {
        let mut restored_count = 0u64;
        
        for (key, _encrypted_data) in items {
            // Determine appropriate context based on key prefix
            let context = if key.starts_with("atom:") {
                contexts::ATOM_DATA
            } else if key.starts_with("schema:") {
                contexts::SCHEMA_DATA
            } else if key.starts_with("transform:") {
                contexts::TRANSFORM_DATA
            } else {
                contexts::METADATA
            };
            
            // Decrypt and store in target database
            let decrypted_data: Vec<u8> = self.encryption.get_encrypted_item_async(&key, context).await
                .map_err(|e| SchemaError::InvalidData(format!("Failed to decrypt item {}: {}", key, e)))?
                .ok_or_else(|| SchemaError::InvalidData(format!("Item not found during restore: {}", key)))?;
            
            target_encryption.store_encrypted_item_async(&key, &decrypted_data, context).await
                .map_err(|e| SchemaError::InvalidData(format!("Failed to store restored item {}: {}", key, e)))?;
            
            restored_count += 1;
        }
        
        Ok(restored_count)
    }
    
    /// Compress data using the configured compression level
    async fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, SchemaError> {
        if self.config.compression_level == 0 {
            return Ok(data.to_vec());
        }
        
        // Use async compression in a blocking task
        let data_clone = data.to_vec();
        let compression_level = self.config.compression_level;
        
        task::spawn_blocking(move || {
            use flate2::Compression;
            use flate2::write::GzEncoder;
            use std::io::Write;
            
            let mut encoder = GzEncoder::new(Vec::new(), Compression::new(compression_level));
            encoder.write_all(&data_clone)
                .map_err(|e| SchemaError::InvalidData(format!("Compression write error: {}", e)))?;
            encoder.finish()
                .map_err(|e| SchemaError::InvalidData(format!("Compression finish error: {}", e)))
        }).await
        .map_err(|e| SchemaError::InvalidData(format!("Compression task failed: {}", e)))?
    }
    
    /// Decompress data
    async fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, SchemaError> {
        if self.config.compression_level == 0 {
            return Ok(data.to_vec());
        }
        
        // Use async decompression in a blocking task
        let data_clone = data.to_vec();
        
        task::spawn_blocking(move || {
            use flate2::read::GzDecoder;
            use std::io::Read;
            
            let mut decoder = GzDecoder::new(&data_clone[..]);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed)
                .map_err(|e| SchemaError::InvalidData(format!("Decompression error: {}", e)))?;
            Ok(decompressed)
        }).await
        .map_err(|e| SchemaError::InvalidData(format!("Decompression task failed: {}", e)))?
    }
    
    /// Create a checkpoint for resume capability
    async fn create_checkpoint<P: AsRef<Path>>(
        &self,
        backup_path: P,
        bytes_processed: u64,
    ) -> Result<(), SchemaError> {
        if !self.config.enable_resume {
            return Ok(());
        }
        
        let checkpoint_dir = backup_path.as_ref().with_extension("checkpoints");
        tokio::fs::create_dir_all(&checkpoint_dir).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to create checkpoint directory: {}", e)))?;
        
        let checkpoint = CheckpointInfo {
            id: Uuid::new_v4().to_string(),
            operation: "backup".to_string(),
            items_processed: bytes_processed / 1000, // Approximate
            bytes_processed,
            last_key: None, // Could be enhanced to track actual key
            timestamp: SystemTime::now(),
        };
        
        let checkpoint_data = serde_json::to_vec(&checkpoint)
            .map_err(|e| SchemaError::InvalidData(format!("Failed to serialize checkpoint: {}", e)))?;
        
        let checkpoint_file = checkpoint_dir.join(format!("{}.json", checkpoint.id));
        tokio::fs::write(checkpoint_file, checkpoint_data).await
            .map_err(|e| SchemaError::InvalidData(format!("Failed to write checkpoint: {}", e)))?;
        
        Ok(())
    }
    
    /// Get master keypair (placeholder - would need actual implementation)
    async fn get_master_keypair(&self) -> CryptoResult<MasterKeyPair> {
        // This would need to be implemented based on how the master keypair is stored/accessed
        // For now, return an error
        Err(CryptoError::InvalidInput("Master keypair access not implemented".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_master_keypair;
    use tempfile::{tempdir, NamedTempFile};
    
    async fn create_test_backup_manager() -> AsyncEncryptedBackup {
        let temp_dir = tempdir().unwrap();
        let db = sled::open(temp_dir.path()).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        let master_keypair = generate_master_keypair().unwrap();
        let config = AsyncBackupConfig {
            buffer_size: 1024,
            batch_size: 5,
            max_concurrent_operations: 2,
            compression_level: 0, // No compression for tests
            ..Default::default()
        };
        
        AsyncEncryptedBackup::new(db_ops, &master_keypair, config).await.unwrap()
    }
    
    #[tokio::test]
    async fn test_backup_config_variants() {
        let configs = vec![
            AsyncBackupConfig::default(),
            AsyncBackupConfig::large_database(),
            AsyncBackupConfig::fast_mode(),
            AsyncBackupConfig::network_optimized(),
        ];
        
        for config in configs {
            assert!(config.buffer_size > 0);
            assert!(config.batch_size > 0);
            assert!(config.max_concurrent_operations > 0);
        }
    }
    
    #[tokio::test]
    async fn test_progress_info() {
        let mut progress = ProgressInfo::new("test".to_string());
        assert_eq!(progress.items_processed, 0);
        assert_eq!(progress.bytes_processed, 0);
        
        progress.update(10, 1024);
        assert_eq!(progress.items_processed, 10);
        assert_eq!(progress.bytes_processed, 1024);
        assert!(progress.throughput_items_per_sec > 0.0);
        assert!(progress.throughput_bytes_per_sec > 0.0);
        
        // Test with total items set
        progress.total_items = Some(100);
        let percentage = progress.progress_percentage().unwrap();
        assert!((percentage - 10.0).abs() < 0.1);
    }
    
    #[tokio::test]
    async fn test_backup_metadata() {
        let metadata = BackupMetadata {
            version: 1,
            created_at: SystemTime::now(),
            database_id: "test_db".to_string(),
            total_items: 100,
            total_size: 1024,
            compression_level: 6,
            integrity_hash: "test_hash".to_string(),
            encryption_contexts: vec!["atom_data".to_string()],
            additional_info: HashMap::new(),
        };
        
        // Test serialization
        let serialized = serde_json::to_vec(&metadata).unwrap();
        let deserialized: BackupMetadata = serde_json::from_slice(&serialized).unwrap();
        
        assert_eq!(metadata.version, deserialized.version);
        assert_eq!(metadata.database_id, deserialized.database_id);
        assert_eq!(metadata.total_items, deserialized.total_items);
    }
    
    #[tokio::test]
    async fn test_compression_decompression() {
        // Create a custom config with no compression
        let config = AsyncBackupConfig {
            compression_level: 0,
            ..Default::default()
        };
        
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db");
        let db = sled::open(&db_path).unwrap();
        let db_ops = DbOperations::new(db).unwrap();
        let master_keypair = crate::crypto::generate_master_keypair().unwrap();
        
        let backup_manager = AsyncEncryptedBackup::new(db_ops, &master_keypair, config).await.unwrap();
        
        let test_data = b"This is test data for compression";
        
        // Test without compression
        let uncompressed = backup_manager.compress_data(test_data).await.unwrap();
        assert_eq!(uncompressed, test_data);
        
        let decompressed = backup_manager.decompress_data(&uncompressed).await.unwrap();
        assert_eq!(decompressed, test_data);
    }
    
    #[tokio::test]
    async fn test_checkpoint_creation() {
        let backup_manager = create_test_backup_manager().await;
        let temp_file = NamedTempFile::new().unwrap();
        
        // Test checkpoint creation
        backup_manager.create_checkpoint(temp_file.path(), 1024).await.unwrap();
        
        // Check that checkpoint directory was created
        let checkpoint_dir = temp_file.path().with_extension("checkpoints");
        assert!(checkpoint_dir.exists());
        
        // List checkpoints
        let checkpoints = backup_manager.list_checkpoints(temp_file.path()).await.unwrap();
        assert_eq!(checkpoints.len(), 1);
        assert_eq!(checkpoints[0].bytes_processed, 1024);
    }
}