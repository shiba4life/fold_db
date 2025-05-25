use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use fold_node::network::NetworkConfig;
use std::time::Duration;
use tempfile::tempdir;

fn create_configs() -> (DataFoldNode, NetworkConfig) {
    let dir = tempdir().unwrap();
    let node_config = NodeConfig {
        storage_path: dir.path().to_path_buf(),
        default_trust_distance: 1,
        network_listen_address: "/ip4/127.0.0.1/tcp/0".to_string(),
    };
    let network_config = NetworkConfig {
        listen_address: "127.0.0.1:0".parse().unwrap(),
        request_timeout: 1,
        enable_mdns: false,
        max_connections: 5,
        keep_alive_interval: 1,
        max_message_size: 1024 * 1024,
        discovery_port: 0,
        connection_timeout: Duration::from_secs(1),
        announcement_interval: Duration::from_secs(1),
    };
    (DataFoldNode::new(node_config).unwrap(), network_config)
}

#[tokio::test]
async fn init_start_stop_network() {
    let (mut node, config) = create_configs();
    assert!(node.init_network(config).await.is_ok());
    assert!(node.start_network().await.is_ok());
    assert!(node.stop_network().await.is_ok());
}
