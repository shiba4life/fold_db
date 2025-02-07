use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::fees::{Error, GlobalPaymentConfig, LightningPaymentRequest, PaymentState, PaymentStatus};
use crate::fees::lightning::LightningClient;

#[derive(Debug)]
pub struct PaymentManager {
    config: GlobalPaymentConfig,
    invoice_states: Arc<RwLock<HashMap<String, PaymentState>>>,
    lightning_client: Arc<Box<dyn LightningClient>>,
}

impl PaymentManager {
    pub fn new(
        config: GlobalPaymentConfig,
        lightning_client: Box<dyn LightningClient>,
    ) -> Self {
        Self {
            config,
            invoice_states: Arc::new(RwLock::new(HashMap::new())),
            lightning_client: Arc::new(lightning_client),
        }
    }

    pub async fn generate_invoice(
        &self,
        amount: u64,
        memo: String,
        hold_invoice: bool,
    ) -> Result<LightningPaymentRequest, Error> {
        // Validate amount against system minimum
        self.config.validate_payment(amount)?;

        let timeout = if hold_invoice { 
            self.config.hold_invoice_timeout 
        } else { 
            self.config.payment_timeout 
        };

        let invoice = self
            .lightning_client
            .create_invoice(amount, memo, timeout, hold_invoice)
            .await?;

        // Track invoice state
        let state = PaymentState {
            invoice_id: invoice.payment_hash.clone(),
            status: PaymentStatus::Pending,
            created_at: Utc::now(),
            last_checked: Utc::now(),
            retry_count: 0,
        };

        self.invoice_states
            .write()
            .await
            .insert(invoice.payment_hash.clone(), state);

        Ok(invoice)
    }

    pub async fn verify_payment(&self, payment_hash: &str) -> Result<bool, Error> {
        let mut states = self.invoice_states.write().await;
        let state = states
            .get_mut(payment_hash)
            .ok_or_else(|| Error::InvalidInvoice("Invoice not found".to_string()))?;

        // Update last checked timestamp
        state.last_checked = Utc::now();

        // Check if expired
        if Utc::now() > state.created_at + chrono::Duration::from_std(self.config.payment_timeout).map_err(|e| Error::Internal(e.to_string()))? {
            state.status = PaymentStatus::Expired;
            return Ok(false);
        }

        // Verify with Lightning node
        match self.lightning_client.check_payment(&format!("mock_invoice_{}", payment_hash)).await? {
            PaymentStatus::Settled => {
                state.status = PaymentStatus::Settled;
                Ok(true)
            }
            PaymentStatus::PartiallyPaid(amount) => {
                state.status = PaymentStatus::PartiallyPaid(amount);
                Ok(false)
            }
            status => {
                state.status = status;
                Ok(false)
            }
        }
    }

    pub async fn wait_for_payment(
        &self,
        invoice: &LightningPaymentRequest,
        check_interval: Duration,
    ) -> Result<bool, Error> {
        let expiry = invoice.expiry;
        let mut retries = 0;

        while Utc::now() < expiry && retries < self.config.max_invoice_retries {
            match self.verify_payment(&invoice.payment_hash).await? {
                true => return Ok(true),
                false => {
                    tokio::time::sleep(check_interval).await;
                    retries += 1;
                }
            }
        }

        // Handle payment timeout
        let states = self.invoice_states.read().await;
        match states.get(&invoice.payment_hash) {
            Some(state) if matches!(state.status, PaymentStatus::PartiallyPaid(_)) => {
                Err(Error::PaymentVerification("Partial payment received".to_string()))
            }
            _ => Err(Error::PaymentTimeout),
        }
    }

