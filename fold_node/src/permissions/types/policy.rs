use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustDistance {
    Distance(u32),
    NoRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitCounts {
    pub counts_by_pub_key: HashMap<String, u8>, // pub_key -> counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    pub read_policy: TrustDistance,
    pub write_policy: TrustDistance,
    pub explicit_write_policy: Option<ExplicitCounts>,
    pub explicit_read_policy: Option<ExplicitCounts>,
}

impl PermissionsPolicy {
    pub fn new(read_policy: TrustDistance, write_policy: TrustDistance) -> Self {
        Self {
            read_policy,
            write_policy,
            explicit_write_policy: None,
            explicit_read_policy: None,
        }
    }
}

impl Default for PermissionsPolicy {
    fn default() -> Self {
        Self {
            read_policy: TrustDistance::Distance(0),
            write_policy: TrustDistance::Distance(0),
            explicit_write_policy: None,
            explicit_read_policy: None,
        }
    }
}
