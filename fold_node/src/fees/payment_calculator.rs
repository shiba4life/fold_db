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

