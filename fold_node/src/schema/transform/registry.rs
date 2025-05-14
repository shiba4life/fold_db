use crate::atom::{Atom, AtomRef};
use crate::schema::transform::executor::TransformExecutor;
use crate::schema::types::{SchemaError, Transform};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Registry for tracking dependencies between atom references and transforms.
/// 
/// The TransformRegistry maintains mappings between:
/// - Atom references (by UUID) and the transforms that depend on them
/// - Transforms and their dependent atom references
/// 
/// When an atom reference is updated, the registry can be used to find and
/// execute all transforms that depend on that reference.
/// Callback function type for getting an atom by its reference UUID
pub type GetAtomFn = Arc<dyn Fn(&str) -> Result<Atom, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for creating a new atom
pub type CreateAtomFn = Arc<dyn Fn(&str, String, Option<String>, JsonValue, Option<crate::atom::AtomStatus>) -> Result<Atom, Box<dyn std::error::Error>> + Send + Sync>;

/// Callback function type for updating an atom reference
pub type UpdateAtomRefFn = Arc<dyn Fn(&str, String, String) -> Result<AtomRef, Box<dyn std::error::Error>> + Send + Sync>;

/// Registry for tracking dependencies between atom references and transforms.
pub struct TransformRegistry {
    /// Maps atom reference UUIDs to the transforms that depend on them
    aref_to_transforms: RwLock<HashMap<String, HashSet<String>>>,
    
    /// Maps transform IDs to their dependent atom reference UUIDs
    transform_to_arefs: RwLock<HashMap<String, HashSet<String>>>,
    
    /// Maps transform IDs to the actual transform objects
    transforms: RwLock<HashMap<String, Transform>>,
    
    /// Maps transform IDs to their output atom reference UUIDs
    transform_outputs: RwLock<HashMap<String, String>>,
    
    /// Callback for getting an atom by its reference UUID
    get_atom_fn: GetAtomFn,
    
    /// Callback for creating a new atom
    create_atom_fn: CreateAtomFn,
    
    /// Callback for updating an atom reference
    update_atom_ref_fn: UpdateAtomRefFn,
}

impl TransformRegistry {
    /// Creates a new TransformRegistry with the specified callback functions.
    pub fn new(
        get_atom_fn: GetAtomFn,
        create_atom_fn: CreateAtomFn,
        update_atom_ref_fn: UpdateAtomRefFn,
    ) -> Self {
        Self {
            aref_to_transforms: RwLock::new(HashMap::new()),
            transform_to_arefs: RwLock::new(HashMap::new()),
            transforms: RwLock::new(HashMap::new()),
            transform_outputs: RwLock::new(HashMap::new()),
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
        }
    }
    
    /// Registers a transform with its input and output atom references.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - A unique identifier for the transform
    /// * `transform` - The transform to register
    /// * `input_arefs` - The atom reference UUIDs that the transform depends on
    /// * `output_aref` - The atom reference UUID where the transform result will be stored
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registration was successful, otherwise an error
    pub fn register_transform(
        &self,
        transform_id: String,
        mut transform: Transform,
        input_arefs: Vec<String>,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        // Validate the transform
        TransformExecutor::validate_transform(&transform)?;
        
        // Set the transform's input dependencies and output reference
        transform.set_input_dependencies(input_arefs.clone());
        transform.set_output_reference(output_aref.clone());
        
        // Register the transform
        {
            let mut transforms = self.transforms.write().unwrap();
            transforms.insert(transform_id.clone(), transform);
        }
        
        // Register the output atom reference
        {
            let mut transform_outputs = self.transform_outputs.write().unwrap();
            transform_outputs.insert(transform_id.clone(), output_aref);
        }
        
        // Register the input atom references
        {
            let mut transform_to_arefs = self.transform_to_arefs.write().unwrap();
            let mut aref_set = HashSet::new();
            
            for aref_uuid in &input_arefs {
                aref_set.insert(aref_uuid.clone());
            }
            
            transform_to_arefs.insert(transform_id.clone(), aref_set);
        }
        
        // Update the reverse mapping (aref -> transforms)
        {
            let mut aref_to_transforms = self.aref_to_transforms.write().unwrap();
            
            for aref_uuid in input_arefs {
                let transform_set = aref_to_transforms
                    .entry(aref_uuid)
                    .or_default();
                transform_set.insert(transform_id.clone());
            }
        }
        
        Ok(())
    }
    
