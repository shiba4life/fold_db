use std::time::Duration;
use fold_node::fees::payment_config::{GlobalPaymentConfig, SchemaPaymentConfig, MarketRate};

#[test]
fn test_global_payment_config_validation() {
    let config = GlobalPaymentConfig::new(100, Duration::from_secs(3600), 3, Duration::from_secs(7200));
    assert!(config.is_ok());

    let config = GlobalPaymentConfig::new(0, Duration::from_secs(3600), 3, Duration::from_secs(7200));
    assert!(config.is_err());

    let config = GlobalPaymentConfig::new(100, Duration::from_secs(0), 3, Duration::from_secs(7200));
    assert!(config.is_err());

    let config = GlobalPaymentConfig::new(100, Duration::from_secs(3600), 0, Duration::from_secs(7200));
    assert!(config.is_err());
}

#[test]
fn test_schema_payment_config_validation() {
    let config = SchemaPaymentConfig::new(1.5, 10);
    assert!(config.is_ok());

    let config = SchemaPaymentConfig::new(0.0, 10);
    assert!(config.is_err());
    let config = SchemaPaymentConfig::new(-1.0, 10);
    assert!(config.is_err());
}

#[test]
fn test_market_rate_staleness() {
    let mut rate = MarketRate::new(100);

    assert!(!rate.is_stale(Duration::from_secs(60)));

    rate.last_updated -= chrono::Duration::seconds(120);

    assert!(rate.is_stale(Duration::from_secs(60)));

    rate.update(150);
    assert!(!rate.is_stale(Duration::from_secs(60)));
}
