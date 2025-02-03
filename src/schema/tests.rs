#[cfg(test)]
mod tests {
    use super::super::types::{Count, ExplicitCounts, PolicyLevel, PermissionsPolicy, Operation};
    use super::super::internal_schema::InternalSchema;
    use super::super::security::SecurityManager;
    use super::super::manager::SchemaManager;
    use std::collections::HashMap;

    #[test]
    fn test_schema_load_and_get() {
        let manager = SchemaManager::new();
        let mut schema = InternalSchema::new();
        schema.fields.insert("field1".to_string(), "aref-uuid-1".to_string());
        schema.fields.insert("field2".to_string(), "aref-uuid-2".to_string());
        
        assert!(manager.load_schema("test", schema.clone()).is_ok());
        let retrieved = manager.get_schema("test").unwrap().unwrap();
        assert_eq!(
            retrieved.fields.get("field1"),
            Some(&"aref-uuid-1".to_string())
        );
    }

    #[test]
    fn test_schema_unload() {
        let manager = SchemaManager::new();
        let schema = InternalSchema::new();
        
        assert!(manager.load_schema("test", schema).is_ok());
        assert!(manager.is_loaded("test").unwrap());
        assert!(manager.unload_schema("test").unwrap());
        assert!(!manager.is_loaded("test").unwrap());
    }

    #[test]
    fn test_nonexistent_schema() {
        let manager = SchemaManager::new();
        assert!(manager.get_schema("nonexistent").unwrap().is_none());
        assert!(!manager.is_loaded("nonexistent").unwrap());
    }

    #[test]
    fn test_schema_update() {
        let manager = SchemaManager::new();
        let mut schema1 = InternalSchema::new();
        schema1.fields.insert("field1".to_string(), "uuid1".to_string());
        
        let mut schema2 = InternalSchema::new();
        schema2.fields.insert("field1".to_string(), "uuid2".to_string());
        
        assert!(manager.load_schema("test", schema1).is_ok());
        assert!(manager.load_schema("test", schema2).is_ok());
        
        let retrieved = manager.get_schema("test").unwrap().unwrap();
        assert_eq!(
            retrieved.fields.get("field1"),
            Some(&"uuid2".to_string())
        );
    }

    #[test]
    fn test_explicit_once_permission() {
        let mut schema = InternalSchema::new();
        schema.fields.insert("field1".to_string(), "aref-uuid-1".to_string());
        
        // Set up policy requiring explicit permission once
        let mut policies = HashMap::new();
        policies.insert("field1".to_string(), PermissionsPolicy {
            read_policy: PolicyLevel::ExplicitOnce,
            write_policy: PolicyLevel::ExplicitOnce,
        });
        schema.policies = Some(policies);

        // Grant one-time permission
        schema.set_explicit_permissions("pubkey1".to_string(), ExplicitCounts {
            r: Count::Limited(1),
            w: Count::Limited(1),
        });

        // First read should succeed
        assert!(SecurityManager::check_permission(
            &mut schema,
            "field1",
            Operation::Read,
            0,
            true,
            "pubkey1"
        ));

        // Second read should fail
        assert!(!SecurityManager::check_permission(
            &mut schema,
            "field1",
            Operation::Read,
            0,
            true,
            "pubkey1"
        ));
    }

    #[test]
    fn test_explicit_many_permission() {
        let mut schema = InternalSchema::new();
        schema.fields.insert("field1".to_string(), "aref-uuid-1".to_string());
        
        // Set up policy requiring explicit permission with multiple uses
        let mut policies = HashMap::new();
        policies.insert("field1".to_string(), PermissionsPolicy {
            read_policy: PolicyLevel::ExplicitMany,
            write_policy: PolicyLevel::ExplicitMany,
        });
        schema.policies = Some(policies);

        // Grant multiple permissions
        schema.set_explicit_permissions("pubkey2".to_string(), ExplicitCounts {
            r: Count::Limited(3),
            w: Count::Limited(2),
        });

        // First three reads should succeed
        for _ in 0..3 {
            assert!(SecurityManager::check_permission(
                &mut schema,
                "field1",
                Operation::Read,
                0,
                true,
                "pubkey2"
            ));
        }

        // Fourth read should fail
        assert!(!SecurityManager::check_permission(
            &mut schema,
            "field1",
            Operation::Read,
            0,
            true,
            "pubkey2"
        ));
    }

    #[test]
    fn test_unlimited_permission() {
        let mut schema = InternalSchema::new();
        schema.fields.insert("field1".to_string(), "aref-uuid-1".to_string());
        
        // Set up policy requiring explicit permission
        let mut policies = HashMap::new();
        policies.insert("field1".to_string(), PermissionsPolicy {
            read_policy: PolicyLevel::ExplicitMany,
            write_policy: PolicyLevel::ExplicitMany,
        });
        schema.policies = Some(policies);

        // Grant unlimited permissions
        schema.set_explicit_permissions("pubkey3".to_string(), ExplicitCounts {
            r: Count::Unlimited,
            w: Count::Unlimited,
        });

        // Multiple reads should all succeed
        for _ in 0..10 {
            assert!(SecurityManager::check_permission(
                &mut schema,
                "field1",
                Operation::Read,
                0,
                true,
                "pubkey3"
            ));
        }
    }
}
