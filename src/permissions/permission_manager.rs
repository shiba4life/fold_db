use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};

pub struct PermissionManager {

}

impl PermissionManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn has_read_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u32) -> bool {
        // Check explicit read policy first
        if let Some(explicit_policy) = &permissions_policy.explicit_read_policy {
            if explicit_policy.counts_by_pub_key.contains_key(pub_key) {
                return true;
            }
        }
        // Then check trust distance - lower trust_distance means higher trust
        match permissions_policy.read_policy {
            TrustDistance::NoRequirement => true,
            TrustDistance::Distance(required_distance) => trust_distance <= required_distance
        }
    }

    pub fn has_write_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u32) -> bool {
        // Write permissions only use explicit policy
        if let Some(explicit_policy) = &permissions_policy.explicit_write_policy {
            explicit_policy.counts_by_pub_key.contains_key(pub_key)
        } else {
            false // No explicit write permission means no write access
        }
    }
}
