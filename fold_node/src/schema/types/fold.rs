use crate::fees::SchemaPaymentConfig;
use crate::schema::types::field::FieldVariant;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Defines a set of fields grouped under a common name.
///
/// A Fold mirrors the behaviour of a `Schema` but stores its
/// fields using the `FieldVariant` type so that each field can
/// be represented as a `SingleField`, `CollectionField` or `RangeField`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fold {
    /// Unique name identifying this fold
    pub name: String,
    /// Collection of fields within this fold
    pub fields: HashMap<String, FieldVariant>,
    /// Payment configuration for fold level operations
    pub payment_config: SchemaPaymentConfig,
}

impl Fold {
    /// Creates a new fold with the specified name.
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            name,
            fields: HashMap::new(),
            payment_config: SchemaPaymentConfig::default(),
        }
    }

    /// Sets the fields for this fold.
    #[must_use]
    pub fn with_fields(mut self, fields: HashMap<String, FieldVariant>) -> Self {
        self.fields = fields;
        self
    }

    /// Sets the payment configuration for this fold.
    #[must_use]
    pub fn with_payment_config(mut self, payment_config: SchemaPaymentConfig) -> Self {
        self.payment_config = payment_config;
        self
    }

    /// Adds a single field to the fold.
    pub fn add_field(&mut self, field_name: String, field: FieldVariant) {
        self.fields.insert(field_name, field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
    use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};
    use crate::schema::types::field::{Field, SingleField};
    use uuid::Uuid;

    fn create_field(policy: PermissionsPolicy) -> FieldVariant {
        let mut field = SingleField::new(
            policy,
            FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
            HashMap::new(),
        );
        field.set_ref_atom_uuid(Uuid::new_v4().to_string());
        FieldVariant::Single(field)
    }

    fn multi_field_fold() -> Fold {
        let mut fold = Fold::new("test_fold".to_string());
        let fields = vec![
            ("public_field", PermissionsPolicy::default()),
            (
                "protected_field",
                PermissionsPolicy::new(TrustDistance::Distance(1), TrustDistance::Distance(2)),
            ),
            (
                "private_field",
                PermissionsPolicy::new(TrustDistance::Distance(3), TrustDistance::Distance(3)),
            ),
        ];

        for (name, policy) in fields {
            fold.add_field(name.to_string(), create_field(policy));
        }
        fold
    }

    #[test]
    fn test_fold_creation() {
        let fold_name = "test_fold".to_string();
        let fold = Fold::new(fold_name.clone());

        assert_eq!(fold.name, fold_name);
        assert!(fold.fields.is_empty());
    }

    #[test]
    fn test_fold_field_management() {
        let mut fold = Fold::new("test_fold".to_string());
        let field_name = "test_field".to_string();
        let field = create_field(PermissionsPolicy::default());

        fold.add_field(field_name.clone(), field.clone());

        assert!(fold.fields.contains_key(&field_name));
        let stored_field = fold.fields.get(&field_name).unwrap();
        assert!(stored_field.ref_atom_uuid().is_some());
    }

    #[test]
    fn test_multi_field_count() {
        let fold = multi_field_fold();
        assert_eq!(fold.fields.len(), 3);
    }
}
