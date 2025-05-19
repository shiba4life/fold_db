use fold_node::datafold_node::load_node_config;
use std::env;

#[test]
fn default_when_file_missing_with_port() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing.json");
    let config = load_node_config(Some(missing.to_str().unwrap()), Some(1234));
    assert_eq!(config.network_listen_address, "/ip4/0.0.0.0/tcp/1234");
}

#[test]
fn default_when_env_missing_file() {
    let tmp = tempfile::tempdir().unwrap();
    let missing = tmp.path().join("missing2.json");
    env::set_var("NODE_CONFIG", missing.to_str().unwrap());
    let config = load_node_config(None, None);
    env::remove_var("NODE_CONFIG");
    assert_eq!(config.storage_path, std::path::PathBuf::from("data"));
}