    /// Registers a transform with automatic input dependency detection.
    ///
    /// This method analyzes the transform logic to detect input dependencies,
    /// and registers the transform with those dependencies.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - A unique identifier for the transform
    /// * `transform` - The transform to register
    /// * `output_aref` - The atom reference UUID where the transform result will be stored
    ///
    /// # Returns
    ///
    /// `Ok(())` if the registration was successful, otherwise an error
    pub fn register_transform_auto(
        &self,
        transform_id: String,
        transform: Transform,
        output_aref: String,
    ) -> Result<(), SchemaError> {
        // Analyze the transform to detect input dependencies
        let dependencies = transform.analyze_dependencies();
        
        // Register the transform with the detected dependencies
        self.register_transform(
            transform_id,
            transform,
            dependencies.into_iter().collect(),
            output_aref,
        )
    }
    
    /// Unregisters a transform.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - The ID of the transform to unregister
    ///
    /// # Returns
    ///
    /// `true` if the transform was found and unregistered, `false` otherwise
    pub fn unregister_transform(&self, transform_id: &str) -> bool {
        // Remove from transforms map
        let found = {
            let mut transforms = self.transforms.write().unwrap();
            transforms.remove(transform_id).is_some()
        };
        
        if found {
            // Remove from transform outputs
            {
                let mut transform_outputs = self.transform_outputs.write().unwrap();
                transform_outputs.remove(transform_id);
            }
            
            // Get the input arefs for this transform
            let input_arefs = {
                let mut transform_to_arefs = self.transform_to_arefs.write().unwrap();
                transform_to_arefs.remove(transform_id).unwrap_or_default()
            };
            
            // Update the reverse mapping (aref -> transforms)
            {
                let mut aref_to_transforms = self.aref_to_transforms.write().unwrap();
                
                for aref_uuid in input_arefs {
                    if let Some(transform_set) = aref_to_transforms.get_mut(&aref_uuid) {
                        transform_set.remove(transform_id);
                        
                        // Remove the entry if the set is empty
                        if transform_set.is_empty() {
                            aref_to_transforms.remove(&aref_uuid);
                        }
                    }
                }
            }
        }
        
        found
    }
    
    /// Handles an atom reference update by executing all dependent transforms.
    ///
    /// # Arguments
    ///
    /// * `aref_uuid` - The UUID of the updated atom reference
    ///
    /// # Returns
    ///
    /// A vector of results from executing the transforms
    pub fn handle_atom_ref_update(&self, aref_uuid: &str) -> Vec<Result<JsonValue, SchemaError>> {
        let mut results = Vec::new();
        
        // Find all transforms that depend on this atom reference
        let transform_ids = {
            let aref_to_transforms = self.aref_to_transforms.read().unwrap();
            
            match aref_to_transforms.get(aref_uuid) {
                Some(transform_set) => transform_set.clone(),
                None => return results, // No dependent transforms
            }
        };
        
        // Execute each transform
        for transform_id in transform_ids {
            let result = self.execute_transform(&transform_id);
            results.push(result);
        }
        
        results
    }
    
