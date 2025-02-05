use std::collections::HashMap;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitCounts {
    pub counts_by_pub_key: HashMap<String, u8>, // pub_key -> counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    pub read_policy: u8,
    pub write_policy: u8,
    pub explicit_write_policy: Option<ExplicitCounts>,
    pub explicit_read_policy: Option<ExplicitCounts>,
}

impl PermissionsPolicy {
    pub fn new(read_policy: u8, write_policy: u8) -> Self {
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
            read_policy: 0,
            write_policy: 0,
            explicit_write_policy: None,
            explicit_read_policy: None,
        }
    }
}
