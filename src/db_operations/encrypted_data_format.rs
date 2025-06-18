//! Encrypted data format handling for database encryption wrapper
//!
//! This module provides the [`EncryptedDataFormat`] struct and implementation 
//! for serialization/deserialization of encrypted data with context information.
//! It handles the binary format used to store encrypted data in the database.

use crate::crypto::{CryptoError, CryptoResult};
use crate::datafold_node::crypto::encryption_at_rest::EncryptedData;

/// Version identifier for encrypted data format
const ENCRYPTED_DATA_VERSION: u8 = 1;

/// Magic bytes to identify encrypted data
const ENCRYPTED_DATA_MAGIC: &[u8] = b"DF_ENC";

/// Size of the encryption header (magic + version + context_len)
const ENCRYPTION_HEADER_BASE_SIZE: usize = 6 + 1 + 1; // magic + version + context_len

/// Maximum size for encryption context names
const MAX_CONTEXT_NAME_SIZE: usize = 64;

/// Encrypted data format with context information
#[derive(Debug, Clone)]
pub struct EncryptedDataFormat {
    /// Version of the encryption format
    version: u8,
    /// Encryption context used
    context: String,
    /// The actual encrypted data
    encrypted_data: EncryptedData,
}

impl EncryptedDataFormat {
    /// Create new encrypted data format
    pub fn new(context: String, encrypted_data: EncryptedData) -> CryptoResult<Self> {
        if context.len() > MAX_CONTEXT_NAME_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Context name too long: {} bytes, maximum is {}",
                context.len(),
                MAX_CONTEXT_NAME_SIZE
            )));
        }

        Ok(Self {
            version: ENCRYPTED_DATA_VERSION,
            context,
            encrypted_data,
        })
    }

    /// Get the encryption context
    pub fn context(&self) -> &str {
        &self.context
    }

    /// Get the encrypted data
    pub fn encrypted_data(&self) -> &EncryptedData {
        &self.encrypted_data
    }

    /// Get the format version
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Serialize to bytes for storage
    pub fn to_bytes(&self) -> CryptoResult<Vec<u8>> {
        let context_bytes = self.context.as_bytes();
        if context_bytes.len() > 255 {
            return Err(CryptoError::InvalidInput(
                "Context name too long for encoding".to_string(),
            ));
        }

        let encrypted_bytes = self.encrypted_data.to_bytes();
        let total_size = ENCRYPTION_HEADER_BASE_SIZE + context_bytes.len() + encrypted_bytes.len();

        let mut result = Vec::with_capacity(total_size);

        // Write header
        result.extend_from_slice(ENCRYPTED_DATA_MAGIC);
        result.push(self.version);
        result.push(context_bytes.len() as u8);
        result.extend_from_slice(context_bytes);

        // Write encrypted data
        result.extend_from_slice(&encrypted_bytes);

        Ok(result)
    }

    /// Deserialize from bytes
    pub fn from_bytes(data: &[u8]) -> CryptoResult<Self> {
        if data.len() < ENCRYPTION_HEADER_BASE_SIZE {
            return Err(CryptoError::InvalidInput(
                "Data too small to contain encryption header".to_string(),
            ));
        }

        // Check magic bytes
        if &data[0..6] != ENCRYPTED_DATA_MAGIC {
            return Err(CryptoError::InvalidInput(
                "Invalid magic bytes in encrypted data".to_string(),
            ));
        }

        // Read version
        let version = data[6];
        if version != ENCRYPTED_DATA_VERSION {
            return Err(CryptoError::InvalidInput(format!(
                "Unsupported encryption format version: {}",
                version
            )));
        }

        // Read context
        let context_len = data[7] as usize;
        if data.len() < ENCRYPTION_HEADER_BASE_SIZE + context_len {
            return Err(CryptoError::InvalidInput(
                "Data too small to contain context".to_string(),
            ));
        }

        let context_start = 8;
        let context_end = context_start + context_len;
        let context = String::from_utf8(data[context_start..context_end].to_vec())
            .map_err(|e| CryptoError::InvalidInput(format!("Invalid context UTF-8: {}", e)))?;

        // Read encrypted data
        let encrypted_start = context_end;
        let encrypted_data = EncryptedData::from_bytes(&data[encrypted_start..])?;

        Ok(Self {
            version,
            context,
            encrypted_data,
        })
    }

    /// Check if data is encrypted by examining magic bytes
    pub fn is_encrypted_data(data: &[u8]) -> bool {
        data.len() >= 6 && &data[0..6] == ENCRYPTED_DATA_MAGIC
    }

    /// Validate the integrity of encrypted data format
    pub fn validate(&self) -> CryptoResult<()> {
        // Validate version
        if self.version != ENCRYPTED_DATA_VERSION {
            return Err(CryptoError::InvalidInput(format!(
                "Invalid format version: {}",
                self.version
            )));
        }

        // Validate context
        if self.context.is_empty() {
            return Err(CryptoError::InvalidInput(
                "Context cannot be empty".to_string(),
            ));
        }

        if self.context.len() > MAX_CONTEXT_NAME_SIZE {
            return Err(CryptoError::InvalidInput(format!(
                "Context name too long: {} bytes",
                self.context.len()
            )));
        }

        // Validate context characters
        if !self.context.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(CryptoError::InvalidInput(
                "Context name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the total size when serialized
    pub fn serialized_size(&self) -> usize {
        ENCRYPTION_HEADER_BASE_SIZE + self.context.len() + self.encrypted_data.to_bytes().len()
    }
}

/// Utilities for working with encrypted data format
pub struct EncryptedDataFormatUtils;

impl EncryptedDataFormatUtils {
    /// Extract context from encrypted data without full deserialization
    pub fn extract_context(data: &[u8]) -> CryptoResult<String> {
        if data.len() < ENCRYPTION_HEADER_BASE_SIZE {
            return Err(CryptoError::InvalidInput(
                "Data too small to contain encryption header".to_string(),
            ));
        }

        // Check magic bytes
        if &data[0..6] != ENCRYPTED_DATA_MAGIC {
            return Err(CryptoError::InvalidInput(
                "Invalid magic bytes in encrypted data".to_string(),
            ));
        }

        // Read context length
        let context_len = data[7] as usize;
        if data.len() < ENCRYPTION_HEADER_BASE_SIZE + context_len {
            return Err(CryptoError::InvalidInput(
                "Data too small to contain context".to_string(),
            ));
        }

        // Extract context
        let context_start = 8;
        let context_end = context_start + context_len;
        let context = String::from_utf8(data[context_start..context_end].to_vec())
            .map_err(|e| CryptoError::InvalidInput(format!("Invalid context UTF-8: {}", e)))?;

        Ok(context)
    }

    /// Extract version from encrypted data
    pub fn extract_version(data: &[u8]) -> CryptoResult<u8> {
        if data.len() < 7 {
            return Err(CryptoError::InvalidInput(
                "Data too small to contain version".to_string(),
            ));
        }

        // Check magic bytes
        if &data[0..6] != ENCRYPTED_DATA_MAGIC {
            return Err(CryptoError::InvalidInput(
                "Invalid magic bytes in encrypted data".to_string(),
            ));
        }

        Ok(data[6])
    }

    /// Validate data format without full deserialization
    pub fn validate_format(data: &[u8]) -> CryptoResult<()> {
        if !EncryptedDataFormat::is_encrypted_data(data) {
            return Err(CryptoError::InvalidInput(
                "Data is not in encrypted format".to_string(),
            ));
        }

        // Try to extract context (this validates the header structure)
        Self::extract_context(data)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datafold_node::crypto::encryption_at_rest::EncryptionAtRest;

    fn create_test_encrypted_data() -> EncryptedData {
        // Create a mock encrypted data for testing
        let key = [0u8; 32];
        let encryptor = EncryptionAtRest::new(key).unwrap();
        encryptor.encrypt(b"test data").unwrap()
    }

    #[test]
    fn test_encrypted_data_format_creation() {
        let encrypted_data = create_test_encrypted_data();
        let context = "test_context".to_string();
        
        let format = EncryptedDataFormat::new(context.clone(), encrypted_data).unwrap();
        
        assert_eq!(format.context(), "test_context");
        assert_eq!(format.version(), ENCRYPTED_DATA_VERSION);
    }

    #[test]
    fn test_context_too_long() {
        let encrypted_data = create_test_encrypted_data();
        let long_context = "a".repeat(MAX_CONTEXT_NAME_SIZE + 1);
        
        let result = EncryptedDataFormat::new(long_context, encrypted_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let encrypted_data = create_test_encrypted_data();
        let context = "test_context".to_string();
        
        let format = EncryptedDataFormat::new(context.clone(), encrypted_data).unwrap();
        let serialized = format.to_bytes().unwrap();
        let deserialized = EncryptedDataFormat::from_bytes(&serialized).unwrap();
        
        assert_eq!(deserialized.context(), "test_context");
        assert_eq!(deserialized.version(), ENCRYPTED_DATA_VERSION);
    }

    #[test]
    fn test_is_encrypted_data() {
        let encrypted_data = create_test_encrypted_data();
        let context = "test_context".to_string();
        
        let format = EncryptedDataFormat::new(context, encrypted_data).unwrap();
        let serialized = format.to_bytes().unwrap();
        
        assert!(EncryptedDataFormat::is_encrypted_data(&serialized));
        assert!(!EncryptedDataFormat::is_encrypted_data(b"not encrypted"));
        assert!(!EncryptedDataFormat::is_encrypted_data(b"short"));
    }

    #[test]
    fn test_extract_context() {
        let encrypted_data = create_test_encrypted_data();
        let context = "test_context".to_string();
        
        let format = EncryptedDataFormat::new(context.clone(), encrypted_data).unwrap();
        let serialized = format.to_bytes().unwrap();
        
        let extracted_context = EncryptedDataFormatUtils::extract_context(&serialized).unwrap();
        assert_eq!(extracted_context, context);
    }

    #[test]
    fn test_validation() {
        let encrypted_data = create_test_encrypted_data();
        let context = "valid_context".to_string();
        
        let format = EncryptedDataFormat::new(context, encrypted_data).unwrap();
        assert!(format.validate().is_ok());
    }
}