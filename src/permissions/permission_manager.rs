use crate::permissions::types::policy::PermissionsPolicy;

pub struct PermissionManager {

}

impl PermissionManager {
    pub fn has_read_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u8) -> bool {
        permissions_policy.read_policy >= trust_distance || 
        permissions_policy.explicit_read_policy.as_ref()
            .map_or(false, |counts| counts.counts_by_pub_key.get(pub_key).is_some())
    }

    pub fn has_write_permission(&self, pub_key: &str, permissions_policy: &PermissionsPolicy, trust_distance: u8) -> bool {
        permissions_policy.write_policy >= trust_distance || 
        permissions_policy.explicit_write_policy.as_ref()
            .map_or(false, |counts| counts.counts_by_pub_key.get(pub_key).is_some())
    }
}

