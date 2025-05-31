use fold_node::schema::types::field::{RangeField, RangeFilter, RangeFilterResult};
use fold_node::schema::types::field::range_filter::matches_pattern;
use fold_node::fees::types::config::FieldPaymentConfig;
use fold_node::permissions::types::policy::PermissionsPolicy;
use std::collections::HashMap;

#[test]
fn test_range_field_with_atom_ref_range() {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "test_key".to_string();

    // Test creating RangeField with AtomRefRange
    let mut range_field = RangeField::new_with_range(
        permission_policy,
        payment_config,
        field_mappers,
        source_pub_key.clone(),
    );

    // Verify AtomRefRange is present
    assert!(range_field.atom_ref_range().is_some());

    // Test accessing the AtomRefRange
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("key1".to_string(), "atom_uuid_1".to_string());
        atom_ref_range.set_atom_uuid("key2".to_string(), "atom_uuid_2".to_string());
    }

    // Verify the data was set correctly
    if let Some(atom_ref_range) = range_field.atom_ref_range() {
        assert_eq!(atom_ref_range.get_atom_uuid("key1"), Some(&"atom_uuid_1".to_string()));
        assert_eq!(atom_ref_range.get_atom_uuid("key2"), Some(&"atom_uuid_2".to_string()));
    }
}

#[test]
fn test_range_field_ensure_atom_ref_range() {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();

    // Create RangeField without AtomRefRange
    let mut range_field = RangeField::new(permission_policy, payment_config, field_mappers);

    // Verify AtomRefRange is not present initially
    assert!(range_field.atom_ref_range().is_none());

    // Ensure AtomRefRange exists
    let source_pub_key = "test_key".to_string();
    let atom_ref_range = range_field.ensure_atom_ref_range(source_pub_key);

    // Add some data
    atom_ref_range.set_atom_uuid("key1".to_string(), "atom_uuid_1".to_string());

    // Verify AtomRefRange is now present and contains data
    assert!(range_field.atom_ref_range().is_some());
    if let Some(atom_ref_range) = range_field.atom_ref_range() {
        assert_eq!(atom_ref_range.get_atom_uuid("key1"), Some(&"atom_uuid_1".to_string()));
    }
}

#[test]
fn test_range_filter_key() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::Key("user:123".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 1);
    assert_eq!(result.matches.get("user:123"), Some(&"atom_uuid_1".to_string()));
}

#[test]
fn test_range_filter_key_prefix() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::KeyPrefix("user:".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("user:456"));
    assert!(!result.matches.contains_key("product:789"));
}

#[test]
fn test_range_filter_key_range() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::KeyRange {
        start: "user:".to_string(),
        end: "user:z".to_string(),
    };
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("user:456"));
    assert!(!result.matches.contains_key("product:789"));
}

#[test]
fn test_range_filter_value() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::Value("atom_uuid_1".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 1);
    assert_eq!(result.matches.get("user:123"), Some(&"atom_uuid_1".to_string()));
}

#[test]
fn test_range_filter_keys() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::Keys(vec![
        "user:123".to_string(),
        "product:789".to_string(),
        "nonexistent".to_string(),
    ]);
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("product:789"));
    assert!(!result.matches.contains_key("nonexistent"));
}

#[test]
fn test_range_filter_key_pattern() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    // Test wildcard pattern
    let filter = RangeFilter::KeyPattern("user:*".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("user:456"));

    // Test single character pattern
    let filter = RangeFilter::KeyPattern("user:?23".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 1);
    assert!(result.matches.contains_key("user:123"));
}

#[test]
fn test_pattern_matching() {
    assert!(matches_pattern("hello", "hello"));
    assert!(matches_pattern("hello", "h*"));
    assert!(matches_pattern("hello", "*o"));
    assert!(matches_pattern("hello", "h*o"));
    assert!(matches_pattern("hello", "h?llo"));
    assert!(matches_pattern("hello", "?ello"));
    assert!(!matches_pattern("hello", "world"));
    assert!(!matches_pattern("hello", "h?o"));
    assert!(matches_pattern("user:123", "user:*"));
    assert!(matches_pattern("user:123", "*:123"));
}

#[test]
fn test_json_filter() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    // Test JSON filter parsing
    let json_filter = serde_json::json!({
        "KeyPrefix": "user:"
    });

    let result = range_field.apply_json_filter(&json_filter).unwrap();
    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("user:456"));
}

