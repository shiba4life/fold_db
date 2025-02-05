use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::PermissionsPolicy;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub permission_policy: PermissionsPolicy,
    pub ref_atom_uuid: String,
    // field value is fetched through the ref_atom_uuid
}

impl SchemaField {
    pub fn new(permission_policy: PermissionsPolicy, ref_atom_uuid: String) -> Self {
        Self {
            permission_policy,
            ref_atom_uuid,
        }
    }
}
