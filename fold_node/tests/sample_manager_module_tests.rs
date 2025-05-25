use fold_node::datafold_node::sample_manager::SampleManager;

#[tokio::test]
async fn sample_manager_loads_schemas() {
    let manager = SampleManager::new().await.expect("failed to load samples");
    let schemas = manager.list_schema_samples();
    assert!(schemas.contains(&"UserProfile".to_string()));
    assert!(schemas.contains(&"ProductCatalog".to_string()));
    assert!(schemas.contains(&"UserProfileView".to_string()));
    assert!(schemas.contains(&"BlogPostSummary".to_string()));
}
