#![allow(dead_code)]
use fold_node::testing::{
    ExplicitCounts, FieldPaymentConfig, FieldVariant, PermissionsPolicy, Schema, SingleField,
    SchemaPaymentConfig, TrustDistance,
};
use std::collections::HashMap;

pub fn create_field_with_permissions(
    ref_atom_uuid: String,
    read_distance: u32,
    write_distance: u32,
    explicit_read_keys: Option<HashMap<String, u8>>,
    explicit_write_keys: Option<HashMap<String, u8>>,
) -> FieldVariant {
    let mut field = FieldVariant::Single(
        SingleField::new(
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
        )
    );
    
    // Set the ref_atom_uuid on the field
    use fold_node::testing::Field;
    field.set_ref_atom_uuid(ref_atom_uuid);
    
    field
}

pub fn create_schema_with_fields(name: String, fields: HashMap<String, FieldVariant>) -> Schema {
    Schema::new_range(name, "key".to_string())
        .with_fields(fields)
        .with_payment_config(SchemaPaymentConfig::default())
}
