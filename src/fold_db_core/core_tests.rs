//! Core test suite for FoldDB operations
//!
//! This module contains integration tests for FoldDB operations,
//! test utilities and helpers for validating the split functionality.

#[cfg(test)]
mod tests {
    use super::super::coordinator::FoldDB;
    use super::super::operations::{EncryptionOperations, MutationOperations, QueryOperations};
    use crate::schema::types::{Mutation, Query};
    use crate::schema::{Schema, SchemaError};
    use serde_json::{json, Value};
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Test utilities for FoldDB operations
    pub struct TestUtils;

    impl TestUtils {
        /// Create a temporary FoldDB instance for testing
        pub fn create_test_db() -> (FoldDB, TempDir) {
            let temp_dir = TempDir::new().expect("Failed to create temp directory");
            let db_path = temp_dir.path().to_str().unwrap();
            let fold_db = FoldDB::new(db_path).expect("Failed to create FoldDB");
            (fold_db, temp_dir)
        }

        /// Create a simple test schema
        pub fn create_test_schema() -> Schema {
            use crate::schema::types::field::{FieldVariant, SingleField};
            use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};
            use crate::fees::types::config::FieldPaymentConfig;
            use std::collections::HashMap;

            let mut schema = Schema::new("test_schema".to_string());
            
            // Create field1 (String)
            let permission_policy1 = PermissionsPolicy::default();
            let payment_config1 = FieldPaymentConfig::default();
            let field1 = FieldVariant::Single(SingleField::new(
                permission_policy1,
                payment_config1,
                HashMap::new(),
            ));
            schema.fields.insert("field1".to_string(), field1);
            
            // Create field2 (Number)
            let permission_policy2 = PermissionsPolicy::default();
            let payment_config2 = FieldPaymentConfig::default();
            let field2 = FieldVariant::Single(SingleField::new(
                permission_policy2,
                payment_config2,
                HashMap::new(),
            ));
            schema.fields.insert("field2".to_string(), field2);
            
            schema
        }

        /// Create a test range schema
        pub fn create_test_range_schema() -> Schema {
            use crate::schema::types::field::{FieldVariant, RangeField};
            use crate::permissions::types::policy::{PermissionsPolicy, TrustDistance};
            use crate::fees::types::config::FieldPaymentConfig;
            use std::collections::HashMap;

            let mut schema = Schema::new_range("test_range_schema".to_string(), "id".to_string());
            
            // Create id field (Range String)
            let permission_policy_id = PermissionsPolicy::default();
            let payment_config_id = FieldPaymentConfig::default();
            let id_field = FieldVariant::Range(RangeField::new(
                permission_policy_id,
                payment_config_id,
                HashMap::new(),
            ));
            schema.fields.insert("id".to_string(), id_field);
            
            // Create value field (Range String)
            let permission_policy_value = PermissionsPolicy::default();
            let payment_config_value = FieldPaymentConfig::default();
            let value_field = FieldVariant::Range(RangeField::new(
                permission_policy_value,
                payment_config_value,
                HashMap::new(),
            ));
            schema.fields.insert("value".to_string(), value_field);
            
            schema
        }

        /// Create a test mutation
        pub fn create_test_mutation(schema_name: &str) -> Mutation {
            let mut fields_and_values = HashMap::new();
            fields_and_values.insert("field1".to_string(), json!("test_value"));
            fields_and_values.insert("field2".to_string(), json!(42));

            Mutation {
                schema_name: schema_name.to_string(),
                fields_and_values,
                pub_key: "test_pub_key".to_string(),
                trust_distance: 0,
                mutation_type: crate::schema::types::MutationType::Create,
            }
        }

