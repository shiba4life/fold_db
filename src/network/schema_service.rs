/// Type alias for the schema check callback function
pub type SchemaCheckCallback = Box<dyn Fn(&[String]) -> Vec<String> + Send + Sync>;

/// Service for handling schema operations over the network.
///
/// SchemaService provides functionality for checking schema availability and
/// handling schema-related requests from other nodes. It uses a callback-based
/// approach to integrate with the local node's schema system.
///
/// # Features
///
/// * Schema availability checking
/// * Customizable callback for schema validation
/// * Integration with the local node's schema system
///
/// # Examples
///
/// ```rust
/// use datafold::network::schema_service::SchemaService;
///
/// fn is_schema_available(name: &str) -> bool {
///     // This would normally check against your schema storage
///     name == "user_profile"
/// }
///
/// let mut service = SchemaService::new();
///
/// // Set a custom callback for schema checking
/// service.set_schema_check_callback(|schema_names| {
///     // Return the subset of schema_names that are available
///     schema_names.iter()
///         .filter(|name| is_schema_available(name))
///         .cloned()
///         .collect()
/// });
///
/// // Check which schemas are available
/// let available = service.check_schemas(&["user_profile".to_string(), "posts".to_string()]);
/// ```
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

    /// Set the callback function for checking schema availability.
    ///
    /// This function allows setting a custom callback that determines which schemas
    /// are available on this node. The callback is used by the `check_schemas` method
    /// to respond to schema availability requests from other nodes.
    ///
    /// # Arguments
    ///
    /// * `callback` - A function that takes a slice of schema names and returns a vector
    ///   containing the subset of those names that are available on this node.
    ///   The function must be `Send` and `Sync` to be safely shared across threads.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use datafold::network::schema_service::SchemaService;
    ///
    /// struct MockSchemaManager {
    ///     available_schemas: Vec<String>
    /// }
    ///
    /// impl MockSchemaManager {
    ///     fn schema_exists(&self, name: &str) -> Option<bool> {
    ///         Some(self.available_schemas.contains(&name.to_string()))
    ///     }
    /// }
    ///
    /// let schema_manager = MockSchemaManager {
    ///     available_schemas: vec!["user_profile".to_string()]
    /// };
    ///
    /// let mut service = SchemaService::new();
    ///
    /// // Set a custom callback that checks against a local schema manager
    /// service.set_schema_check_callback(move |schema_names| {
    ///     schema_names
    ///         .iter()
    ///         .filter(|name| schema_manager.schema_exists(name).unwrap_or(false))
    ///         .cloned()
    ///         .collect()
    /// });
    /// ```
    pub fn set_schema_check_callback<F>(&mut self, callback: F)
    where
        F: Fn(&[String]) -> Vec<String> + Send + Sync + 'static,
    {
        self.schema_check_callback = Box::new(callback);
    }

    /// Check which schemas from the provided list are available on this node.
    ///
    /// This function uses the registered callback to determine which schemas
    /// from the provided list are available on this node. It's used by the
    /// network layer to respond to schema availability requests from other nodes.
    ///
    /// # Arguments
    ///
    /// * `schema_names` - A slice of schema names to check for availability
    ///
    /// # Returns
    ///
    /// A vector containing the subset of schema names that are available on this node.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use datafold::network::schema_service::SchemaService;
    ///
    /// let mut service = SchemaService::new();
    /// let schemas_to_check = vec!["user_profile".to_string(), "posts".to_string()];
    /// let available_schemas = service.check_schemas(&schemas_to_check);
    /// println!("Available schemas: {:?}", available_schemas);
    /// ```
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
            names
                .iter()
                .filter(|name| name.contains("1"))
                .cloned()
                .collect()
        });

        // Should now return only schemas containing "1"
        let result = service.check_schemas(&[
            "schema1".to_string(),
            "schema2".to_string(),
            "test1".to_string(),
        ]);

        assert_eq!(result, vec!["schema1".to_string(), "test1".to_string()]);
    }
}
