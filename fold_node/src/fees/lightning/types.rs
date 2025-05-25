use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub host: String,
    pub port: u16,
    pub macaroon_path: String,
    pub tls_cert_path: String,
    pub network: Network,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Network {
    #[default]
    Mainnet,
    Testnet,
    Regtest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub pubkey: String,
    pub alias: String,
    pub network: Network,
    pub version: String,
    pub block_height: u32,
    pub synced_to_chain: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String, // Renamed from channel_id to avoid struct name repetition
    pub capacity: u64,
    pub local_balance: u64,
    pub remote_balance: u64,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingFees {
    pub base_fee_msat: u64,
    pub fee_rate_millionths: u64,
    pub time_lock_delta: u32,
}

impl NodeConfig {
    #[cfg(test)]
    pub fn new(
        host: String,
        port: u16,
        macaroon_path: String,
        tls_cert_path: String,
        network: Network,
    ) -> Self {
        Self {
            host,
            port,
            macaroon_path,
            tls_cert_path,
            network,
        }
    }

    #[cfg(test)]
    pub fn get_connection_string(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Mainnet => write!(f, "mainnet"),
            Self::Testnet => write!(f, "testnet"),
            Self::Regtest => write!(f, "regtest"),
        }
    }
}

