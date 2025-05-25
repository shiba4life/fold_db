//! Product Catalog Range Field Demo
//! 
//! This example demonstrates how to work with Range fields in a practical
//! product catalog scenario, showing inventory management and product
//! attribute querying using various range filter types.

use fold_node::schema::types::field::{RangeField, RangeFilter, RangeFilterResult};
use fold_node::fees::types::config::FieldPaymentConfig;
use fold_node::permissions::types::policy::PermissionsPolicy;
use std::collections::HashMap;
use serde_json::json;

fn main() {
    println!("=== Product Catalog Range Field Demo ===\n");

    // Create inventory and attributes range fields
    let mut inventory_field = create_inventory_field();
    let mut attributes_field = create_attributes_field();

    // Populate with sample data
    populate_inventory_data(&mut inventory_field);
    populate_attributes_data(&mut attributes_field);

    println!("Sample data loaded:");
    println!("- Inventory locations: {}", inventory_field.count());
    println!("- Product attributes: {}\n", attributes_field.count());

    // Demonstrate inventory queries
    demonstrate_inventory_queries(&inventory_field);
    
    // Demonstrate attribute queries
    demonstrate_attribute_queries(&attributes_field);
    
    // Demonstrate advanced filtering patterns
    demonstrate_advanced_patterns(&inventory_field, &attributes_field);
}

fn create_inventory_field() -> RangeField {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "inventory_manager".to_string();

    RangeField::new_with_range(permission_policy, payment_config, field_mappers, source_pub_key)
}

fn create_attributes_field() -> RangeField {
    let permission_policy = PermissionsPolicy::default();
    let payment_config = FieldPaymentConfig::default();
    let field_mappers = HashMap::new();
    let source_pub_key = "product_manager".to_string();

    RangeField::new_with_range(permission_policy, payment_config, field_mappers, source_pub_key)
}

fn populate_inventory_data(field: &mut RangeField) {
    if let Some(atom_ref_range) = field.atom_ref_range_mut() {
        // Gaming Laptop inventory
        atom_ref_range.set_atom_uuid("warehouse:north".to_string(), "25".to_string());
        atom_ref_range.set_atom_uuid("warehouse:south".to_string(), "18".to_string());
        atom_ref_range.set_atom_uuid("warehouse:east".to_string(), "32".to_string());
        atom_ref_range.set_atom_uuid("warehouse:west".to_string(), "12".to_string());
        atom_ref_range.set_atom_uuid("store:downtown".to_string(), "5".to_string());
        atom_ref_range.set_atom_uuid("store:mall".to_string(), "8".to_string());
        atom_ref_range.set_atom_uuid("store:outlet".to_string(), "3".to_string());
        
        // Additional locations for demonstration
        atom_ref_range.set_atom_uuid("distribution:center1".to_string(), "100".to_string());
        atom_ref_range.set_atom_uuid("distribution:center2".to_string(), "75".to_string());
        atom_ref_range.set_atom_uuid("online:reserved".to_string(), "50".to_string());
    }
}

fn populate_attributes_data(field: &mut RangeField) {
    if let Some(atom_ref_range) = field.atom_ref_range_mut() {
        // Gaming Laptop attributes
        atom_ref_range.set_atom_uuid("brand".to_string(), "TechCorp".to_string());
        atom_ref_range.set_atom_uuid("model".to_string(), "GX-2024".to_string());
        atom_ref_range.set_atom_uuid("cpu".to_string(), "Intel i7-13700H".to_string());
        atom_ref_range.set_atom_uuid("gpu".to_string(), "RTX 4060".to_string());
        atom_ref_range.set_atom_uuid("ram".to_string(), "16GB DDR5".to_string());
        atom_ref_range.set_atom_uuid("storage".to_string(), "1TB NVMe SSD".to_string());
        atom_ref_range.set_atom_uuid("display".to_string(), "15.6 inch 144Hz".to_string());
        atom_ref_range.set_atom_uuid("weight".to_string(), "2.3kg".to_string());
        atom_ref_range.set_atom_uuid("color".to_string(), "Black".to_string());
        atom_ref_range.set_atom_uuid("warranty".to_string(), "2 years".to_string());
        atom_ref_range.set_atom_uuid("connectivity:wifi".to_string(), "Wi-Fi 6E".to_string());
        atom_ref_range.set_atom_uuid("connectivity:bluetooth".to_string(), "Bluetooth 5.2".to_string());
        atom_ref_range.set_atom_uuid("connectivity:ports".to_string(), "USB-C, HDMI, USB-A".to_string());
    }
}

