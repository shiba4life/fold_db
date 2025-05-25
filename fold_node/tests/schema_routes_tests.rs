use fold_node::datafold_node::{DataFoldNode, config::NodeConfig};
use tempfile::tempdir;

#[test]
fn list_schemas_empty() {
    let dir = tempdir().unwrap();
    let config = NodeConfig::new(dir.path().to_path_buf());
    let node = DataFoldNode::new(config).unwrap();
    let schemas = node.list_schemas().unwrap();
    assert!(schemas.is_empty());
}
