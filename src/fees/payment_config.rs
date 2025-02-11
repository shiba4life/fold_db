use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::fees::types::payment::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalPaymentConfig {
    pub system_base_rate: u64,
    pub payment_timeout: Duration,
    pub max_invoice_retries: u32,
    pub hold_invoice_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaPaymentConfig {
    pub base_multiplier: f64,
    pub min_payment_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketRate {
    pub base_rate: u64,
    pub last_updated: DateTime<Utc>,
}

impl GlobalPaymentConfig {
    /// Creates a new `GlobalPaymentConfig`
    ///
    /// # Errors
    ///
    /// Returns an Error if:
    /// - The system base rate is 0
    /// - The payment timeout is 0
    /// - The max invoice retries is 0
    /// - The hold invoice timeout is 0
    pub fn new(
        system_base_rate: u64,
        payment_timeout: Duration,
        max_invoice_retries: u32,
        hold_invoice_timeout: Duration,
    ) -> Result<Self, Error> {
        if system_base_rate == 0 {
            return Err(Error::InvalidAmount(
                "System base rate must be greater than 0".to_string(),
            ));
        }

        if payment_timeout.as_secs() == 0 || hold_invoice_timeout.as_secs() == 0 {
            return Err(Error::InvalidAmount("Timeout durations must be greater than 0".to_string()));
        }

        if max_invoice_retries == 0 {
            return Err(Error::InvalidAmount(
                "Maximum invoice retries must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            system_base_rate,
            payment_timeout,
            max_invoice_retries,
            hold_invoice_timeout,
        })
    }

    /// Validates a payment amount against configuration
    ///
    /// # Errors
    ///
    /// Returns an Error if:
    /// - The payment amount is below the system base rate
    pub fn validate_payment(&self, amount: u64) -> Result<(), Error> {
        if amount < self.system_base_rate {
            return Err(Error::InvalidAmount(format!(
                "Payment amount {} is below system base rate {}",
                amount, self.system_base_rate
            )));
        }
        Ok(())
    }
}

impl SchemaPaymentConfig {
    /// Creates a new `SchemaPaymentConfig`
    ///
    /// # Errors
    ///
    /// Returns an Error if:
    /// - The base multiplier is not positive (must be greater than 0)
    /// - The minimum payment threshold is invalid
    pub fn new(base_multiplier: f64, min_payment_threshold: u64) -> Result<Self, Error> {
        if base_multiplier <= 0.0 {
            return Err(Error::InvalidAmount(
                "Schema base multiplier must be positive".to_string(),
            ));
        }

        Ok(Self {
            base_multiplier,
            min_payment_threshold,
        })
    }
}

impl MarketRate {
    #[must_use]
    pub fn new(base_rate: u64) -> Self {
        Self {
            base_rate,
            last_updated: Utc::now(),
        }
    }

    pub fn update(&mut self, new_rate: u64) {
        self.base_rate = new_rate;
        self.last_updated = Utc::now();
    }

    #[must_use]
    pub fn is_stale(&self, max_age: Duration) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.last_updated)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
        age > max_age
    }
}

impl Default for SchemaPaymentConfig {
    fn default() -> Self {
        Self::new(1.0, 0).expect("Default schema payment config should be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_payment_config_validation() {
        // Valid config
        let config = GlobalPaymentConfig::new(
            100,
            Duration::from_secs(3600),
            3,
            Duration::from_secs(7200),
        );
        assert!(config.is_ok());

        // Invalid system base rate
        let config = GlobalPaymentConfig::new(
            0,
            Duration::from_secs(3600),
            3,
            Duration::from_secs(7200),
        );
        assert!(config.is_err());

        // Invalid payment timeout
        let config = GlobalPaymentConfig::new(
            100,
            Duration::from_secs(0),
            3,
            Duration::from_secs(7200),
        );
        assert!(config.is_err());

        // Invalid max retries
        let config = GlobalPaymentConfig::new(
            100,
            Duration::from_secs(3600),
            0,
            Duration::from_secs(7200),
        );
        assert!(config.is_err());
    }

    #[test]
    fn test_schema_payment_config_validation() {
        // Valid config
        let config = SchemaPaymentConfig::new(1.5, 10);
        assert!(config.is_ok());

        // Invalid base multiplier
        let config = SchemaPaymentConfig::new(0.0, 10);
        assert!(config.is_err());
        let config = SchemaPaymentConfig::new(-1.0, 10);
        assert!(config.is_err());
    }

    #[test]
    fn test_market_rate_staleness() {
        let mut rate = MarketRate::new(100);
        
        // Not stale immediately
        assert!(!rate.is_stale(Duration::from_secs(60)));
        
        // Force last_updated to be in the past
        rate.last_updated = rate.last_updated - chrono::Duration::seconds(120);
        
        // Should be stale now
        assert!(rate.is_stale(Duration::from_secs(60)));
        
        // Update should reset staleness
        rate.update(150);
        assert!(!rate.is_stale(Duration::from_secs(60)));
    }
}