fn demonstrate_inventory_queries(field: &RangeField) {
    println!("=== Inventory Management Queries ===\n");

    // Query 1: Check specific warehouse inventory
    println!("1. Inventory at North Warehouse:");
    let filter = RangeFilter::Key("warehouse:north".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 2: Get all warehouse inventory
    println!("2. All Warehouse Inventory:");
    let filter = RangeFilter::KeyPrefix("warehouse:".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 3: Get all retail store inventory
    println!("3. All Retail Store Inventory:");
    let filter = RangeFilter::KeyPrefix("store:".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 4: Get inventory for specific locations
    println!("4. Inventory for Key Locations:");
    let filter = RangeFilter::Keys(vec![
        "warehouse:north".to_string(),
        "store:downtown".to_string(),
        "online:reserved".to_string(),
    ]);
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 5: Find distribution centers using pattern
    println!("5. Distribution Centers (Pattern Matching):");
    let filter = RangeFilter::KeyPattern("distribution:*".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 6: Find locations with low inventory (value-based search)
    println!("6. Locations with 3 Units (Low Inventory):");
    let filter = RangeFilter::Value("3".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);
}

fn demonstrate_attribute_queries(field: &RangeField) {
    println!("=== Product Attribute Queries ===\n");

    // Query 1: Get brand information
    println!("1. Product Brand:");
    let filter = RangeFilter::Key("brand".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 2: Get all hardware specifications
    println!("2. Hardware Specifications (CPU, GPU, RAM):");
    let filter = RangeFilter::Keys(vec![
        "cpu".to_string(),
        "gpu".to_string(),
        "ram".to_string(),
        "storage".to_string(),
    ]);
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 3: Get connectivity options using pattern
    println!("3. Connectivity Options:");
    let filter = RangeFilter::KeyPattern("connectivity:*".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 4: Find products by TechCorp brand
    println!("4. Products by TechCorp (Value Search):");
    let filter = RangeFilter::Value("TechCorp".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 5: Get display and physical attributes
    println!("5. Display and Physical Attributes:");
    let filter = RangeFilter::Keys(vec![
        "display".to_string(),
        "weight".to_string(),
        "color".to_string(),
    ]);
    let result = field.apply_filter(&filter);
    print_result(&result);

    // Query 6: Find warranty information
    println!("6. Warranty Information:");
    let filter = RangeFilter::KeyPattern("*warranty*".to_string());
    let result = field.apply_filter(&filter);
    print_result(&result);
}

fn demonstrate_advanced_patterns(inventory_field: &RangeField, attributes_field: &RangeField) {
    println!("=== Advanced Filtering Patterns ===\n");

    // Pattern 1: Range-based key filtering
    println!("1. Warehouses in Alphabetical Range (east to south):");
    let filter = RangeFilter::KeyRange {
        start: "warehouse:east".to_string(),
        end: "warehouse:south".to_string(),
    };
    let result = inventory_field.apply_filter(&filter);
    print_result(&result);

    // Pattern 2: Complex pattern matching
    println!("2. All Storage-Related Attributes:");
    let filter = RangeFilter::KeyPattern("*storage*".to_string());
    let result = attributes_field.apply_filter(&filter);
    print_result(&result);

    // Pattern 3: JSON filter demonstration
    println!("3. JSON Filter Example (Connectivity Prefix):");
    let json_filter = json!({
        "KeyPrefix": "connectivity:"
    });
    match attributes_field.apply_json_filter(&json_filter) {
        Ok(result) => print_result(&result),
        Err(e) => println!("Error: {}", e),
    }

    // Pattern 4: Utility functions
    println!("4. Utility Functions:");
    println!("   All inventory keys: {:?}", inventory_field.get_all_keys());
    println!("   Warehouse keys only: {:?}", 
             inventory_field.get_keys_in_range("warehouse:", "warehouse:z"));
    println!("   Total inventory locations: {}", inventory_field.count());
    println!("   Total product attributes: {}", attributes_field.count());

    // Pattern 5: Performance comparison
    println!("\n5. Performance Comparison:");
    demonstrate_performance_patterns(inventory_field);
}

fn demonstrate_performance_patterns(field: &RangeField) {
    use std::time::Instant;

    // Specific key lookup (fastest)
    let start = Instant::now();
    let filter = RangeFilter::Key("warehouse:north".to_string());
    let _result = field.apply_filter(&filter);
    let duration = start.elapsed();
    println!("   Specific key lookup: {:?}", duration);

    // Prefix search (fast)
    let start = Instant::now();
    let filter = RangeFilter::KeyPrefix("warehouse:".to_string());
    let _result = field.apply_filter(&filter);
    let duration = start.elapsed();
    println!("   Prefix search: {:?}", duration);

    // Pattern matching (moderate)
    let start = Instant::now();
    let filter = RangeFilter::KeyPattern("*house:*".to_string());
    let _result = field.apply_filter(&filter);
    let duration = start.elapsed();
    println!("   Pattern matching: {:?}", duration);

    // Value search (slower - scans all values)
    let start = Instant::now();
    let filter = RangeFilter::Value("25".to_string());
    let _result = field.apply_filter(&filter);
    let duration = start.elapsed();
    println!("   Value search: {:?}", duration);
}

fn print_result(result: &RangeFilterResult) {
    println!("   Found {} matches:", result.total_count);
    for (key, value) in &result.matches {
        println!("     {} -> {}", key, value);
    }
    println!();
}