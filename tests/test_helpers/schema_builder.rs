use fold_db::fees::payment_config::SchemaPaymentConfig;
use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use fold_db::schema::mapper::{MappingRule, SchemaMapper};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::Schema;
use std::collections::HashMap;

pub fn create_default_payment_config() -> FieldPaymentConfig {
    FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap()
}

pub fn create_field_with_permissions(
    ref_atom_uuid: String,
    read_distance: u32,
    write_distance: u32,
    explicit_read_keys: Option<HashMap<String, u8>>,
    explicit_write_keys: Option<HashMap<String, u8>>,
) -> SchemaField {
    SchemaField {
        ref_atom_uuid,
        permission_policy: PermissionsPolicy {
            read_policy: TrustDistance::Distance(read_distance),
            write_policy: TrustDistance::Distance(write_distance),
            explicit_read_policy: explicit_read_keys.map(|counts| ExplicitCounts {
                counts_by_pub_key: counts,
            }),
            explicit_write_policy: explicit_write_keys.map(|counts| ExplicitCounts {
                counts_by_pub_key: counts,
            }),
        },
        payment_config: create_default_payment_config(),
    }
}

pub fn create_schema_with_fields(
    name: String,
    fields: HashMap<String, SchemaField>,
    mappers: Vec<SchemaMapper>,
) -> Schema {
    Schema {
        name,
        fields,
        schema_mappers: mappers,
        payment_config: SchemaPaymentConfig::default(),
    }
}

pub fn create_rename_mapper(
    source_schema_name: String,
    target_schema_name: String,
    source_field: String,
    target_field: String,
) -> SchemaMapper {
    SchemaMapper::new(
        source_schema_name,
        target_schema_name,
        vec![MappingRule::Rename {
            source_field,
            target_field,
        }],
    )
}
