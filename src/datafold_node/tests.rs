use super::*;
use tempfile::tempdir;

fn create_test_config() -> NodeConfig {
    let dir = tempdir().unwrap();
    NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    }
}

#[test]
fn test_node_creation() {
    let config = create_test_config();
    let node = DataFoldNode::new(config);
    assert!(node.is_ok());
}

#[test]
fn test_add_trusted_node() {
    let config = create_test_config();
    let mut node = DataFoldNode::new(config).unwrap();

    assert!(node.add_trusted_node("test_node").is_ok());
    assert!(node.get_trusted_nodes().contains_key("test_node"));
    assert!(node.remove_trusted_node("test_node").is_ok());
    assert!(!node.get_trusted_nodes().contains_key("test_node"));
}
