use fold_node::datafold_node::sample_manager::SampleManager;

#[tokio::test]
async fn list_sample_schemas_empty() {
    let manager = SampleManager::new().await.expect("failed to load samples");
    let schemas = manager.list_schema_samples();
    assert!(!schemas.is_empty());
}
