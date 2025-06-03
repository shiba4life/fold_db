use fold_node::testing::{
    ExplicitCounts, Field, FieldPaymentConfig, FieldVariant, PermissionsPolicy, Schema,
    SchemaPaymentConfig, SingleField, TrustDistance,
};
use std::collections::HashMap;

#[allow(dead_code)]
pub fn create_field_with_permissions(
    ref_atom_uuid: String,
    read_distance: u32,
    write_distance: u32,
    explicit_read_keys: Option<HashMap<String, u8>>,
    explicit_write_keys: Option<HashMap<String, u8>>,
) -> FieldVariant {
    let mut field = SingleField::new(
        PermissionsPolicy {
            read_policy: TrustDistance::Distance(read_distance),
            write_policy: TrustDistance::Distance(write_distance),
            explicit_read_policy: explicit_read_keys.map(|counts| ExplicitCounts {
                counts_by_pub_key: counts,
            }),
            explicit_write_policy: explicit_write_keys.map(|counts| ExplicitCounts {
                counts_by_pub_key: counts,
            }),
        },
        FieldPaymentConfig::default(),
        HashMap::new(),
    );
    field.set_ref_atom_uuid(ref_atom_uuid);
    FieldVariant::Single(field)
}

#[allow(dead_code)]
pub fn create_schema_with_fields(name: String, fields: HashMap<String, FieldVariant>) -> Schema {
    Schema::new(name)
        .with_fields(fields)
        .with_payment_config(SchemaPaymentConfig::default())
}
