mod client;

pub use crate::fees::types::lightning::{
    Channel, NodeConfig, NodeInfo, Network, RoutingFees,
};

#[cfg(test)]
mod mock;

pub use client::LightningClient;
#[cfg(test)]
pub use mock::MockLightningClient;
