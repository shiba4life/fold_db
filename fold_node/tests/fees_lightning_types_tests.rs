use fold_node::fees::types::lightning::{NodeConfig, Network};

#[test]
fn test_node_config() {
    let config = NodeConfig::new(
        "127.0.0.1".to_string(),
        9735,
        "/path/to/macaroon".to_string(),
        "/path/to/tls.cert".to_string(),
        Network::Mainnet,
    );

    assert_eq!(config.get_connection_string(), "127.0.0.1:9735");
    assert!(matches!(config.network, Network::Mainnet));
}

#[test]
fn test_network_display() {
    assert_eq!(Network::Mainnet.to_string(), "mainnet");
    assert_eq!(Network::Testnet.to_string(), "testnet");
    assert_eq!(Network::Regtest.to_string(), "regtest");
}

#[test]
fn test_network_default() {
    assert!(matches!(Network::default(), Network::Mainnet));
}