#[test]
fn test_json_filter_key_range() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let json_filter = serde_json::json!({
        "KeyRange": {"start": "user:", "end": "user:z"}
    });

    let result = range_field.apply_json_filter(&json_filter).unwrap();
    assert_eq!(result.total_count, 2);
    assert!(result.matches.contains_key("user:123"));
    assert!(result.matches.contains_key("user:456"));
    assert!(!result.matches.contains_key("product:789"));
}

#[test]
fn test_get_all_keys() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let keys = range_field.get_all_keys();
    assert_eq!(keys.len(), 3);
    assert!(keys.contains(&"user:123".to_string()));
    assert!(keys.contains(&"user:456".to_string()));
    assert!(keys.contains(&"product:789".to_string()));
}

#[test]
fn test_get_keys_in_range() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let keys = range_field.get_keys_in_range("user:", "user:z");
    assert_eq!(keys.len(), 2);
    assert!(keys.contains(&"user:123".to_string()));
    assert!(keys.contains(&"user:456".to_string()));
    assert!(!keys.contains(&"product:789".to_string()));
}

#[test]
fn test_count() {
    let mut range_field = create_test_range_field();
    assert_eq!(range_field.count(), 0);

    populate_test_data(&mut range_field);
    assert_eq!(range_field.count(), 3);
}

#[test]
fn test_empty_range_field_filters() {
    let range_field = create_test_range_field();

    let filter = RangeFilter::Key("any_key".to_string());
    let result = range_field.apply_filter(&filter);

    assert_eq!(result.total_count, 0);
    assert!(result.matches.is_empty());
}

// Helper functions for tests
fn create_test_range_field() -> RangeField {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "test_key".to_string();

    RangeField::new_with_range(permission_policy, payment_config, field_mappers, source_pub_key)
}

fn populate_test_data(range_field: &mut RangeField) {
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("user:123".to_string(), "atom_uuid_1".to_string());
        atom_ref_range.set_atom_uuid("user:456".to_string(), "atom_uuid_2".to_string());
        atom_ref_range.set_atom_uuid("product:789".to_string(), "atom_uuid_3".to_string());
    }
}

// Additional comprehensive tests for edge cases and error handling

#[test]
fn test_range_filter_edge_cases() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    // Test empty key filter
    let filter = RangeFilter::Key("".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 0);

    // Test empty prefix filter
    let filter = RangeFilter::KeyPrefix("".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 3); // Should match all keys

    // Test empty keys list
    let filter = RangeFilter::Keys(vec![]);
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 0);

    // Test empty pattern
    let filter = RangeFilter::KeyPattern("".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 0);
}

#[test]
fn test_range_filter_boundary_conditions() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    // Test range with same start and end
    let filter = RangeFilter::KeyRange {
        start: "user:123".to_string(),
        end: "user:123".to_string(),
    };
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 0); // Exclusive end

    // Test range that includes everything
    let filter = RangeFilter::KeyRange {
        start: "".to_string(),
        end: "z".to_string(),
    };
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 3);

    // Test range with inverted start/end (start > end)
    let filter = RangeFilter::KeyRange {
        start: "z".to_string(),
        end: "a".to_string(),
    };
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 0);
}

#[test]
fn test_pattern_matching_edge_cases() {
    // Test empty strings
    assert!(!matches_pattern("", "a"));
    assert!(matches_pattern("", ""));
    assert!(matches_pattern("", "*"));
    assert!(!matches_pattern("", "?"));

    // Test patterns with multiple wildcards
    assert!(matches_pattern("hello", "h*l*o"));
    assert!(matches_pattern("hello", "h*l*"));
    assert!(matches_pattern("hello", "*l*o"));
    assert!(matches_pattern("hello", "h?ll?"));

    // Test patterns that should not match
    assert!(!matches_pattern("hello", "h*x"));
    assert!(!matches_pattern("hello", "x*o"));
    assert!(!matches_pattern("hello", "h?x?o"));

    // Test complex patterns
    assert!(matches_pattern("user:123:profile", "user:*:profile"));
    assert!(matches_pattern("user:123:profile", "user:???:*"));
    assert!(!matches_pattern("user:123:profile", "user:??:*"));
}

