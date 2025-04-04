/// Type alias for the schema check callback function
pub type SchemaCheckCallback = Box<dyn Fn(&[String]) -> Vec<String> + Send + Sync>;

/// Service for handling schema operations
pub struct SchemaService {
    /// Callback function for checking schema availability
    schema_check_callback: SchemaCheckCallback,
}

impl Clone for SchemaService {
    fn clone(&self) -> Self {
        // Since we can't clone the function pointer directly,
        // we create a new service with the default callback
        // This is only used for testing and initialization
        Self::new()
    }
}

impl Default for SchemaService {
    fn default() -> Self {
        Self {
            // Default callback returns empty list (no schemas available)
            schema_check_callback: Box::new(|_| Vec::new()),
        }
    }
}

impl SchemaService {
    /// Create a new schema service
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the callback function for checking schema availability
    ///
    /// The callback takes a slice of schema names and returns a vector
    /// containing the subset of those names that are available on this node.
    pub fn set_schema_check_callback<F>(&mut self, callback: F)
    where
        F: Fn(&[String]) -> Vec<String> + Send + Sync + 'static,
    {
        self.schema_check_callback = Box::new(callback);
    }

    /// Check which schemas from the provided list are available on this node
    ///
    /// Returns a subset of the input schema names that are available.
    pub fn check_schemas(&self, schema_names: &[String]) -> Vec<String> {
        (self.schema_check_callback)(schema_names)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_service() {
        let mut service = SchemaService::new();
        
        // Default callback should return empty list
        let result = service.check_schemas(&["schema1".to_string(), "schema2".to_string()]);
        assert!(result.is_empty());
        
        // Set custom callback
        service.set_schema_check_callback(|names| {
            names.iter()
                .filter(|name| name.contains("1"))
                .cloned()
                .collect()
        });
        
        // Should now return only schemas containing "1"
        let result = service.check_schemas(&[
            "schema1".to_string(),
            "schema2".to_string(),
            "test1".to_string()
        ]);
        
        assert_eq!(result, vec!["schema1".to_string(), "test1".to_string()]);
    }
}