        /// Create a test query
        pub fn create_test_query(schema_name: &str) -> Query {
            Query {
                schema_name: schema_name.to_string(),
                fields: vec!["field1".to_string(), "field2".to_string()],
                pub_key: "test_pub_key".to_string(),
                trust_distance: 0,
                filter: None,
            }
        }
    }

    #[test]
    fn test_folddb_initialization() {
        let (fold_db, _temp_dir) = TestUtils::create_test_db();
        
        // Test that all components are properly initialized
        assert!(fold_db.db_ops().get_node_id().is_ok());
        // Event statistics should be initialized (any non-negative value is valid)
        let _stats = fold_db.event_monitor().get_statistics();
    }

    #[test]
    fn test_schema_operations() {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        // Test schema loading
        assert!(fold_db.add_schema_available(test_schema.clone()).is_ok());
        assert!(fold_db.approve_schema(&test_schema.name).is_ok());
        
        // Test schema retrieval
        let retrieved_schema = fold_db.get_schema(&test_schema.name);
        assert!(retrieved_schema.is_ok());
        assert!(retrieved_schema.unwrap().is_some());
    }

    #[test]
    fn test_mutation_operations() {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        // Setup schema
        assert!(fold_db.add_schema_available(test_schema.clone()).is_ok());
        assert!(fold_db.approve_schema(&test_schema.name).is_ok());
        
        // Create mutation operations
        let mutation_ops = MutationOperations::new(
            fold_db.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            fold_db.message_bus(),
        );
        
        let test_mutation = TestUtils::create_test_mutation(&test_schema.name);
        
        // Test mutation validation
        assert!(mutation_ops.validate_mutation(&test_mutation).is_ok());
        assert!(mutation_ops.check_mutation_permissions(&test_mutation).is_ok());
    }

    #[test]
    fn test_query_operations() {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        // Setup schema
        assert!(fold_db.add_schema_available(test_schema.clone()).is_ok());
        assert!(fold_db.approve_schema(&test_schema.name).is_ok());
        
        // Create query operations
        let query_ops = QueryOperations::new(
            fold_db.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            fold_db.db_ops(),
        );
        
        let test_query = TestUtils::create_test_query(&test_schema.name);
        
        // Test query validation
        assert!(query_ops.validate_query(&test_query).is_ok());
        assert!(query_ops.check_query_permissions(&test_query).is_ok());
        
        // Test query optimization
        let optimized_query = query_ops.optimize_query(&test_query);
        assert!(optimized_query.is_ok());
    }

    #[test]
    fn test_encryption_operations() {
        let (fold_db, _temp_dir) = TestUtils::create_test_db();
        
        // Create encryption operations
        let mut encryption_ops = EncryptionOperations::new(fold_db.db_ops());
        
        // Test encryption state
        assert!(!encryption_ops.is_atom_encryption_enabled());
        
        // Test encryption stats
        let stats = encryption_ops.get_encryption_stats();
        assert!(stats.is_ok());
        let stats_map = stats.unwrap();
        assert_eq!(stats_map.get("encryption_enabled"), Some(&0));
    }

    #[test]
    fn test_range_schema_operations() {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let range_schema = TestUtils::create_test_range_schema();
        
        // Setup range schema
        assert!(fold_db.add_schema_available(range_schema.clone()).is_ok());
        assert!(fold_db.approve_schema(&range_schema.name).is_ok());
        
        // Create range mutation
        let mut fields_and_values = HashMap::new();
        fields_and_values.insert("id".to_string(), json!({"value": "test_id"}));
        fields_and_values.insert("value".to_string(), json!({"value": "test_value"}));
        
        let range_mutation = Mutation {
            schema_name: range_schema.name.clone(),
            fields_and_values,
            pub_key: "test_pub_key".to_string(),
            trust_distance: 0,
            mutation_type: crate::schema::types::MutationType::Create,
        };
        
        // Test range mutation validation
        let mutation_ops = MutationOperations::new(
            fold_db.schema_manager(),
            crate::permissions::PermissionWrapper::new(),
            fold_db.message_bus(),
        );
        
        let validation_result = mutation_ops.validate_mutation(&range_mutation);
        if let Err(e) = &validation_result {
            eprintln!("Range mutation validation failed: {:?}", e);
        }
        assert!(validation_result.is_ok());
    }

    #[test]
    fn test_event_statistics() {
        let (fold_db, _temp_dir) = TestUtils::create_test_db();
        
        // Test event monitor functionality
        let stats = fold_db.get_event_statistics();
        // Event statistics should be initialized (any non-negative value is valid)
        let _total = stats.total_events;
        
        // Test event summary logging (should not panic)
        fold_db.log_event_summary();
    }

    #[test]
    fn test_transform_operations() {
        let (fold_db, _temp_dir) = TestUtils::create_test_db();
        
        // Test transform manager access
        let transform_manager = fold_db.transform_manager();
        assert!(transform_manager.list_transforms().is_ok());
        
        // Test transform queue processing (should not panic)
        fold_db.process_transform_queue();
    }

    #[test]
    fn test_permissions_integration() {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        // Setup schema
        assert!(fold_db.add_schema_available(test_schema.clone()).is_ok());
        assert!(fold_db.approve_schema(&test_schema.name).is_ok());
        
        // Test node permissions
        let node_id = fold_db.get_node_id().expect("Failed to get node ID");
        let schemas = vec![test_schema.name.clone()];
        
        assert!(fold_db.set_schema_permissions(&node_id, &schemas).is_ok());
        let retrieved_schemas = fold_db.get_schema_permissions(&node_id);
        assert!(!retrieved_schemas.is_empty());
    }
}

/// Benchmark utilities for performance testing
#[cfg(test)]
pub mod benchmarks {
    use super::tests::TestUtils;
    use std::time::Instant;

    /// Benchmark FoldDB initialization time
    pub fn benchmark_initialization() -> std::time::Duration {
        let start = Instant::now();
        let (_fold_db, _temp_dir) = TestUtils::create_test_db();
        start.elapsed()
    }

    /// Benchmark schema operations
    pub fn benchmark_schema_operations() -> std::time::Duration {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        let start = Instant::now();
        let _ = fold_db.add_schema_available(test_schema.clone());
        let _ = fold_db.approve_schema(&test_schema.name);
        start.elapsed()
    }

    /// Benchmark mutation processing
    pub fn benchmark_mutation_processing() -> std::time::Duration {
        let (mut fold_db, _temp_dir) = TestUtils::create_test_db();
        let test_schema = TestUtils::create_test_schema();
        
        // Setup
        let _ = fold_db.add_schema_available(test_schema.clone());
        let _ = fold_db.approve_schema(&test_schema.name);
        
        let test_mutation = TestUtils::create_test_mutation(&test_schema.name);
        
        let start = Instant::now();
        let _ = fold_db.write_schema(test_mutation);
        start.elapsed()
    }
}