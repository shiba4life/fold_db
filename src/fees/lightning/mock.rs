use std::time::Duration;
use async_trait::async_trait;
use chrono::Utc;
use uuid::Uuid;

use crate::fees::{Error, LightningPaymentRequest, PaymentStatus};
use super::client::LightningClient;

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
        // Generate a mock payment hash
        let payment_hash = Uuid::new_v4().to_string();
        
        Ok(LightningPaymentRequest {
            amount,
            invoice: format!("mock_invoice_{}", payment_hash),
            expiry: Utc::now() + chrono::Duration::from_std(expiry).map_err(|e| Error::Internal(e.to_string()))?,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_invoice_creation() {
        let client = MockLightningClient::new();
        let result = client
            .create_invoice(
                100,
                "Test payment".to_string(),
                Duration::from_secs(3600),
                false,
            )
            .await;

        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.amount, 100);
        assert!(invoice.invoice.starts_with("mock_invoice_"));
    }

    #[tokio::test]
    async fn test_mock_payment_status() {
        let client = MockLightningClient::new();

        // Test valid mock invoice
        let status = client.check_payment("mock_invoice_123").await;
        assert!(status.is_ok());
        assert!(matches!(status.unwrap(), PaymentStatus::Settled));

        // Test invalid invoice format
        let status = client.check_payment("invalid_invoice").await;
        assert!(status.is_err());
        assert!(matches!(status.unwrap_err(), Error::InvalidInvoice(_)));
    }

    #[tokio::test]
    async fn test_mock_node_connection() {
        let client = MockLightningClient::new();
        let connected = client.check_node_connection().await.unwrap();
        assert!(connected);
    }
}
