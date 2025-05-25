use crate::fees::types::payment::Error;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldPaymentConfig {
    pub base_multiplier: f64,
    pub trust_distance_scaling: TrustDistanceScaling,
    pub min_payment: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustDistanceScaling {
    Linear {
        slope: f64,
        intercept: f64,
        min_factor: f64,
    },
    Exponential {
        base: f64,
        scale: f64,
        min_factor: f64,
    },
    None,
}

impl GlobalPaymentConfig {
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
            return Err(Error::InvalidAmount(
                "Timeout durations must be greater than 0".to_string(),
            ));
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

    pub fn is_stale(&self, max_age: Duration) -> bool {
        let age = Utc::now()
            .signed_duration_since(self.last_updated)
            .to_std()
            .unwrap_or(Duration::from_secs(0));
        age > max_age
    }
}

impl FieldPaymentConfig {
    pub fn new(
        base_multiplier: f64,
        trust_distance_scaling: TrustDistanceScaling,
        min_payment: Option<u64>,
    ) -> Result<Self, Error> {
        if base_multiplier <= 0.0 {
            return Err(Error::InvalidAmount(
                "Base multiplier must be positive".to_string(),
            ));
        }

        // Validate trust distance scaling parameters
        match &trust_distance_scaling {
            TrustDistanceScaling::Linear {
                slope: _,
                intercept: _,
                min_factor,
            } => {
                if *min_factor < 1.0 {
                    return Err(Error::InvalidAmount(
                        "Minimum scaling factor must be >= 1.0".to_string(),
                    ));
                }
            }
            TrustDistanceScaling::Exponential {
                base,
                scale: _,
                min_factor,
            } => {
                if *base <= 0.0 {
                    return Err(Error::InvalidAmount(
                        "Exponential base must be positive".to_string(),
                    ));
                }
                if *min_factor < 1.0 {
                    return Err(Error::InvalidAmount(
                        "Minimum scaling factor must be >= 1.0".to_string(),
                    ));
                }
            }
            TrustDistanceScaling::None => {}
        }

        Ok(Self {
            base_multiplier,
            trust_distance_scaling,
            min_payment,
        })
    }
}

impl Default for SchemaPaymentConfig {
    fn default() -> Self {
        Self::new(1.0, 0).expect("Default schema payment config should be valid")
    }
}

impl Default for FieldPaymentConfig {
    fn default() -> Self {
        Self::new(1.0, TrustDistanceScaling::None, None)
            .expect("Default payment config should be valid")
    }
}

