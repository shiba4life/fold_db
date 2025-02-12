pub mod config;
pub mod lightning;
pub mod payment;

// Re-export commonly used types
pub use config::{
    FieldPaymentConfig, GlobalPaymentConfig, MarketRate, SchemaPaymentConfig, TrustDistanceScaling,
};
pub use lightning::{Channel, Network, NodeConfig, NodeInfo, RoutingFees};
pub use payment::{Error, LightningPaymentRequest, PaymentState, PaymentStatus};