    /// Executes a transform and updates its output atom reference.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - The ID of the transform to execute
    ///
    /// # Returns
    ///
    /// The result of executing the transform
    fn execute_transform(&self, transform_id: &str) -> Result<JsonValue, SchemaError> {
        // Get the transform
        let transform = {
            let transforms = self.transforms.read().unwrap();
            
            match transforms.get(transform_id) {
                Some(transform) => transform.clone(),
                None => return Err(SchemaError::InvalidField(format!("Transform not found: {}", transform_id))),
            }
        };
        
        // Create an input provider function that gets values from atom references
        let get_atom_fn = &self.get_atom_fn;
        let transform_to_arefs = self.transform_to_arefs.read().unwrap();
        let input_arefs = transform_to_arefs.get(transform_id).cloned().unwrap_or_default();
        
        let input_provider = move |input_name: &str| -> Result<JsonValue, Box<dyn std::error::Error>> {
            // Check if this input name is a registered atom reference
            if input_arefs.contains(input_name) {
                // Get the atom from the atom reference
                let atom = (get_atom_fn)(input_name)?;
                
                // Return the atom content
                Ok(atom.content().clone())
            } else {
                // If not found, return an error
                Err(format!("Input not found: {}", input_name).into())
            }
        };
        
        // Execute the transform with the input provider
        let result = TransformExecutor::execute_transform_with_provider(&transform, input_provider)?;
        
        // Update the output atom reference
        let output_aref = {
            let transform_outputs = self.transform_outputs.read().unwrap();
            
            match transform_outputs.get(transform_id) {
                Some(aref_uuid) => aref_uuid.clone(),
                None => return Err(SchemaError::InvalidField(format!("Transform output not found: {}", transform_id))),
            }
        };
        
        // Create a new atom with the transform result
        let atom = match (self.create_atom_fn)(
            "transform_result",
            "transform_system".to_string(),
            None,
            result.clone(),
            None,
        ) {
            Ok(atom) => atom,
            Err(e) => return Err(SchemaError::InvalidField(format!("Failed to create atom: {}", e))),
        };
        
        // Update the output atom reference
        match (self.update_atom_ref_fn)(
            &output_aref,
            atom.uuid().to_string(),
            "transform_system".to_string(),
        ) {
            Ok(_) => {},
            Err(e) => return Err(SchemaError::InvalidField(format!("Failed to update atom reference: {}", e))),
        }
        
        Ok(result)
    }
    
    /// Gets all transforms that depend on the specified atom reference.
    ///
    /// # Arguments
    ///
    /// * `aref_uuid` - The UUID of the atom reference
    ///
    /// # Returns
    ///
    /// A set of transform IDs that depend on the atom reference
    pub fn get_dependent_transforms(&self, aref_uuid: &str) -> HashSet<String> {
        let aref_to_transforms = self.aref_to_transforms.read().unwrap();
        
        match aref_to_transforms.get(aref_uuid) {
            Some(transform_set) => transform_set.clone(),
            None => HashSet::new(),
        }
    }
    
    /// Gets all atom references that a transform depends on.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - The ID of the transform
    ///
    /// # Returns
    ///
    /// A set of atom reference UUIDs that the transform depends on
    pub fn get_transform_inputs(&self, transform_id: &str) -> HashSet<String> {
        let transform_to_arefs = self.transform_to_arefs.read().unwrap();
        
        match transform_to_arefs.get(transform_id) {
            Some(aref_set) => aref_set.clone(),
            None => HashSet::new(),
        }
    }
    
