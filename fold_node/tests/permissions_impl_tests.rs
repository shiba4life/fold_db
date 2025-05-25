use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use tempfile::tempdir;

#[test]
fn trusted_node_management() {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let mut node = DataFoldNode::new(config).unwrap();
    node.add_trusted_node("peer1").unwrap();
    assert!(node.get_trusted_nodes().contains_key("peer1"));
    node.remove_trusted_node("peer1").unwrap();
    assert!(!node.get_trusted_nodes().contains_key("peer1"));
}
