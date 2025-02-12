use crate::fees::{Error, LightningPaymentRequest, PaymentStatus};
use async_trait::async_trait;
use std::fmt::Debug;
use std::time::Duration;

#[async_trait]
pub trait LightningClient: Send + Sync + Debug {
    /// Creates a new invoice for the specified amount
    async fn create_invoice(
        &self,
        amount: u64,
        memo: String,
        expiry: Duration,
        hold_invoice: bool,
    ) -> Result<LightningPaymentRequest, Error>;

    /// Checks the payment status of an invoice
    async fn check_payment(&self, invoice: &str) -> Result<PaymentStatus, Error>;

    /// Cancels an existing invoice
    async fn cancel_invoice(&self, invoice: &str) -> Result<(), Error>;

    /// Gets the node's public key
    async fn get_node_pubkey(&self) -> Result<String, Error>;

    /// Checks if the node is connected and operational
    async fn check_node_connection(&self) -> Result<bool, Error>;
}
