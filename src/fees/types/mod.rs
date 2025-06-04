pub mod config;
pub mod lightning;
pub mod payment;

// Re-export commonly used types
pub use config::{FieldPaymentConfig, TrustDistanceScaling};
