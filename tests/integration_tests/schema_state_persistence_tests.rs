use fold_node::testing::{FieldVariant, SingleField, Schema};
use fold_node::fees::types::{FieldPaymentConfig, TrustDistanceScaling};
use fold_node::permissions::types::policy::{PermissionsPolicy, TrustDistance};
use fold_node::{DataFoldNode, NodeConfig};
use tempfile::tempdir;
use std::collections::HashMap;

#[test]
fn unload_state_persists_on_disk() {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let mut node = DataFoldNode::new(config.clone()).unwrap();

    let field = FieldVariant::Single(SingleField::new(
        PermissionsPolicy::new(TrustDistance::Distance(0), TrustDistance::Distance(0)),
        FieldPaymentConfig::new(1.0, TrustDistanceScaling::None, None).unwrap(),
        HashMap::new(),
    ));

    let mut schema = Schema::new("PersistUnload".to_string());
    schema.add_field("id".to_string(), field);

    node.add_schema_available(schema).unwrap();
    node.approve_schema("PersistUnload").unwrap();

    node.unload_schema("PersistUnload").unwrap();

    drop(node);

    let node2 = DataFoldNode::new(config).unwrap();

    assert!(node2.list_available_schemas().unwrap().contains(&"PersistUnload".to_string()));
    let loaded_names: Vec<String> = node2.list_schemas().unwrap();
    assert!(!loaded_names.contains(&"PersistUnload".to_string()));
}
