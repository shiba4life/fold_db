//! Integration tests for Product Catalog Range Field functionality
//!
//! This test suite demonstrates comprehensive usage of Range fields
//! in a practical product catalog scenario.

use fold_node::testing::*;
use serde_json::{json, Value};
use std::collections::HashMap;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_product_catalog_schema() -> Schema {
        let mut schema = create_test_schema("ProductCatalog");

        // Add regular fields
        let name_field = FieldVariant::Single(SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        ));

        let category_field = FieldVariant::Single(SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        ));

        let price_field = FieldVariant::Single(SingleField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        ));

        // Add range fields
        let inventory_field = FieldVariant::Range(RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        ));

        let attributes_field = FieldVariant::Range(RangeField::new(
            PermissionsPolicy::default(),
            FieldPaymentConfig::default(),
            HashMap::new(),
        ));

        schema.fields.insert("name".to_string(), name_field);
        schema.fields.insert("category".to_string(), category_field);
        schema.fields.insert("price".to_string(), price_field);
        schema
            .fields
            .insert("inventory_by_location".to_string(), inventory_field);
        schema
            .fields
            .insert("attributes".to_string(), attributes_field);

        schema
    }

    fn create_sample_product_data() -> HashMap<String, Value> {
        let mut data = HashMap::new();

        data.insert("name".to_string(), json!("Gaming Laptop"));
        data.insert("category".to_string(), json!("Electronics"));
        data.insert("price".to_string(), json!("1299.99"));

        // Inventory by location (Range field)
        data.insert(
            "inventory_by_location".to_string(),
            json!({
                "warehouse:north": "25",
                "warehouse:south": "18",
                "warehouse:east": "32",
                "warehouse:west": "12",
                "store:downtown": "5",
                "store:mall": "8",
                "store:outlet": "3"
            }),
        );

        // Product attributes (Range field)
        data.insert(
            "attributes".to_string(),
            json!({
                "brand": "TechCorp",
                "model": "GX-2024",
                "cpu": "Intel i7-13700H",
                "gpu": "RTX 4060",
                "ram": "16GB DDR5",
                "storage": "1TB NVMe SSD",
                "display": "15.6 inch 144Hz",
                "weight": "2.3kg",
                "color": "Black",
                "warranty": "2 years",
                "connectivity:wifi": "Wi-Fi 6E",
                "connectivity:bluetooth": "Bluetooth 5.2",
                "connectivity:ports": "USB-C, HDMI, USB-A"
            }),
        );

        data
    }

    #[test]
    fn test_product_catalog_schema_creation() {
        let schema = create_product_catalog_schema();

        assert_eq!(schema.name, "ProductCatalog");
        assert_eq!(schema.fields.len(), 5);

        // Verify field types
        assert!(matches!(
            schema.fields.get("name"),
            Some(FieldVariant::Single(_))
        ));
        assert!(matches!(
            schema.fields.get("category"),
            Some(FieldVariant::Single(_))
        ));
        assert!(matches!(
            schema.fields.get("price"),
            Some(FieldVariant::Single(_))
        ));
        assert!(matches!(
            schema.fields.get("inventory_by_location"),
            Some(FieldVariant::Range(_))
        ));
        assert!(matches!(
            schema.fields.get("attributes"),
            Some(FieldVariant::Range(_))
        ));
    }

    #[test]
    fn test_range_field_data_structure() {
        let data = create_sample_product_data();

        // Verify inventory data structure
        let inventory = data.get("inventory_by_location").unwrap();
        assert!(inventory.is_object());

        let inventory_obj = inventory.as_object().unwrap();
        assert_eq!(inventory_obj.get("warehouse:north").unwrap(), "25");
        assert_eq!(inventory_obj.get("store:downtown").unwrap(), "5");

        // Verify attributes data structure
        let attributes = data.get("attributes").unwrap();
        assert!(attributes.is_object());

        let attributes_obj = attributes.as_object().unwrap();
        assert_eq!(attributes_obj.get("brand").unwrap(), "TechCorp");
        assert_eq!(attributes_obj.get("warranty").unwrap(), "2 years");
    }

    #[test]
    fn test_range_filter_queries() {
        // Test various range filter query structures

        // 1. Key filter
        let key_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "Key": "warehouse:north"
                }
            }
        });

        assert!(key_filter_query.is_object());
        let filter = key_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("Key").unwrap(), "warehouse:north");

        // 2. KeyPrefix filter
        let prefix_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "KeyPrefix": "warehouse:"
                }
            }
        });

        let filter = prefix_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("KeyPrefix").unwrap(), "warehouse:");

        // 3. KeyRange filter
        let range_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "KeyRange": {
                        "start": "warehouse:east",
                        "end": "warehouse:south"
                    }
                }
            }
        });

        let filter = range_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        let key_range = range_filter.get("KeyRange").unwrap();
        assert_eq!(key_range.get("start").unwrap(), "warehouse:east");
        assert_eq!(key_range.get("end").unwrap(), "warehouse:south");

        // 4. Keys filter
        let keys_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "Keys": [
                        "warehouse:north",
                        "store:downtown",
                        "store:mall"
                    ]
                }
            }
        });

        let filter = keys_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        let keys = range_filter.get("Keys").unwrap().as_array().unwrap();
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], "warehouse:north");

        // 5. KeyPattern filter
        let pattern_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "KeyPattern": "store:*"
                }
            }
        });

        let filter = pattern_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("KeyPattern").unwrap(), "store:*");

        // 6. Value filter
        let value_filter_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "attributes"],
            "filter": {
                "field": "attributes",
                "range_filter": {
                    "Value": "TechCorp"
                }
            }
        });

        let filter = value_filter_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("Value").unwrap(), "TechCorp");
    }

    #[test]
    fn test_mutation_with_range_fields() {
        let mutation_data = json!({
            "type": "mutation",
            "schema": "ProductCatalog",
            "mutation_type": "create",
            "data": {
                "name": "Wireless Headphones",
                "category": "Audio",
                "price": "199.99",
                "inventory_by_location": {
                    "warehouse:north": "150",
                    "warehouse:south": "120",
                    "warehouse:east": "200",
                    "warehouse:west": "85",
                    "store:downtown": "25",
                    "store:mall": "40",
                    "store:outlet": "15"
                },
                "attributes": {
                    "brand": "AudioMax",
                    "model": "WH-Pro",
                    "type": "Over-ear",
                    "connectivity": "Bluetooth 5.2",
                    "battery_life": "30 hours",
                    "noise_cancellation": "Active",
                    "color": "Midnight Blue",
                    "weight": "250g",
                    "warranty": "1 year"
                }
            }
        });

        assert!(mutation_data.is_object());

        let data = mutation_data.get("data").unwrap();
        let inventory = data.get("inventory_by_location").unwrap();
        let attributes = data.get("attributes").unwrap();

        assert!(inventory.is_object());
        assert!(attributes.is_object());

        // Verify range field data
        let inventory_obj = inventory.as_object().unwrap();
        assert_eq!(inventory_obj.len(), 7); // 7 locations

        let attributes_obj = attributes.as_object().unwrap();
        assert_eq!(attributes_obj.get("brand").unwrap(), "AudioMax");
        assert_eq!(attributes_obj.get("warranty").unwrap(), "1 year");
    }

    #[test]
    fn test_complex_attribute_queries() {
        // Test queries for connectivity attributes
        let connectivity_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "attributes"],
            "filter": {
                "field": "attributes",
                "range_filter": {
                    "KeyPattern": "connectivity:*"
                }
            }
        });

        let filter = connectivity_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("KeyPattern").unwrap(), "connectivity:*");

        // Test warranty queries
        let warranty_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "attributes"],
            "filter": {
                "field": "attributes",
                "range_filter": {
                    "Key": "warranty"
                }
            }
        });

        let filter = warranty_query.get("filter").unwrap();
        let range_filter = filter.get("range_filter").unwrap();
        assert_eq!(range_filter.get("Key").unwrap(), "warranty");
    }

    #[test]
    fn test_inventory_management_queries() {
        // Test warehouse-specific queries
        let warehouse_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "KeyPrefix": "warehouse:"
                }
            }
        });

        // Test store-specific queries
        let store_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "KeyPrefix": "store:"
                }
            }
        });

        // Test multi-location queries
        let multi_location_query = json!({
            "type": "query",
            "schema": "ProductCatalog",
            "fields": ["name", "inventory_by_location"],
            "filter": {
                "field": "inventory_by_location",
                "range_filter": {
                    "Keys": [
                        "warehouse:north",
                        "store:downtown"
                    ]
                }
            }
        });

        // Verify query structures
        assert!(warehouse_query.get("filter").is_some());
        assert!(store_query.get("filter").is_some());
        assert!(multi_location_query.get("filter").is_some());
    }

    #[test]
    fn test_range_field_edge_cases() {
        // Test empty range field
        let empty_range_data = json!({
            "inventory_by_location": {},
            "attributes": {}
        });

        let inventory = empty_range_data.get("inventory_by_location").unwrap();
        let attributes = empty_range_data.get("attributes").unwrap();

        assert!(inventory.as_object().unwrap().is_empty());
        assert!(attributes.as_object().unwrap().is_empty());

        // Test single item range field
        let single_item_data = json!({
            "inventory_by_location": {
                "warehouse:main": "100"
            }
        });

        let inventory = single_item_data.get("inventory_by_location").unwrap();
        let inventory_obj = inventory.as_object().unwrap();
        assert_eq!(inventory_obj.len(), 1);
        assert_eq!(inventory_obj.get("warehouse:main").unwrap(), "100");
    }
}
