use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPaymentConfig {
    pub base_multiplier: f64,
    pub trust_distance_scaling: TrustDistanceScaling,
    pub min_payment: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustDistanceScaling {
    Linear {
        slope: f64,
        intercept: f64,
        min_factor: f64,
    },
    Exponential {
        base: f64,
        scale: f64,
        min_factor: f64,
    },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningPaymentRequest {
    pub amount: u64,
    pub invoice: String,
    pub expiry: DateTime<Utc>,
    pub payment_hash: String,
    pub hold_invoice: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentState {
    pub invoice_id: String,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub last_checked: DateTime<Utc>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PaymentStatus {
    Pending,
    Settled,
    Expired,
    Failed,
    PartiallyPaid(u64),
    Cancelled,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid payment amount: {0}")]
    InvalidAmount(String),
    
    #[error("Invalid trust distance: {0}")]
    InvalidTrustDistance(String),
    
    #[error("Lightning node error: {0}")]
    LightningNode(String),
    
    #[error("Invalid invoice: {0}")]
    InvalidInvoice(String),
    
    #[error("Payment timeout")]
    PaymentTimeout,
    
    #[error("Payment verification failed: {0}")]
    PaymentVerification(String),
    
    #[error("Payment expired")]
    PaymentExpired,
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl FieldPaymentConfig {
    pub fn new(
        base_multiplier: f64,
        trust_distance_scaling: TrustDistanceScaling,
        min_payment: Option<u64>,
    ) -> Result<Self, Error> {
        if base_multiplier <= 0.0 {
            return Err(Error::InvalidAmount(
                "Base multiplier must be positive".to_string(),
            ));
        }

        // Validate trust distance scaling parameters
        match &trust_distance_scaling {
            TrustDistanceScaling::Linear { slope: _, intercept: _, min_factor } => {
                if *min_factor < 1.0 {
                    return Err(Error::InvalidAmount(
                        "Minimum scaling factor must be >= 1.0".to_string(),
                    ));
                }
            }
            TrustDistanceScaling::Exponential { base, scale: _, min_factor } => {
                if *base <= 0.0 {
                    return Err(Error::InvalidAmount(
                        "Exponential base must be positive".to_string(),
                    ));
                }
                if *min_factor < 1.0 {
                    return Err(Error::InvalidAmount(
                        "Minimum scaling factor must be >= 1.0".to_string(),
                    ));
                }
            }
            TrustDistanceScaling::None => {}
        }

        Ok(Self {
            base_multiplier,
            trust_distance_scaling,
            min_payment,
        })
    }
}

impl PaymentState {
    pub fn is_final(&self) -> bool {
        matches!(
            self.status,
            PaymentStatus::Settled | PaymentStatus::Failed | PaymentStatus::Cancelled
        )
    }

    pub fn can_retry(&self, max_retries: u32) -> bool {
        !self.is_final() && self.retry_count < max_retries
    }
}

impl Default for FieldPaymentConfig {
    fn default() -> Self {
        Self::new(
            1.0,
            TrustDistanceScaling::None,
            None,
        ).expect("Default payment config should be valid")
    }
}
