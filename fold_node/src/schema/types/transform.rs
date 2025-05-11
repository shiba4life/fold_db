use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Represents a transformation that can be applied to field values.
///
/// Transforms define how data from source fields is processed to produce
/// a derived value. They are expressed in a domain-specific language (DSL)
/// that supports basic arithmetic, comparisons, conditionals, and a small
/// set of built-in functions.
///
/// # Features
///
/// * Declarative syntax for expressing transformations
/// * Support for basic arithmetic, comparisons, and conditionals
/// * Optional signature for verification and auditability
/// * Payment requirements for accessing transformed data
/// * Automatic input dependency tracking
///
/// # Example
///
/// ```
/// use fold_node::schema::types::Transform;
///
/// let transform = Transform::new(
///     "let bmi = weight / (height ^ 2); return 0.5 * blood_pressure + 1.2 * bmi;".to_string(),
///     false,
///     Some("sha256sum(\"v1.0.3\")".to_string()),
///     true,
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform {
    /// The transform logic expressed in the DSL
    pub logic: String,
    
    /// Whether this transform is reversible
    pub reversible: bool,
    
    /// Optional signature for verification
    pub signature: Option<String>,
    
    /// Whether payment is required for this transform
    pub payment_required: bool,
    
    /// Input dependencies for this transform
    #[serde(default)]
    pub input_dependencies: Vec<String>,
    
    /// Output reference for this transform
    #[serde(skip)]
    pub output_reference: Option<String>,
}

impl Transform {
    /// Creates a new Transform with the specified parameters.
    ///
    /// # Arguments
    ///
    /// * `logic` - The transform logic expressed in the DSL
    /// * `reversible` - Whether this transform is reversible
    /// * `signature` - Optional signature for verification
    /// * `payment_required` - Whether payment is required for this transform
    ///
    /// # Returns
    ///
    /// A new Transform instance
    #[must_use]
    pub fn new(
        logic: String,
        reversible: bool,
        signature: Option<String>,
        payment_required: bool,
    ) -> Self {
        Self {
            logic,
            reversible,
            signature,
            payment_required,
            input_dependencies: Vec::new(),
            output_reference: None,
        }
    }
    
    /// Creates a new Transform with input dependencies and output reference.
    ///
    /// # Arguments
    ///
    /// * `logic` - The transform logic expressed in the DSL
    /// * `reversible` - Whether this transform is reversible
    /// * `signature` - Optional signature for verification
    /// * `payment_required` - Whether payment is required for this transform
    /// * `input_dependencies` - The input dependencies for this transform
    /// * `output_reference` - The output reference for this transform
    ///
    /// # Returns
    ///
    /// A new Transform instance
    #[must_use]
    pub fn with_dependencies(
        logic: String,
        reversible: bool,
        signature: Option<String>,
        payment_required: bool,
        input_dependencies: Vec<String>,
        output_reference: Option<String>,
    ) -> Self {
        Self {
            logic,
            reversible,
            signature,
            payment_required,
            input_dependencies,
            output_reference,
        }
    }
    
    /// Sets the input dependencies for this transform.
    ///
    /// # Arguments
    ///
    /// * `dependencies` - The input dependencies for this transform
    pub fn set_input_dependencies(&mut self, dependencies: Vec<String>) {
        self.input_dependencies = dependencies;
    }
    
    /// Sets the output reference for this transform.
    ///
    /// # Arguments
    ///
    /// * `output_reference` - The output reference for this transform
    pub fn set_output_reference(&mut self, output_reference: String) {
        self.output_reference = Some(output_reference);
    }
    
    /// Gets the input dependencies for this transform.
    ///
    /// # Returns
    ///
    /// The input dependencies for this transform
    pub fn get_input_dependencies(&self) -> &[String] {
        &self.input_dependencies
    }
    
    /// Gets the output reference for this transform.
    ///
    /// # Returns
    ///
    /// The output reference for this transform, if any
    pub fn get_output_reference(&self) -> Option<&String> {
        self.output_reference.as_ref()
    }
    
    /// Analyzes the transform logic to extract variable names that might be input dependencies.
    ///
    /// This is a simple implementation that just looks for identifiers in the logic.
    /// A more sophisticated implementation would parse the logic and extract actual variable references.
    ///
    /// # Returns
    ///
    /// A set of potential input dependencies
    pub fn analyze_dependencies(&self) -> HashSet<String> {
        // This is a very simple implementation that just looks for words in the logic
        // A real implementation would parse the logic and extract actual variable references
        let mut dependencies = HashSet::new();
        
        // Split the logic by non-alphanumeric characters
        for word in self.logic.split(|c: char| !c.is_alphanumeric()) {
            if !word.is_empty() && !word.chars().next().unwrap().is_numeric() {
                // Skip keywords and operators
                match word {
                    "let" | "if" | "else" | "return" | "true" | "false" | "null" => continue,
                    _ => {}
                }
                
                dependencies.insert(word.to_string());
            }
        }
        
        dependencies
    }
}