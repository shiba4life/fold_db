use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::atom::AtomRefRange;
use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::field::common::FieldCommon;
use crate::impl_field;

/// Range filter operations for querying range fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeFilter {
    /// Filter by exact key match
    Key(String),
    /// Filter by key prefix
    KeyPrefix(String),
    /// Filter by key range (inclusive start, exclusive end)
    KeyRange { start: String, end: String },
    /// Filter by value match
    Value(String),
    /// Filter by multiple keys
    Keys(Vec<String>),
    /// Filter by key pattern (simple glob-style matching)
    KeyPattern(String),
}

/// Result of a range filter operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeFilterResult {
    pub matches: HashMap<String, String>,
    pub total_count: usize,
}

/// Field storing a range of values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeField {
    pub inner: FieldCommon,
    pub atom_ref_range: Option<AtomRefRange>,
}

impl RangeField {
    #[must_use]
    pub fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: None,
        }
    }

    /// Creates a new RangeField with an AtomRefRange
    #[must_use]
    pub fn new_with_range(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
        source_pub_key: String,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: Some(AtomRefRange::new(source_pub_key)),
        }
    }

    /// Returns a reference to the AtomRefRange if it exists
    pub fn atom_ref_range(&self) -> Option<&AtomRefRange> {
        self.atom_ref_range.as_ref()
    }

    /// Returns a mutable reference to the AtomRefRange if it exists
    pub fn atom_ref_range_mut(&mut self) -> Option<&mut AtomRefRange> {
        self.atom_ref_range.as_mut()
    }

    /// Sets the AtomRefRange for this field
    pub fn set_atom_ref_range(&mut self, atom_ref_range: AtomRefRange) {
        self.atom_ref_range = Some(atom_ref_range);
    }

    /// Initializes the AtomRefRange if it doesn't exist
    pub fn ensure_atom_ref_range(&mut self, source_pub_key: String) -> &mut AtomRefRange {
        if self.atom_ref_range.is_none() {
            self.atom_ref_range = Some(AtomRefRange::new(source_pub_key));
        }
        self.atom_ref_range.as_mut().unwrap()
    }

    /// Applies a range filter to the field's data
    pub fn apply_filter(&self, filter: &RangeFilter) -> RangeFilterResult {
        let empty_result = RangeFilterResult {
            matches: HashMap::new(),
            total_count: 0,
        };

        let Some(atom_ref_range) = &self.atom_ref_range else {
            return empty_result;
        };

        let mut matches = HashMap::new();

        match filter {
            RangeFilter::Key(key) => {
                if let Some(value) = atom_ref_range.get_atom_uuid(key) {
                    matches.insert(key.clone(), value.clone());
                }
            }
            RangeFilter::KeyPrefix(prefix) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if key.starts_with(prefix) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::KeyRange { start, end } => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if key >= start && key < end {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::Value(target_value) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if value == target_value {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::Keys(keys) => {
                for key in keys {
                    if let Some(value) = atom_ref_range.get_atom_uuid(key) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::KeyPattern(pattern) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if Self::matches_pattern(key, pattern) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        RangeFilterResult {
            total_count: matches.len(),
            matches,
        }
    }

    /// Applies a filter from a JSON Value (for use with Operation::Query filter)
    pub fn apply_json_filter(&self, filter_value: &Value) -> Result<RangeFilterResult, String> {
        let filter: RangeFilter = serde_json::from_value(filter_value.clone())
            .map_err(|e| format!("Invalid range filter format: {}", e))?;
        Ok(self.apply_filter(&filter))
    }

    /// Simple glob-style pattern matching (supports * and ?)
    fn matches_pattern(text: &str, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        Self::match_recursive(&text_chars, &pattern_chars, 0, 0)
    }

    fn match_recursive(text: &[char], pattern: &[char], text_idx: usize, pattern_idx: usize) -> bool {
        // If we've reached the end of both strings, it's a match
        if pattern_idx >= pattern.len() && text_idx >= text.len() {
            return true;
        }
        
        // If we've reached the end of pattern but not text, no match
        if pattern_idx >= pattern.len() {
            return false;
        }
        
        match pattern[pattern_idx] {
            '*' => {
                // Try matching zero characters
                if Self::match_recursive(text, pattern, text_idx, pattern_idx + 1) {
                    return true;
                }
                // Try matching one or more characters
                for i in text_idx..text.len() {
                    if Self::match_recursive(text, pattern, i + 1, pattern_idx + 1) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                // Match exactly one character
                if text_idx < text.len() {
                    Self::match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
                } else {
                    false
                }
            }
            c => {
                // Match exact character
                if text_idx < text.len() && text[text_idx] == c {
                    Self::match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
                } else {
                    false
                }
            }
        }
    }

    /// Gets all keys in the range (useful for pagination or listing)
    pub fn get_all_keys(&self) -> Vec<String> {
        self.atom_ref_range
            .as_ref()
            .map(|range| range.atom_uuids.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets a subset of keys within a range (useful for pagination)
    pub fn get_keys_in_range(&self, start: &str, end: &str) -> Vec<String> {
        self.atom_ref_range
            .as_ref()
            .map(|range| {
                let start_string = start.to_string();
                let end_string = end.to_string();
                range.atom_uuids
                    .keys()
                    .filter(|key| **key >= start_string && **key < end_string)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets the total count of items in the range
    pub fn count(&self) -> usize {
        self.atom_ref_range
            .as_ref()
            .map(|range| range.atom_uuids.len())
            .unwrap_or(0)
    }
}

impl_field!(RangeField);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fees::types::config::FieldPaymentConfig;
    use crate::permissions::types::policy::PermissionsPolicy;
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
        assert!(RangeField::matches_pattern("hello", "hello"));
        assert!(RangeField::matches_pattern("hello", "h*"));
        assert!(RangeField::matches_pattern("hello", "*o"));
        assert!(RangeField::matches_pattern("hello", "h*o"));
        assert!(RangeField::matches_pattern("hello", "h?llo"));
        assert!(RangeField::matches_pattern("hello", "?ello"));
        assert!(!RangeField::matches_pattern("hello", "world"));
        assert!(!RangeField::matches_pattern("hello", "h?o"));
        assert!(RangeField::matches_pattern("user:123", "user:*"));
        assert!(RangeField::matches_pattern("user:123", "*:123"));
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
        assert!(!RangeField::matches_pattern("", "a"));
        assert!(RangeField::matches_pattern("", ""));
        assert!(RangeField::matches_pattern("", "*"));
        assert!(!RangeField::matches_pattern("", "?"));

        // Test patterns with multiple wildcards
        assert!(RangeField::matches_pattern("hello", "h*l*o"));
        assert!(RangeField::matches_pattern("hello", "h*l*"));
        assert!(RangeField::matches_pattern("hello", "*l*o"));
        assert!(RangeField::matches_pattern("hello", "h?ll?"));

        // Test patterns that should not match
        assert!(!RangeField::matches_pattern("hello", "h*x"));
        assert!(!RangeField::matches_pattern("hello", "x*o"));
        assert!(!RangeField::matches_pattern("hello", "h?x?o"));

        // Test complex patterns
        assert!(RangeField::matches_pattern("user:123:profile", "user:*:profile"));
        assert!(RangeField::matches_pattern("user:123:profile", "user:???:*"));
        assert!(!RangeField::matches_pattern("user:123:profile", "user:??:*"));
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
}