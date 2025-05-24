use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::atom::AtomRefRange;
use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::field::common::FieldCommon;
use crate::impl_field;

/// Field storing a range of values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeField {
    pub inner: FieldCommon,
    pub atom_ref_range: Option<AtomRefRange>,
}

impl RangeField {
    #[must_use]
    pub fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: None,
        }
    }

    /// Creates a new RangeField with an AtomRefRange
    #[must_use]
    pub fn new_with_range(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
        source_pub_key: String,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: Some(AtomRefRange::new(source_pub_key)),
        }
    }

    /// Returns a reference to the AtomRefRange if it exists
    pub fn atom_ref_range(&self) -> Option<&AtomRefRange> {
        self.atom_ref_range.as_ref()
    }

    /// Returns a mutable reference to the AtomRefRange if it exists
    pub fn atom_ref_range_mut(&mut self) -> Option<&mut AtomRefRange> {
        self.atom_ref_range.as_mut()
    }

    /// Sets the AtomRefRange for this field
    pub fn set_atom_ref_range(&mut self, atom_ref_range: AtomRefRange) {
        self.atom_ref_range = Some(atom_ref_range);
    }

    /// Initializes the AtomRefRange if it doesn't exist
    pub fn ensure_atom_ref_range(&mut self, source_pub_key: String) -> &mut AtomRefRange {
        if self.atom_ref_range.is_none() {
            self.atom_ref_range = Some(AtomRefRange::new(source_pub_key));
        }
        self.atom_ref_range.as_mut().unwrap()
    }
}

impl_field!(RangeField);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::FieldPaymentConfig;
    use crate::permissions::types::policy::PermissionsPolicy;
    use std::collections::HashMap;

    #[test]
    fn test_range_field_with_atom_ref_range() {
        let permission_policy = PermissionsPolicy::default();
        let payment_config = FieldPaymentConfig::default();
        let field_mappers = HashMap::new();
        let source_pub_key = "test_key".to_string();

        // Test creating RangeField with AtomRefRange
        let mut range_field = RangeField::new_with_range(
            permission_policy,
            payment_config,
            field_mappers,
            source_pub_key.clone(),
        );

        // Verify AtomRefRange is present
        assert!(range_field.atom_ref_range().is_some());

        // Test accessing the AtomRefRange
        if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
            atom_ref_range.set_atom_uuid("key1".to_string(), "atom_uuid_1".to_string());
            atom_ref_range.set_atom_uuid("key2".to_string(), "atom_uuid_2".to_string());
        }

        // Verify the data was set correctly
        if let Some(atom_ref_range) = range_field.atom_ref_range() {
            assert_eq!(atom_ref_range.get_atom_uuid("key1"), Some(&"atom_uuid_1".to_string()));
            assert_eq!(atom_ref_range.get_atom_uuid("key2"), Some(&"atom_uuid_2".to_string()));
        }
    }

    #[test]
    fn test_range_field_ensure_atom_ref_range() {
        let permission_policy = PermissionsPolicy::default();
        let payment_config = FieldPaymentConfig::default();
        let field_mappers = HashMap::new();

        // Create RangeField without AtomRefRange
        let mut range_field = RangeField::new(permission_policy, payment_config, field_mappers);

        // Verify AtomRefRange is not present initially
        assert!(range_field.atom_ref_range().is_none());

        // Ensure AtomRefRange exists
        let source_pub_key = "test_key".to_string();
        let atom_ref_range = range_field.ensure_atom_ref_range(source_pub_key);

        // Add some data
        atom_ref_range.set_atom_uuid("key1".to_string(), "atom_uuid_1".to_string());

        // Verify AtomRefRange is now present and contains data
        assert!(range_field.atom_ref_range().is_some());
        if let Some(atom_ref_range) = range_field.atom_ref_range() {
            assert_eq!(atom_ref_range.get_atom_uuid("key1"), Some(&"atom_uuid_1".to_string()));
        }
    }
}