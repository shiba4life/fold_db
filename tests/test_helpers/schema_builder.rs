use fold_db::fees::payment_config::SchemaPaymentConfig;
use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{ExplicitCounts, PermissionsPolicy, TrustDistance};
use fold_db::schema::mapper::{MappingRule, SchemaMapper, parse_mapping_dsl};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::Schema;
use std::collections::HashMap;


pub fn create_dsl_mapper(
    source_schema_name: String,
    dsl: String,
) -> SchemaMapper {
    let rules = parse_mapping_dsl(&dsl).expect("Failed to parse DSL");
    SchemaMapper::new(source_schema_name, rules)
}

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
    mappers: Vec<SchemaMapper>,
) -> Schema {
    Schema::new(name)
        .with_fields(fields)
        .with_schema_mappers(mappers)
        .with_payment_config(SchemaPaymentConfig::default())
}

pub fn create_rename_mapper(
    source_schema_name: String,
    source_field: String,
    target_field: String,
) -> SchemaMapper {
    SchemaMapper::new(
        source_schema_name,
        vec![MappingRule::Rename {
            source_field,
            target_field,
        }],
    )
}
