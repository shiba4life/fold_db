use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use tempfile::tempdir;

#[test]
fn queue_info_works() {
    let dir = tempdir().unwrap();
    let config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let node = DataFoldNode::new(config).unwrap();
    let info = node.get_transform_queue_info().unwrap();
    assert!(info.get("queue").is_some());
}