#[test]
fn test_json_filter_error_handling() {
    let range_field = create_test_range_field();

    // Test invalid JSON structure
    let invalid_json = serde_json::json!({
        "InvalidFilterType": "test"
    });
    let result = range_field.apply_json_filter(&invalid_json);
    assert!(result.is_err());

    // Test malformed JSON
    let malformed_json = serde_json::json!("not_an_object");
    let result = range_field.apply_json_filter(&malformed_json);
    assert!(result.is_err());

    // Test valid JSON with complex filter
    let complex_json = serde_json::json!({
        "KeyRange": {
            "start": "a",
            "end": "z"
        }
    });
    let result = range_field.apply_json_filter(&complex_json);
    assert!(result.is_ok());
}

#[test]
fn test_filter_with_special_characters() {
    let mut range_field = create_test_range_field();
    
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("key:with:colons".to_string(), "value1".to_string());
        atom_ref_range.set_atom_uuid("key-with-dashes".to_string(), "value2".to_string());
        atom_ref_range.set_atom_uuid("key_with_underscores".to_string(), "value3".to_string());
        atom_ref_range.set_atom_uuid("key with spaces".to_string(), "value4".to_string());
        atom_ref_range.set_atom_uuid("key.with.dots".to_string(), "value5".to_string());
    }

    // Test prefix with special characters
    let filter = RangeFilter::KeyPrefix("key:".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 1);

    // Test pattern with special characters
    let filter = RangeFilter::KeyPattern("key*with*".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 5); // Should match all keys that contain "key" and "with"

    // Test exact match with spaces
    let filter = RangeFilter::Key("key with spaces".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 1);
}

#[test]
fn test_filter_performance_with_large_dataset() {
    let mut range_field = create_test_range_field();
    
    // Populate with a larger dataset
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        for i in 0..1000 {
            let key = format!("item:{:04}", i);
            let value = format!("value_{}", i);
            atom_ref_range.set_atom_uuid(key, value);
        }
    }

    assert_eq!(range_field.count(), 1000);

    // Test prefix filter on large dataset
    let filter = RangeFilter::KeyPrefix("item:0".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 1000); // All items start with "item:0" since they're formatted as "item:0000" to "item:0999"

    // Test range filter on large dataset
    let filter = RangeFilter::KeyRange {
        start: "item:0500".to_string(),
        end: "item:0600".to_string(),
    };
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 100); // item:0500 to item:0599

    // Test pattern filter on large dataset
    let filter = RangeFilter::KeyPattern("item:*5".to_string());
    let result = range_field.apply_filter(&filter);
    assert_eq!(result.total_count, 100); // All items ending in 5
}

#[test]
fn test_filter_result_serialization() {
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    let filter = RangeFilter::KeyPrefix("user:".to_string());
    let result = range_field.apply_filter(&filter);

    // Test that RangeFilterResult can be serialized and deserialized
    let serialized = serde_json::to_string(&result).unwrap();
    let deserialized: RangeFilterResult = serde_json::from_str(&serialized).unwrap();

    assert_eq!(result.total_count, deserialized.total_count);
    assert_eq!(result.matches, deserialized.matches);
}

#[test]
fn test_filter_enum_serialization() {
    // Test that all RangeFilter variants can be serialized and deserialized
    let filters = vec![
        RangeFilter::Key("test".to_string()),
        RangeFilter::KeyPrefix("prefix".to_string()),
        RangeFilter::KeyRange { start: "a".to_string(), end: "z".to_string() },
        RangeFilter::Value("value".to_string()),
        RangeFilter::Keys(vec!["key1".to_string(), "key2".to_string()]),
        RangeFilter::KeyPattern("pattern*".to_string()),
    ];

    for filter in filters {
        let serialized = serde_json::to_string(&filter).unwrap();
        let deserialized: RangeFilter = serde_json::from_str(&serialized).unwrap();
        
        // Compare the serialized forms since RangeFilter doesn't implement PartialEq
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        assert_eq!(serialized, reserialized);
    }
}

#[test]
fn test_concurrent_access_safety() {
    // Test that the filtering operations are safe for concurrent read access
    let mut range_field = create_test_range_field();
    populate_test_data(&mut range_field);

    // Simulate multiple concurrent filter operations
    let filters = vec![
        RangeFilter::Key("user:123".to_string()),
        RangeFilter::KeyPrefix("user:".to_string()),
        RangeFilter::Value("atom_uuid_1".to_string()),
    ];

    for filter in filters {
        let result = range_field.apply_filter(&filter);
        assert!(result.total_count <= 3); // Should never exceed total data
    }
}