    /// Gets the output atom reference for a transform.
    ///
    /// # Arguments
    ///
    /// * `transform_id` - The ID of the transform
    ///
    /// # Returns
    ///
    /// The UUID of the output atom reference, if found
    pub fn get_transform_output(&self, transform_id: &str) -> Option<String> {
        let transform_outputs = self.transform_outputs.read().unwrap();
        transform_outputs.get(transform_id).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_operations::DbOperations;
    use crate::fold_db_core::atom_manager::AtomManager;
    use tempfile::tempdir;
    
    fn setup_test_env() -> (Arc<AtomManager>, Arc<TransformRegistry>) {
        // Create a temporary directory for the test database
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test_db");
        
        // Create a database operations instance
        let db = sled::open(db_path.to_str().unwrap()).unwrap();
        let db_ops = DbOperations::new(db);
        
        // Create an atom manager
        let atom_manager = Arc::new(AtomManager::new(db_ops));
        
        // Create callback functions
        let am_clone = Arc::clone(&atom_manager);
        let get_atom_fn: GetAtomFn = Arc::new(move |aref_uuid| {
            am_clone.get_latest_atom(aref_uuid)
        });
        
        let am_clone = Arc::clone(&atom_manager);
        let create_atom_fn: CreateAtomFn = Arc::new(move |schema_name, source_pub_key, prev_atom_uuid, content, status| {
            am_clone.create_atom(schema_name, source_pub_key, prev_atom_uuid, content, status)
        });
        
        let am_clone = Arc::clone(&atom_manager);
        let update_atom_ref_fn: UpdateAtomRefFn = Arc::new(move |aref_uuid, atom_uuid, source_pub_key| {
            am_clone.update_atom_ref(aref_uuid, atom_uuid, source_pub_key)
        });
        
        // Create a transform registry
        let registry = Arc::new(TransformRegistry::new(
            get_atom_fn,
            create_atom_fn,
            update_atom_ref_fn,
        ));
        
        (atom_manager, registry)
    }
    
    #[test]
    fn test_register_and_unregister_transform() {
        let (_, registry) = setup_test_env();
        
        // Create a transform
        let transform = Transform::new(
            "input1 + input2".to_string(),
            false,
            None,
            false,
        );
        
        // Register the transform
        let result = registry.register_transform(
            "test_transform".to_string(),
            transform,
            vec!["input1".to_string(), "input2".to_string()],
            "output".to_string(),
        );
        
        assert!(result.is_ok());
        
        // Check that the transform was registered
        let dependent_transforms = registry.get_dependent_transforms("input1");
        assert!(dependent_transforms.contains("test_transform"));
        
        let transform_inputs = registry.get_transform_inputs("test_transform");
        assert!(transform_inputs.contains("input1"));
        assert!(transform_inputs.contains("input2"));
        
        let transform_output = registry.get_transform_output("test_transform");
        assert_eq!(transform_output, Some("output".to_string()));
        
        // Unregister the transform
        let result = registry.unregister_transform("test_transform");
        assert!(result);
        
        // Check that the transform was unregistered
        let dependent_transforms = registry.get_dependent_transforms("input1");
        assert!(!dependent_transforms.contains("test_transform"));
        
        let transform_inputs = registry.get_transform_inputs("test_transform");
        assert!(transform_inputs.is_empty());
        
        let transform_output = registry.get_transform_output("test_transform");
        assert_eq!(transform_output, None);
    }
    
    #[test]
    fn test_handle_atom_ref_update() {
        let (atom_manager, registry) = setup_test_env();
        
        // Create a transform with a pre-parsed expression
        use crate::schema::transform::ast::{Expression, Operator, Value};
        
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Variable("input1".to_string())),
            operator: Operator::Add,
            right: Box::new(Expression::Variable("input2".to_string())),
        };
        
        let transform = Transform::new_with_expr(
            "input1 + input2".to_string(),
            expr,
            false,
            None,
            false,
        );
        
        // Create input atoms
        let atom1 = atom_manager
            .create_atom(
                "test_schema",
                "test_key".to_string(),
                None,
                serde_json::json!(5),
                None,
            )
            .unwrap();
            
        let atom2 = atom_manager
            .create_atom(
                "test_schema",
                "test_key".to_string(),
                None,
                serde_json::json!(10),
                None,
            )
            .unwrap();
            
        // Create atom references for inputs
        let _ = atom_manager
            .update_atom_ref(
                "input1",
                atom1.uuid().to_string(),
                "test_key".to_string(),
            )
            .unwrap();
            
        let _ = atom_manager
            .update_atom_ref(
                "input2",
                atom2.uuid().to_string(),
                "test_key".to_string(),
            )
            .unwrap();
        
        // Create an output atom reference
        let _ = atom_manager.update_atom_ref(
            "output",
            "dummy".to_string(),
            "test_key".to_string(),
        ).unwrap();
        
        // Register the transform
        let result = registry.register_transform(
            "test_transform".to_string(),
            transform,
            vec!["input1".to_string(), "input2".to_string()],
            "output".to_string(),
        );
        
        assert!(result.is_ok());
        
        // Handle an atom reference update
        let results = registry.handle_atom_ref_update("input1");
        assert_eq!(results.len(), 1);
        
        // Check the result
        let result = &results[0];
        assert!(result.is_ok());
        // Compare the numeric values, not the exact JSON representation
        match result.as_ref().unwrap() {
            JsonValue::Number(n) => {
                let value = n.as_f64().unwrap();
                assert!((value - 15.0).abs() < 0.001, "Expected 15, got {}", value);
            },
            _ => panic!("Expected number, got {:?}", result.as_ref().unwrap()),
        }
        
        // Check that the output atom reference was updated
        let output_atom = atom_manager.get_latest_atom("output").unwrap();
        // Compare the numeric values, not the exact JSON representation
        match output_atom.content() {
            JsonValue::Number(n) => {
                let value = n.as_f64().unwrap();
                assert!((value - 15.0).abs() < 0.001, "Expected 15, got {}", value);
            },
            _ => panic!("Expected number, got {:?}", output_atom.content()),
        }
    }
}