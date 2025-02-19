use fold_db::testing::{
    FieldPaymentConfig,
    TrustDistanceScaling,
    PermissionsPolicy,
    TrustDistance,
    SchemaField,
    Schema,
};
use fold_db::{DataFoldNode, NodeConfig};
use std::collections::HashMap;
use tempfile::tempdir;

fn main() {
    // Create a temporary directory for the node
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
    };

    // Create a new node
    let mut node = DataFoldNode::new(config).unwrap();

    // Create a schema
    let mut schema = Schema::new("user_profile".to_string());

    // Add name field
    let name_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    schema.add_field("name".to_string(), name_field);

    // Add email field
    let email_field = SchemaField::new(
        PermissionsPolicy::new(
            TrustDistance::Distance(1),
            TrustDistance::Distance(1),
        ),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    );
    schema.add_field("email".to_string(), email_field);

    // Load the schema
    node.load_schema(schema).unwrap();
    node.allow_schema("user_profile").unwrap();

    println!("Node initialized with user_profile schema");
}
