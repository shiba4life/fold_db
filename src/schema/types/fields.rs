use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::PermissionsPolicy;
use crate::fees::types::config::FieldPaymentConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub permission_policy: PermissionsPolicy,
    pub ref_atom_uuid: String,
    pub payment_config: FieldPaymentConfig,
    // field value is fetched through the ref_atom_uuid
}

impl SchemaField {
    pub fn new(permission_policy: PermissionsPolicy, ref_atom_uuid: String, payment_config: FieldPaymentConfig) -> Self {
        Self {
            permission_policy,
            ref_atom_uuid,
            payment_config,
        }
    }
}
