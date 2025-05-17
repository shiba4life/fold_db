use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use crate::transform::ast::TransformDeclaration;

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
    /// The name of the transform
    #[serde(default)]
    pub name: String,
    
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
    
    /// The parsed expression (not serialized)
    #[serde(skip)]
    pub parsed_expr: Option<crate::transform::ast::Expression>,
    
    /// The parsed transform declaration (not serialized)
    #[serde(skip)]
    pub parsed_declaration: Option<TransformDeclaration>,
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
            name: String::new(),
            logic,
            reversible,
            signature,
            payment_required,
            input_dependencies: Vec::new(),
            output_reference: None,
            parsed_expr: None,
            parsed_declaration: None,
        }
    }
    
    /// Creates a new Transform with a pre-parsed expression.
    ///
    /// # Arguments
    ///
    /// * `logic` - The transform logic expressed in the DSL
    /// * `parsed_expr` - The pre-parsed expression
    /// * `reversible` - Whether this transform is reversible
    /// * `signature` - Optional signature for verification
    /// * `payment_required` - Whether payment is required for this transform
    ///
    /// # Returns
    ///
    /// A new Transform instance
    #[must_use]
    pub fn new_with_expr(
        logic: String,
        parsed_expr: crate::transform::ast::Expression,
        reversible: bool,
        signature: Option<String>,
        payment_required: bool,
    ) -> Self {
        Self {
            name: String::new(),
            logic,
            reversible,
            signature,
            payment_required,
            input_dependencies: Vec::new(),
            output_reference: None,
            parsed_expr: Some(parsed_expr),
            parsed_declaration: None,
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
            name: String::new(),
            logic,
            reversible,
            signature,
            payment_required,
            input_dependencies,
            output_reference,
            parsed_expr: None,
            parsed_declaration: None,
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
        let mut dependencies = HashSet::new();
        
        // Split by dots to handle schema.field format
        for part in self.logic.split(|c: char| !c.is_alphanumeric() && c != '.') {
            if part.is_empty() || part.chars().next().unwrap().is_numeric() {
                continue;
            }
            
            // Skip keywords and operators
            match part {
                "let" | "if" | "else" | "return" | "true" | "false" | "null" => continue,
                _ => {}
            }
            
            // If it contains a dot, it's a schema.field reference
            if part.contains('.') {
                let parts: Vec<&str> = part.split('.').collect();
                if parts.len() == 2 {
                    // Add just the field name, not the schema prefix
                    dependencies.insert(parts[1].to_string());
                }
            } else {
                // For backward compatibility, add the whole part if it's not a schema.field
                dependencies.insert(part.to_string());
            }
        }
        
        dependencies
    }
    /// Creates a new Transform from a TransformDeclaration.
    ///
    /// # Arguments
    ///
    /// * `declaration` - The transform declaration
    ///
    /// # Returns
    ///
    /// A new Transform instance
    #[must_use]
    pub fn from_declaration(declaration: TransformDeclaration) -> Self {
        // Extract logic from the declaration
        let logic = declaration.logic.iter()
            .map(|expr| format!("{}", expr))
            .collect::<Vec<_>>()
            .join("\n");
        
        // Set payment requirement to false since it's been removed
        let payment_required = false;
        
        Self {
            name: declaration.name.clone(),
            logic,
            reversible: declaration.reversible,
            signature: declaration.signature.clone(),
            payment_required,
            input_dependencies: Vec::new(), // Will be populated later
            output_reference: None,
            parsed_expr: None, // Will be populated later
            parsed_declaration: Some(declaration),
        }
    }
    
    /// Gets the name of the transform.
    ///
    /// # Returns
    ///
    /// The name of the transform
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::transform::ast::{Expression, Operator, TransformDeclaration};

    #[test]
    fn test_transform_from_declaration() {
        let declaration = TransformDeclaration {
            name: "test_transform".to_string(),
            logic: vec![
                Expression::Return(Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Variable("field1".to_string())),
                    operator: Operator::Add,
                    right: Box::new(Expression::Variable("field2".to_string())),
                })),
            ],
            reversible: false,
            signature: None,
        };

        let transform = Transform::from_declaration(declaration);

        assert_eq!(transform.name, "test_transform");
        assert_eq!(transform.logic, "return (field1 + field2)"); // Removed trailing semicolon
        assert_eq!(transform.reversible, false);
        assert!(transform.signature.is_none());
        assert_eq!(transform.payment_required, false); // payment_required is hardcoded to false in from_declaration
        assert!(transform.input_dependencies.is_empty()); // input_dependencies is not populated in from_declaration
        assert!(transform.output_reference.is_none());
        assert!(transform.parsed_expr.is_none()); // parsed_expr is not populated in from_declaration
        assert!(transform.parsed_declaration.is_some());
    }
}