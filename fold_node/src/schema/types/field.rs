use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::collections::HashMap;

use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::Transform;

/// Common interface for all schema fields.
///
/// The `Field` trait exposes accessors for properties shared by all field
/// implementations. These mirror the methods that previously existed on
/// `SchemaField`.
pub trait Field {
    /// Returns the permission policy associated with this field.
    fn permission_policy(&self) -> &PermissionsPolicy;

    /// Returns the payment configuration for this field.
    fn payment_config(&self) -> &FieldPaymentConfig;

    /// Gets the atom reference uuid for this field, if one exists.
    fn ref_atom_uuid(&self) -> Option<&String>;

    /// Sets the atom reference uuid for this field.
    fn set_ref_atom_uuid(&mut self, uuid: String);

    /// Returns any field mappers configured for this field.
    fn field_mappers(&self) -> &HashMap<String, String>;

    /// Sets the field mappers for this field.
    fn set_field_mappers(&mut self, mappers: HashMap<String, String>);

    /// Returns the transform associated with this field, if any.
    fn transform(&self) -> Option<&Transform>;

    /// Sets the transform for this field.
    fn set_transform(&mut self, transform: Transform);

    /// Indicates whether this field is writable.
    fn writable(&self) -> bool;

    /// Sets whether this field can be written to.
    fn set_writable(&mut self, writable: bool);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldCommon {
    permission_policy: PermissionsPolicy,
    payment_config: FieldPaymentConfig,
    ref_atom_uuid: Option<String>,
    field_mappers: HashMap<String, String>,
    transform: Option<Transform>,
    writable: bool,
}

impl FieldCommon {
    fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
    ) -> Self {
        Self {
            permission_policy,
            payment_config,
            ref_atom_uuid: None,
            field_mappers,
            transform: None,
            writable: true,
        }
    }
}

macro_rules! impl_field {
    ($t:ty) => {
        impl Field for $t {
            fn permission_policy(&self) -> &PermissionsPolicy {
                &self.inner.permission_policy
            }

            fn payment_config(&self) -> &FieldPaymentConfig {
                &self.inner.payment_config
            }

            fn ref_atom_uuid(&self) -> Option<&String> {
                self.inner.ref_atom_uuid.as_ref()
            }

            fn set_ref_atom_uuid(&mut self, uuid: String) {
                self.inner.ref_atom_uuid = Some(uuid);
            }

            fn field_mappers(&self) -> &HashMap<String, String> {
                &self.inner.field_mappers
            }

            fn set_field_mappers(&mut self, mappers: HashMap<String, String>) {
                self.inner.field_mappers = mappers;
            }

            fn transform(&self) -> Option<&Transform> {
                self.inner.transform.as_ref()
            }

            fn set_transform(&mut self, transform: Transform) {
                self.inner.transform = Some(transform);
            }

            fn writable(&self) -> bool {
                self.inner.writable
            }

            fn set_writable(&mut self, writable: bool) {
                self.inner.writable = writable;
            }
        }
    };
}

/// Field storing a single value.
#[deprecated(note = "Schema system is deprecated and will be removed")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SingleField {
    inner: FieldCommon,
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

/// Field storing a collection of values.
#[deprecated(note = "Schema system is deprecated and will be removed")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionField {
    inner: FieldCommon,
}

impl CollectionField {
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

impl_field!(CollectionField);

/// Field storing a range of values.
#[deprecated(note = "Schema system is deprecated and will be removed")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeField {
    inner: FieldCommon,
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
        }
    }
}

impl_field!(RangeField);

/// Enumeration over all field variants.
#[derive(Debug, Clone)]
pub enum FieldVariant {
    /// Single value field
    Single(SingleField),
    /// Collection of values
    Collection(CollectionField),
    /// Range of values
    Range(RangeField),
}

impl Field for FieldVariant {
    fn permission_policy(&self) -> &PermissionsPolicy {
        match self {
            Self::Single(f) => f.permission_policy(),
            Self::Collection(f) => f.permission_policy(),
            Self::Range(f) => f.permission_policy(),
        }
    }

    fn payment_config(&self) -> &FieldPaymentConfig {
        match self {
            Self::Single(f) => f.payment_config(),
            Self::Collection(f) => f.payment_config(),
            Self::Range(f) => f.payment_config(),
        }
    }

    fn ref_atom_uuid(&self) -> Option<&String> {
        match self {
            Self::Single(f) => f.ref_atom_uuid(),
            Self::Collection(f) => f.ref_atom_uuid(),
            Self::Range(f) => f.ref_atom_uuid(),
        }
    }

    fn set_ref_atom_uuid(&mut self, uuid: String) {
        match self {
            Self::Single(f) => f.set_ref_atom_uuid(uuid),
            Self::Collection(f) => f.set_ref_atom_uuid(uuid),
            Self::Range(f) => f.set_ref_atom_uuid(uuid),
        }
    }

    fn field_mappers(&self) -> &HashMap<String, String> {
        match self {
            Self::Single(f) => f.field_mappers(),
            Self::Collection(f) => f.field_mappers(),
            Self::Range(f) => f.field_mappers(),
        }
    }

    fn set_field_mappers(&mut self, mappers: HashMap<String, String>) {
        match self {
            Self::Single(f) => f.set_field_mappers(mappers),
            Self::Collection(f) => f.set_field_mappers(mappers),
            Self::Range(f) => f.set_field_mappers(mappers),
        }
    }

    fn transform(&self) -> Option<&Transform> {
        match self {
            Self::Single(f) => f.transform(),
            Self::Collection(f) => f.transform(),
            Self::Range(f) => f.transform(),
        }
    }

    fn set_transform(&mut self, transform: Transform) {
        match self {
            Self::Single(f) => f.set_transform(transform),
            Self::Collection(f) => f.set_transform(transform),
            Self::Range(f) => f.set_transform(transform),
        }
    }

    fn writable(&self) -> bool {
        match self {
            Self::Single(f) => f.writable(),
            Self::Collection(f) => f.writable(),
            Self::Range(f) => f.writable(),
        }
    }

    fn set_writable(&mut self, writable: bool) {
        match self {
            Self::Single(f) => f.set_writable(writable),
            Self::Collection(f) => f.set_writable(writable),
            Self::Range(f) => f.set_writable(writable),
        }
    }
}

impl Serialize for FieldVariant {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use crate::schema::types::fields::FieldType;

        #[derive(Serialize)]
        struct Helper<'a> {
            #[serde(flatten)]
            inner: &'a FieldCommon,
            field_type: FieldType,
        }

        let helper = match self {
            Self::Single(f) => Helper { inner: &f.inner, field_type: FieldType::Single },
            Self::Collection(f) => Helper { inner: &f.inner, field_type: FieldType::Collection },
            Self::Range(f) => Helper { inner: &f.inner, field_type: FieldType::Range },
        };

        helper.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FieldVariant {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use crate::schema::types::fields::FieldType;

        #[derive(Deserialize)]
        struct Helper {
            #[serde(flatten)]
            inner: FieldCommon,
            field_type: Option<FieldType>,
        }

        let helper = Helper::deserialize(deserializer)?;
        Ok(match helper.field_type.unwrap_or(FieldType::Single) {
            FieldType::Single => Self::Single(SingleField { inner: helper.inner }),
            FieldType::Collection => Self::Collection(CollectionField { inner: helper.inner }),
            FieldType::Range => Self::Range(RangeField { inner: helper.inner }),
        })
    }
}

