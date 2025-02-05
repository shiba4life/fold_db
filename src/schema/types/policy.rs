use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyLevel {
    Public,
    Private,
    Explicit,
    ExplicitOnce,
    ExplicitMany,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Count {
    pub read: u32,
    pub write: u32,
}

impl Count {
    pub const LIMITED: fn(u32) -> Count = |limit| Count {
        read: limit,
        write: limit,
    };
    
    pub const UNLIMITED: fn() -> Count = || Count {
        read: u32::MAX,
        write: u32::MAX,
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplicitCounts {
    pub counts: HashMap<String, Count>, // pub_key -> counts
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsPolicy {
    pub read_policy: PolicyLevel,
    pub write_policy: PolicyLevel,
    pub explicit_counts: Option<ExplicitCounts>,
}

impl PermissionsPolicy {
    pub fn new(read_policy: PolicyLevel, write_policy: PolicyLevel) -> Self {
        Self {
            read_policy,
            write_policy,
            explicit_counts: None,
        }
    }

    pub fn set_explicit_permissions(&mut self, pub_key: String, r: Count, w: Count) {
        let mut counts = self.explicit_counts.get_or_insert(ExplicitCounts {
            counts: HashMap::new(),
        });
        counts.counts.insert(pub_key, Count {
            read: r.read,
            write: w.write,
        });
    }
}
