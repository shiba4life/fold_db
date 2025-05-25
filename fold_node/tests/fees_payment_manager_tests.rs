use std::time::Duration;
use fold_node::fees::payment_manager::PaymentManager;
use fold_node::fees::lightning::mock::MockLightningClient;
use fold_node::fees::{GlobalPaymentConfig, PaymentStatus};

async fn setup_test_manager() -> PaymentManager {
    let config = GlobalPaymentConfig::new(
        50,
        Duration::from_secs(3600),
        3,
        Duration::from_secs(7200),
    )
    .unwrap();

    let lightning_client = Box::new(MockLightningClient::new());
    PaymentManager::new(config, lightning_client)
}

#[tokio::test]
async fn test_invoice_generation() {
    let manager = setup_test_manager().await;

    let result = manager
        .generate_invoice(100, "Test payment".to_string(), false)
        .await;

    assert!(result.is_ok());
    let invoice = result.unwrap();
    assert_eq!(invoice.amount, 100);
}

#[tokio::test]
async fn test_payment_verification() {
    let manager = setup_test_manager().await;

    let invoice = manager
        .generate_invoice(100, "Test payment".to_string(), false)
        .await
        .unwrap();

    let result = manager.verify_payment(&invoice.payment_hash).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_payment_cancellation() {
    let manager = setup_test_manager().await;

    let invoice = manager
        .generate_invoice(100, "Test payment".to_string(), false)
        .await
        .unwrap();

    let result = manager.cancel_payment(&invoice.payment_hash).await;
    assert!(result.is_ok());

    let status = manager
        .get_payment_status(&invoice.payment_hash)
        .await
        .unwrap();
    assert!(matches!(status, PaymentStatus::Cancelled));
}

#[tokio::test]
async fn test_expired_invoice_cleanup() {
    let config = GlobalPaymentConfig::new(50, Duration::from_secs(1), 3, Duration::from_secs(1)).unwrap();
    let lightning_client = Box::new(MockLightningClient::new());
    let manager = PaymentManager::new(config, lightning_client);

    let invoice = manager.generate_invoice(100, "Test payment".to_string(), false).await.unwrap();
    tokio::time::sleep(Duration::from_secs(2)).await;
    manager.cleanup_expired_invoices().await.unwrap();
    let status = manager.get_payment_status(&invoice.payment_hash).await.unwrap();
    assert!(matches!(status, PaymentStatus::Expired));
}

