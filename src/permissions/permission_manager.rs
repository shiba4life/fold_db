use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};

#[derive(Default)]
pub struct PermissionManager {

}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_read_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u32) -> bool {
        // Check trust distance first
        let trust_allowed = match permissions_policy.read_policy {
            TrustDistance::NoRequirement => {
                eprintln!("No distance requirement for {}", pub_key);
                true
            },
            TrustDistance::Distance(required_distance) => {
                let allowed = trust_distance <= required_distance;
                eprintln!("Trust distance check for {}: {} <= {} = {}", 
                    pub_key, trust_distance, required_distance, allowed);
                allowed
            }
        };

        // If trust distance check passes, allow access
        if trust_allowed {
            eprintln!("Trust distance check passed for {}", pub_key);
            return true;
        }

        // If trust distance check fails, check explicit permissions
        if let Some(explicit_policy) = &permissions_policy.explicit_read_policy {
            let allowed = explicit_policy.counts_by_pub_key.contains_key(pub_key);
            eprintln!("Trust distance failed, checking explicit permission for {}: {}", pub_key, allowed);
            allowed
        } else {
            eprintln!("Trust distance failed and no explicit permissions for {}", pub_key);
            false
        }
    }

    pub fn has_write_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u32) -> bool {
        // Check trust distance first
        let trust_allowed = match permissions_policy.write_policy {
            TrustDistance::NoRequirement => {
                eprintln!("No distance requirement for {}", pub_key);
                true
            },
            TrustDistance::Distance(required_distance) => {
                let allowed = trust_distance <= required_distance;
                eprintln!("Trust distance check for {}: {} <= {} = {}", 
                    pub_key, trust_distance, required_distance, allowed);
                allowed
            }
        };

        // If trust distance check passes, allow access
        if trust_allowed {
            eprintln!("Trust distance check passed for {}", pub_key);
            return true;
        }

        // If trust distance check fails, check explicit permissions
        if let Some(explicit_policy) = &permissions_policy.explicit_write_policy {
            let allowed = explicit_policy.counts_by_pub_key.contains_key(pub_key);
            eprintln!("Trust distance failed, checking explicit permission for {}: {}", pub_key, allowed);
            allowed
        } else {
            eprintln!("Trust distance failed and no explicit permissions for {}", pub_key);
            false
        }
    }
}
