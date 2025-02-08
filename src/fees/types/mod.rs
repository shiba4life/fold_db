pub mod config;
pub mod lightning;
pub mod payment;

// Re-export commonly used types
pub use payment::{Error, PaymentState, PaymentStatus, LightningPaymentRequest};
pub use config::{GlobalPaymentConfig, SchemaPaymentConfig, MarketRate, FieldPaymentConfig, TrustDistanceScaling};
pub use lightning::{NodeConfig, Network, NodeInfo, Channel, RoutingFees};