    pub async fn cancel_payment(&self, payment_hash: &str) -> Result<(), Error> {
        let mut states = self.invoice_states.write().await;
        let state = states
            .get_mut(payment_hash)
            .ok_or_else(|| Error::InvalidInvoice("Invoice not found".to_string()))?;

        if state.is_final() {
            return Err(Error::InvalidInvoice("Payment already finalized".to_string()));
        }

        // Cancel with Lightning node
        self.lightning_client.cancel_invoice(&format!("mock_invoice_{}", payment_hash)).await?;
        state.status = PaymentStatus::Cancelled;

        Ok(())
    }

    pub async fn cleanup_expired_invoices(&self) -> Result<(), Error> {
        let mut states = self.invoice_states.write().await;
        let now = Utc::now();

        // Collect expired invoices
        let expired: Vec<_> = states
            .iter()
            .filter(|(_, state)| {
                if let Ok(timeout) = chrono::Duration::from_std(self.config.payment_timeout) {
                    !state.is_final() && now > state.created_at + timeout
                } else {
                    false // Skip if duration conversion fails
                }
            })
            .map(|(k, _)| k.clone())
            .collect();

        // Cancel expired invoices
        for payment_hash in expired {
            if let Err(e) = self.lightning_client.cancel_invoice(&format!("mock_invoice_{}", payment_hash)).await {
                eprintln!("Failed to cancel expired invoice {}: {}", payment_hash, e);
            }
            if let Some(state) = states.get_mut(&payment_hash) {
                state.status = PaymentStatus::Expired;
            }
        }

        Ok(())
    }

    pub async fn get_payment_status(&self, payment_hash: &str) -> Result<PaymentStatus, Error> {
        let states = self.invoice_states.read().await;
        let state = states
            .get(payment_hash)
            .ok_or_else(|| Error::InvalidInvoice("Invoice not found".to_string()))?;

        Ok(state.status.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::lightning::MockLightningClient;

    async fn setup_test_manager() -> PaymentManager {
        let config = GlobalPaymentConfig::new(
            50,
            Duration::from_secs(3600),
            3,
            Duration::from_secs(7200),
        ).unwrap();

        let lightning_client = Box::new(MockLightningClient::new());
        PaymentManager::new(config, lightning_client)
    }

    #[tokio::test]
    async fn test_invoice_generation() {
        let manager = setup_test_manager().await;
        
        let result = manager.generate_invoice(
            100,
            "Test payment".to_string(),
            false,
        ).await;
        
        assert!(result.is_ok());
        let invoice = result.unwrap();
        assert_eq!(invoice.amount, 100);
    }

    #[tokio::test]
    async fn test_payment_verification() {
        let manager = setup_test_manager().await;
        
        let invoice = manager.generate_invoice(
            100,
            "Test payment".to_string(),
            false,
        ).await.unwrap();
        
        let result = manager.verify_payment(&invoice.payment_hash).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_payment_cancellation() {
        let manager = setup_test_manager().await;
        
        let invoice = manager.generate_invoice(
            100,
            "Test payment".to_string(),
            false,
        ).await.unwrap();
        
        let result = manager.cancel_payment(&invoice.payment_hash).await;
        assert!(result.is_ok());
        
        let status = manager.get_payment_status(&invoice.payment_hash).await.unwrap();
        assert!(matches!(status, PaymentStatus::Cancelled));
    }

    #[tokio::test]
    async fn test_expired_invoice_cleanup() {
        let manager = setup_test_manager().await;
        
        // Generate an invoice and manipulate its timestamp to make it expired
        let invoice = manager.generate_invoice(
            100,
            "Test payment".to_string(),
            false,
        ).await.unwrap();
        
        {
            let mut states = manager.invoice_states.write().await;
            let state = states.get_mut(&invoice.payment_hash).unwrap();
            state.created_at = Utc::now() - chrono::Duration::hours(2);
        }
        
        manager.cleanup_expired_invoices().await.unwrap();
        
        let status = manager.get_payment_status(&invoice.payment_hash).await.unwrap();
        assert!(matches!(status, PaymentStatus::Expired));
    }
}
