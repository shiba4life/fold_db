use async_trait::async_trait;
use chrono::Utc;
use std::time::Duration;
use uuid::Uuid;

use super::client::LightningClient;
use crate::fees::{Error, LightningPaymentRequest, PaymentStatus};

#[derive(Debug, Default)]
pub struct MockLightningClient {
    pub node_pubkey: String,
}

impl MockLightningClient {
    pub fn new() -> Self {
        Self {
            node_pubkey: "mock-node-pubkey".to_string(),
        }
    }
}

#[async_trait]
impl LightningClient for MockLightningClient {
    async fn create_invoice(
        &self,
        amount: u64,
        memo: String,
        expiry: Duration,
        hold_invoice: bool,
    ) -> Result<LightningPaymentRequest, Error> {
        let _ = memo; // Acknowledge the parameter to avoid unused variable warning
                      // Generate a mock payment hash
        let payment_hash = Uuid::new_v4().to_string();

        Ok(LightningPaymentRequest {
            amount,
            invoice: format!("mock_invoice_{}", payment_hash),
            expiry: Utc::now()
                + chrono::Duration::from_std(expiry).map_err(|e| Error::Internal(e.to_string()))?,
            payment_hash,
            hold_invoice,
        })
    }

    async fn check_payment(&self, invoice: &str) -> Result<PaymentStatus, Error> {
        if !invoice.starts_with("mock_invoice_") {
            return Err(Error::InvalidInvoice("Invalid invoice format".to_string()));
        }

        // For testing, consider all mock invoices as settled
        Ok(PaymentStatus::Settled)
    }

    async fn cancel_invoice(&self, invoice: &str) -> Result<(), Error> {
        if !invoice.starts_with("mock_invoice_") {
            return Err(Error::InvalidInvoice("Invalid invoice format".to_string()));
        }
        Ok(())
    }

    async fn get_node_pubkey(&self) -> Result<String, Error> {
        Ok(self.node_pubkey.clone())
    }

    async fn check_node_connection(&self) -> Result<bool, Error> {
        // Mock implementation always returns connected
        Ok(true)
    }
}

