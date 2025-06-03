//! Example demonstrating RangeField filtering functionality
//!
//! This example shows how to use the new range filtering capabilities
//! added to RangeField for querying and filtering range-based data.

use fold_node::fees::types::config::FieldPaymentConfig;
use fold_node::permissions::types::policy::PermissionsPolicy;
use fold_node::schema::types::field::{RangeField, RangeFilter, RangeFilterResult};
use serde_json::json;
use std::collections::HashMap;

fn main() {
    println!("=== RangeField Filtering Example ===\n");

    // Create a new RangeField
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "example_node".to_string();

    let mut range_field = RangeField::new_with_range(
        permission_policy,
        payment_config,
        field_mappers,
        source_pub_key,
    );

    // Populate with sample data
    if let Some(atom_ref_range) = range_field.atom_ref_range_mut() {
        atom_ref_range.set_atom_uuid("user:alice".to_string(), "atom_uuid_alice".to_string());
        atom_ref_range.set_atom_uuid("user:bob".to_string(), "atom_uuid_bob".to_string());
        atom_ref_range.set_atom_uuid("user:charlie".to_string(), "atom_uuid_charlie".to_string());
        atom_ref_range.set_atom_uuid("product:laptop".to_string(), "atom_uuid_laptop".to_string());
        atom_ref_range.set_atom_uuid("product:phone".to_string(), "atom_uuid_phone".to_string());
        atom_ref_range.set_atom_uuid("order:12345".to_string(), "atom_uuid_order1".to_string());
        atom_ref_range.set_atom_uuid("order:67890".to_string(), "atom_uuid_order2".to_string());
    }

    println!("Sample data loaded. Total items: {}\n", range_field.count());

    // Example 1: Filter by exact key
    println!("1. Filter by exact key 'user:alice':");
    let filter = RangeFilter::Key("user:alice".to_string());
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 2: Filter by key prefix
    println!("2. Filter by key prefix 'user:':");
    let filter = RangeFilter::KeyPrefix("user:".to_string());
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 3: Filter by key range
    println!("3. Filter by key range 'product:' to 'product:z':");
    let filter = RangeFilter::KeyRange {
        start: "product:".to_string(),
        end: "product:z".to_string(),
    };
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 4: Filter by multiple specific keys
    println!("4. Filter by multiple keys:");
    let filter = RangeFilter::Keys(vec![
        "user:alice".to_string(),
        "product:laptop".to_string(),
        "order:12345".to_string(),
        "nonexistent:key".to_string(), // This won't match
    ]);
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 5: Filter by key pattern (glob-style)
    println!("5. Filter by key pattern 'user:*':");
    let filter = RangeFilter::KeyPattern("user:*".to_string());
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 6: Filter by key pattern with single character wildcard
    println!("6. Filter by key pattern 'order:????5':");
    let filter = RangeFilter::KeyPattern("order:????5".to_string());
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 7: Filter by value
    println!("7. Filter by value 'atom_uuid_bob':");
    let filter = RangeFilter::Value("atom_uuid_bob".to_string());
    let result = range_field.apply_filter(&filter);
    print_result(&result);

    // Example 8: JSON filter (for use with API queries)
    println!("8. JSON filter example:");
    let json_filter = json!({
        "KeyPrefix": "product:"
    });
    match range_field.apply_json_filter(&json_filter) {
        Ok(result) => print_result(&result),
        Err(e) => println!("Error: {}", e),
    }

    // Example 9: Utility functions
    println!("9. Utility functions:");
    println!("All keys: {:?}", range_field.get_all_keys());
    println!(
        "Keys in range 'user:' to 'user:z': {:?}",
        range_field.get_keys_in_range("user:", "user:z")
    );
    println!("Total count: {}", range_field.count());
}

fn print_result(result: &RangeFilterResult) {
    println!("  Found {} matches:", result.total_count);
    for (key, value) in &result.matches {
        println!("    {} -> {}", key, value);
    }
    println!();
}
