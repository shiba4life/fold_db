use fold_db::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_db::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_db::schema::types::fields::SchemaField;
use fold_db::schema::Schema;
use fold_db::{DataFoldNode, NodeConfig};
use std::collections::HashMap;
use tempfile::tempdir;
use uuid;

fn main() {
    // Create a temporary directory for the node
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        docker: fold_db::datafold_node::DockerConfig::default(),
    };

    // Create a new node
    let mut node = DataFoldNode::new(config).unwrap();

    // Create a schema
    let mut schema = Schema::new("user_profile".to_string());

    // Add name field
    let name_field = SchemaField {
        ref_atom_uuid: Some(uuid::Uuid::new_v4().to_string()),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        field_mappers: HashMap::new(),
    };
    schema.add_field("name".to_string(), name_field);

    // Add email field
    let email_field = SchemaField {
        ref_atom_uuid: Some(uuid::Uuid::new_v4().to_string()),
        permission_policy: PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        payment_config: FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        field_mappers: HashMap::new(),
    };
    schema.add_field("email".to_string(), email_field);

    // Load the schema
    node.load_schema(schema).unwrap();
    node.allow_schema("user_profile").unwrap();

    println!("Node initialized with user_profile schema");
}
