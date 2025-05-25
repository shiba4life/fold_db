use std::time::Duration;
use fold_node::fees::payment_calculator::{calculate_field_payment, calculate_total_query_payment};
use fold_node::fees::{FieldPaymentConfig, GlobalPaymentConfig, MarketRate, SchemaPaymentConfig, TrustDistanceScaling, Error};

fn setup_test_configs() -> (GlobalPaymentConfig, MarketRate, SchemaPaymentConfig) {
    let global_config = GlobalPaymentConfig::new(
        50,
        Duration::from_secs(3600),
        3,
        Duration::from_secs(7200),
    )
    .unwrap();

    let market_rate = MarketRate::new(100);
    let schema_payment = SchemaPaymentConfig::new(1.5, 10).unwrap();

    (global_config, market_rate, schema_payment)
}

#[test]
fn test_calculate_field_payment() {
    let (global_config, market_rate, schema_payment) = setup_test_configs();

    let field_payment = FieldPaymentConfig::new(
        2.0,
        TrustDistanceScaling::Linear {
            slope: 0.5,
            intercept: 1.0,
            min_factor: 1.0,
        },
        None,
    )
    .unwrap();

    let payment = calculate_field_payment(
        &global_config,
        &market_rate,
        &schema_payment,
        &field_payment,
        2.0,
    )
    .unwrap();

    assert_eq!(payment, 600);

    let field_payment = FieldPaymentConfig::new(
        2.0,
        TrustDistanceScaling::Exponential {
            base: 2.0,
            scale: 0.5,
            min_factor: 1.0,
        },
        None,
    )
    .unwrap();

    let payment = calculate_field_payment(
        &global_config,
        &market_rate,
        &schema_payment,
        &field_payment,
        3.0,
    )
    .unwrap();

    assert!(payment > 800 && payment < 900);
}

#[test]
fn test_minimum_thresholds() {
    let (global_config, market_rate, schema_payment) = setup_test_configs();

    let field_payment = FieldPaymentConfig::new(
        0.1,
        TrustDistanceScaling::None,
        Some(1000),
    )
    .unwrap();

    let payment = calculate_field_payment(
        &global_config,
        &market_rate,
        &schema_payment,
        &field_payment,
        1.0,
    )
    .unwrap();

    assert_eq!(payment, 1000);
}

#[test]
fn test_total_query_payment() {
    let (global_config, market_rate, schema_payment) = setup_test_configs();

    let field1 = FieldPaymentConfig::new(2.0, TrustDistanceScaling::None, None).unwrap();
    let field2 = FieldPaymentConfig::new(1.5, TrustDistanceScaling::None, None).unwrap();
    let fields = vec![(field1, 1.0), (field2, 1.0)];

    let total = calculate_total_query_payment(&global_config, &market_rate, &schema_payment, &fields).unwrap();

    assert_eq!(total, 525);
}

#[test]
fn test_negative_trust_distance() {
    let (global_config, market_rate, schema_payment) = setup_test_configs();

    let field_payment = FieldPaymentConfig::new(2.0, TrustDistanceScaling::None, None).unwrap();

    let result = calculate_field_payment(
        &global_config,
        &market_rate,
        &schema_payment,
        &field_payment,
        -1.0,
    );

    assert!(matches!(result, Err(Error::InvalidTrustDistance(_))));
}
