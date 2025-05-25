use std::time::Duration;
use fold_node::fees::lightning::mock::MockLightningClient;
use fold_node::fees::lightning::LightningClient;
use fold_node::fees::{Error, PaymentStatus};

#[tokio::test]
async fn test_mock_invoice_creation() {
    let client = MockLightningClient::new();
    let result = client
        .create_invoice(100, "Test payment".to_string(), Duration::from_secs(3600), false)
        .await;

    assert!(result.is_ok());
    let invoice = result.unwrap();
    assert_eq!(invoice.amount, 100);
    assert!(invoice.invoice.starts_with("mock_invoice_"));
}

#[tokio::test]
async fn test_mock_payment_status() {
    let client = MockLightningClient::new();

    let status = client.check_payment("mock_invoice_123").await;
    assert!(status.is_ok());
    assert!(matches!(status.unwrap(), PaymentStatus::Settled));

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
