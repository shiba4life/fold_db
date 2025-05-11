#[cfg(test)]
mod tests {
    use crate::schema::transform::{TransformRegistry, GetAtomFn, CreateAtomFn, UpdateAtomRefFn};
    use crate::schema::types::Transform;
    use crate::atom::{Atom, AtomRef};
    use serde_json::json;
    use std::sync::Arc;

    #[test]
    fn test_transform_registry_basic() {
        // Create mock callback functions
        let get_atom_fn: GetAtomFn = Arc::new(|_| {
            Err("Not implemented".into())
        });
        
        let create_atom_fn: CreateAtomFn = Arc::new(|_, _, _, _, _| {
            Err("Not implemented".into())
        });
        
        let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(|_, _, _| {
            Err("Not implemented".into())
        });
        
        // Create a transform registry
        let registry = TransformRegistry::new(
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
        );
        
        // Create a transform
        let transform = Transform::new(
            "a + b".to_string(),
            false,
            None,
            false,
        );
        
        // Register the transform
        let result = registry.register_transform(
            "test_transform".to_string(),
            transform,
            vec!["a".to_string(), "b".to_string()],
            "c".to_string(),
        );
        
        assert!(result.is_ok(), "Failed to register transform");
        
        // Check that the transform was registered
        let dependent_transforms = registry.get_dependent_transforms("a");
        assert!(dependent_transforms.contains("test_transform"), "Transform not registered correctly");
        
        let transform_inputs = registry.get_transform_inputs("test_transform");
        assert!(transform_inputs.contains("a"), "Transform input not registered correctly");
        assert!(transform_inputs.contains("b"), "Transform input not registered correctly");
        
        let transform_output = registry.get_transform_output("test_transform");
        assert_eq!(transform_output, Some("c".to_string()), "Transform output not registered correctly");
        
        // Unregister the transform
        let result = registry.unregister_transform("test_transform");
        assert!(result, "Failed to unregister transform");
        
        // Check that the transform was unregistered
        let dependent_transforms = registry.get_dependent_transforms("a");
        assert!(!dependent_transforms.contains("test_transform"), "Transform not unregistered correctly");
    }
}