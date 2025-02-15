use fold_db::testing::{
    SchemaPaymentConfig,
    FieldPaymentConfig,
    TrustDistanceScaling,
    ExplicitCounts,
    PermissionsPolicy,
    TrustDistance,
    SchemaField,
    Schema,
};
use std::collections::HashMap;

pub fn create_field_with_permissions(
    ref_atom_uuid: String,
    read_distance: u32,
    write_distance: u32,
    explicit_read_keys: Option<HashMap<String, u8>>,
    explicit_write_keys: Option<HashMap<String, u8>>,
) -> SchemaField {
    SchemaField::new(
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
    ).with_ref_atom_uuid(ref_atom_uuid)
}

pub fn create_schema_with_fields(
    name: String,
    fields: HashMap<String, SchemaField>,
) -> Schema {
    Schema::new(name)
        .with_fields(fields)
        .with_payment_config(SchemaPaymentConfig::default())
}
