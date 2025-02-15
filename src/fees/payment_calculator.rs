use crate::fees::{
    Error, FieldPaymentConfig, GlobalPaymentConfig, MarketRate, SchemaPaymentConfig,
    TrustDistanceScaling,
};

/// Calculates the required payment amount for accessing a specific field.
/// 
/// The payment calculation incorporates multiple factors:
/// 1. Base market rate
/// 2. Schema-level multiplier
/// 3. Field-level multiplier
/// 4. Trust distance scaling
/// 5. Minimum payment thresholds
/// 
/// The calculation follows this sequence:
/// 1. Validate trust distance
/// 2. Apply base rate and multipliers
/// 3. Calculate trust distance scaling
/// 4. Apply minimum thresholds
/// 5. Validate final payment amount
/// 
/// # Arguments
/// 
/// * `global_config` - System-wide payment configuration
/// * `market_rate` - Current market base rate
/// * `schema_payment` - Schema-level payment configuration
/// * `field_payment` - Field-specific payment configuration
/// * `trust_distance` - Trust distance from requester to data owner
/// 
/// # Returns
/// 
/// A Result containing the calculated payment amount in satoshis
/// 
/// # Errors
/// 
/// Returns an Error if:
/// - The trust distance is negative
/// - The base payment calculation overflows
/// - The scaling factor calculation fails
/// - The final payment amount is invalid
pub fn calculate_field_payment(
    global_config: &GlobalPaymentConfig,
    market_rate: &MarketRate,
    schema_payment: &SchemaPaymentConfig,
    field_payment: &FieldPaymentConfig,
    trust_distance: f64,
) -> Result<u64, Error> {
    // Validate trust distance
    if trust_distance < 0.0 {
        return Err(Error::InvalidTrustDistance(
            "Trust distance cannot be negative".to_string(),
        ));
    }

    let base = market_rate.base_rate;
    let schema_multiplier = schema_payment.base_multiplier;
    let field_multiplier = field_payment.base_multiplier;

    // Calculate trust distance scaling with safety bounds
    let scale_factor = match &field_payment.trust_distance_scaling {
        TrustDistanceScaling::Linear {
            slope,
            intercept,
            min_factor,
        } => (slope * trust_distance + intercept).max(*min_factor),
        TrustDistanceScaling::Exponential {
            base: exp_base,
            scale,
            min_factor,
        } => exp_base.powf(scale * trust_distance).max(*min_factor),
        TrustDistanceScaling::None => 1.0,
    };

    // Calculate payment with all multipliers
    let payment =
        (base as f64 * schema_multiplier * field_multiplier * scale_factor).round() as u64;

    // Apply minimum thresholds
    let field_min = field_payment.min_payment.unwrap_or(0);
    let schema_min = schema_payment.min_payment_threshold;
    let system_min = global_config.system_base_rate;

    let final_payment = payment.max(field_min).max(schema_min).max(system_min);

    // Validate final payment
    global_config.validate_payment(final_payment)?;

    Ok(final_payment)
}

/// Calculates the total payment required for a query accessing multiple fields.
/// 
/// This function:
/// 1. Calculates individual payments for each field
/// 2. Sums the payments while checking for overflow
/// 3. Applies system-wide minimum threshold
/// 
/// The total payment considers:
/// - Individual field payment calculations
/// - Different trust distances per field
/// - System-wide minimum payment requirement
/// 
/// # Arguments
/// 
/// * `global_config` - System-wide payment configuration
/// * `market_rate` - Current market base rate
/// * `schema_payment` - Schema-level payment configuration
/// * `field_payments` - Vector of (field config, trust distance) pairs
/// 
/// # Returns
/// 
/// A Result containing the total payment amount in satoshis
/// 
/// # Errors
/// 
/// Returns an Error if:
/// - Any individual field payment calculation fails
/// - The sum of payments overflows u64
/// - The final payment amount is invalid
pub fn calculate_total_query_payment(
    global_config: &GlobalPaymentConfig,
    market_rate: &MarketRate,
    schema_payment: &SchemaPaymentConfig,
    field_payments: &[(FieldPaymentConfig, f64)], // (field config, trust distance) pairs
) -> Result<u64, Error> {
    let mut total_payment = 0u64;

    for (field_payment, trust_distance) in field_payments {
        let field_cost = calculate_field_payment(
            global_config,
            market_rate,
            schema_payment,
            field_payment,
            *trust_distance,
        )?;

        // Check for overflow
        total_payment = total_payment
            .checked_add(field_cost)
            .ok_or_else(|| Error::Internal("Payment calculation overflow".to_string()))?;
    }

    // Ensure total payment meets system minimum
    let final_payment = total_payment.max(global_config.system_base_rate);

    Ok(final_payment)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn setup_test_configs() -> (GlobalPaymentConfig, MarketRate, SchemaPaymentConfig) {
        let global_config = GlobalPaymentConfig::new(
            50, // 50 sats minimum
            Duration::from_secs(3600),
            3,
            Duration::from_secs(7200),
        )
        .unwrap();

        let market_rate = MarketRate::new(100); // 100 sats base rate

        let schema_payment = SchemaPaymentConfig::new(1.5, 10).unwrap();

        (global_config, market_rate, schema_payment)
    }

    #[test]
    fn test_calculate_field_payment() {
        let (global_config, market_rate, schema_payment) = setup_test_configs();

        // Test linear scaling
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

        // Expected: 100 * 1.5 * 2.0 * (0.5 * 2 + 1) = 100 * 1.5 * 2.0 * 2 = 600
        assert_eq!(payment, 600);

        // Test exponential scaling
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

        // Expected: 100 * 1.5 * 2.0 * (2^(0.5 * 3)) ≈ 849
        assert!(payment > 800 && payment < 900);
    }

    #[test]
    fn test_minimum_thresholds() {
        let (global_config, market_rate, schema_payment) = setup_test_configs();

        // Test field with high minimum payment
        let field_payment = FieldPaymentConfig::new(
            0.1, // Very low multiplier
            TrustDistanceScaling::None,
            Some(1000), // High minimum threshold
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

        // Should use field minimum payment
        assert_eq!(payment, 1000);
    }

    #[test]
    fn test_total_query_payment() {
        let (global_config, market_rate, schema_payment) = setup_test_configs();

        let field1 = FieldPaymentConfig::new(2.0, TrustDistanceScaling::None, None).unwrap();

        let field2 = FieldPaymentConfig::new(1.5, TrustDistanceScaling::None, None).unwrap();

        let fields = vec![(field1, 1.0), (field2, 1.0)];

        let total =
            calculate_total_query_payment(&global_config, &market_rate, &schema_payment, &fields)
                .unwrap();

        // Expected: (100 * 1.5 * 2.0) + (100 * 1.5 * 1.5) = 300 + 225 = 525
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
}
