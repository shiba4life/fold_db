use fold_node::network::schema_service::SchemaService;

#[test]
fn test_schema_service() {
    let mut service = SchemaService::new();

    // Default callback should return empty list
    let result = service.check_schemas(&["schema1".to_string(), "schema2".to_string()]);
    assert!(result.is_empty());

    // Set custom callback
    service.set_schema_check_callback(|names| {
        names
            .iter()
            .filter(|name| name.contains("1"))
            .cloned()
            .collect()
    });

    // Should now return only schemas containing "1"
    let result = service.check_schemas(&[
        "schema1".to_string(),
        "schema2".to_string(),
        "test1".to_string(),
    ]);

    assert_eq!(result, vec!["schema1".to_string(), "test1".to_string()]);
}
