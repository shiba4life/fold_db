use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use fold_node::schema::Schema;
use tempfile::tempdir;

fn create_node(path: &std::path::Path) -> DataFoldNode {
    let config = NodeConfig {
        storage_path: path.into(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    DataFoldNode::new(config).unwrap()
}

#[test]
fn load_and_list_schema() {
    let dir = tempdir().unwrap();
    let mut node = create_node(dir.path());
    let schema = Schema::new("Test".to_string());
    node.load_schema(schema).unwrap();
    let schemas = node.list_schemas().unwrap();
    assert_eq!(schemas.len(), 1);
}

#[test]
fn load_schema_invalid_fails() {
    let dir = tempdir().unwrap();
    let mut node = create_node(dir.path());

    let mut schema = Schema::new("Bad".to_string());
    schema.payment_config.base_multiplier = 0.0;

    let res = node.load_schema(schema);
    assert!(res.is_err());
}
