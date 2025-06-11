//! Utility functions for protocol validation

use anyhow::Result;
use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique correlation ID for tracking
pub fn generate_correlation_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    
    let mut hasher = Sha256::new();
    hasher.update(timestamp.to_be_bytes());
    hasher.update(uuid::Uuid::new_v4().as_bytes());
    
    let hash = hasher.finalize();
    hex::encode(&hash[..8])
}

/// Convert bytes to hexadecimal string
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

/// Convert hexadecimal string to bytes
pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    hex::decode(hex).map_err(|e| anyhow::anyhow!("Invalid hex string: {}", e))
}

/// Generate a test nonce in UUID4 format
pub fn generate_test_nonce() -> String {
    uuid::Uuid::new_v4().to_string().replace('-', "")
}

/// Get current Unix timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Validate that a string is a valid UUID4
pub fn is_valid_uuid4(s: &str) -> bool {
    uuid::Uuid::parse_str(s).is_ok()
}

/// Format duration in milliseconds as human readable
pub fn format_duration_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{}ms", ms)
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{:.1}m", ms as f64 / 60_000.0)
    }
}

/// Calculate percentage
pub fn calculate_percentage(part: usize, total: usize) -> f64 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_correlation_id() {
        let id1 = generate_correlation_id();
        let id2 = generate_correlation_id();
        
        assert_ne!(id1, id2);
        assert_eq!(id1.len(), 16); // 8 bytes as hex
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "0123456789abcdef");
        
        let decoded = hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_generate_test_nonce() {
        let nonce = generate_test_nonce();
        assert_eq!(nonce.len(), 32); // UUID4 without dashes
        assert!(!nonce.contains('-'));
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration_ms(500), "500ms");
        assert_eq!(format_duration_ms(1500), "1.5s");
        assert_eq!(format_duration_ms(120000), "2.0m");
    }

    #[test]
    fn test_calculate_percentage() {
        assert_eq!(calculate_percentage(0, 0), 0.0);
        assert_eq!(calculate_percentage(50, 100), 50.0);
        assert_eq!(calculate_percentage(1, 3), 33.333333333333336);
    }
}