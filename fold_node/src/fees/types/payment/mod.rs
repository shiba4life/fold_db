use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

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
