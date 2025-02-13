use serde::{Deserialize, Serialize};
use crate::permissions::types::policy::PermissionsPolicy;
use crate::fees::types::config::FieldPaymentConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub permission_policy: PermissionsPolicy,
    pub payment_config: FieldPaymentConfig,
    // field value is fetched through the ref_atom_uuid
    pub ref_atom_uuid: Option<String>,
}

impl SchemaField {
    #[must_use]
    pub fn new(permission_policy: PermissionsPolicy, payment_config: FieldPaymentConfig) -> Self {
        Self {
            permission_policy,
            payment_config,
            ref_atom_uuid: None,
        }
    }

    pub fn with_ref_atom_uuid(mut self, ref_atom_uuid: String) -> Self {
        self.ref_atom_uuid = Some(ref_atom_uuid);
        self
    }
}
