use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::field::common::FieldCommon;
use crate::impl_field;

/// Field storing a single value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleField {
    pub inner: FieldCommon,
}

impl SingleField {
    #[must_use]
    pub fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
        }
    }
}

impl_field!(SingleField);