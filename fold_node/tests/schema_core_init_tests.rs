use fold_node::schema::SchemaCore;
use tempfile::NamedTempFile;

#[test]
fn test_new_invalid_path() {
    let file = NamedTempFile::new().unwrap();
    let result = SchemaCore::new(file.path().to_str().unwrap());
    assert!(result.is_err());
}
