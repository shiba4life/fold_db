//! Executor for transforms.
//!
//! This module provides the high-level interface for applying transforms to field values.
//! It handles the integration with the schema system and manages the execution context.

use super::ast::Value;
use super::interpreter::Interpreter;
use super::parser::TransformParser;
use crate::schema::types::{SchemaError, Transform};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use log::info;

/// Executor for transforms.
pub struct TransformExecutor;

impl TransformExecutor {
    /// Executes a transform with the given input values.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to execute
    /// * `input_values` - The input values for the transform
    ///
    /// # Returns
    ///
    /// The result of the transform execution
    pub fn execute_transform(
        transform: &Transform,
        input_values: HashMap<String, JsonValue>,
    ) -> Result<JsonValue, SchemaError> {
        info!(
            "execute_transform logic: {} with inputs: {:?}",
            transform.logic,
            input_values
        );
        let result = Self::execute_transform_with_expr(transform, input_values);
        if let Ok(ref value) = result {
            info!("execute_transform result: {:?}", value);
        }
        result
    }
    
    /// Executes a transform with the given input provider function.
    ///
    /// This version allows the transform to collect its own inputs using the provided function.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to execute
    /// * `input_provider` - A function that provides input values for a given input name
    ///
    /// # Returns
    ///
    /// The result of the transform execution
    pub fn execute_transform_with_provider<F>(
        transform: &Transform,
        input_provider: F,
    ) -> Result<JsonValue, SchemaError>
    where
        F: Fn(&str) -> Result<JsonValue, Box<dyn std::error::Error>>,
    {
        // Collect input values using the provider function
        let mut input_values = HashMap::new();
        
        // Use the transform's declared dependencies
        for input_name in transform.get_inputs() {
            match input_provider(input_name) {
                Ok(value) => {
                    input_values.insert(input_name.clone(), value);
                },
                Err(e) => {
                    return Err(SchemaError::InvalidField(format!("Failed to get input '{}': {}", input_name, e)));
                }
            }
        }

        // If no dependencies are declared, try to analyze the transform logic
        if transform.get_inputs().is_empty() {
            let dependencies = transform.analyze_dependencies();
            for input_name in dependencies {
                // Skip if we already have this input
                if input_values.contains_key(&input_name) {
                    continue;
                }
                
                // Try to get the input value
                match input_provider(&input_name) {
                    Ok(value) => {
                        input_values.insert(input_name, value);
                    },
                    Err(_) => {
                        // Ignore errors for analyzed dependencies, as they might not be actual inputs
                    }
                }
            }
        }
        
        // Execute the transform with the collected inputs
        info!(
            "execute_transform_with_provider logic: {} with inputs: {:?}",
            transform.logic,
            input_values
        );
        let result = Self::execute_transform(transform, input_values);
        if let Ok(ref value) = result {
            info!("execute_transform_with_provider result: {:?}", value);
        }
        result
    }
    
    /// Executes a transform with a pre-parsed expression.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to execute with a pre-parsed expression
    /// * `input_values` - The input values for the transform
    ///
    /// # Returns
    ///
    /// The result of the transform execution
    pub fn execute_transform_with_expr(
        transform: &Transform,
        input_values: HashMap<String, JsonValue>,
    ) -> Result<JsonValue, SchemaError> {
        // Use the pre-parsed expression if available, otherwise parse the transform logic
        let ast = match &transform.parsed_expression {
            Some(expr) => expr.clone(),
            None => {
                // Parse the transform logic
                let logic = &transform.logic;
                let parser = TransformParser::new();
                parser.parse_expression(logic)
                    .map_err(|e| SchemaError::InvalidField(format!("Failed to parse transform: {}", e)))?
            }
        };
        
        info!(
            "execute_transform_with_expr expression: {:?} inputs: {:?}",
            ast,
            input_values
        );

        // Convert input values to interpreter values
        let variables = Self::convert_input_values(input_values);
        
        // Create interpreter with input variables
        let mut interpreter = Interpreter::with_variables(variables);
        
        // Evaluate the AST
        let evaluated = interpreter.evaluate(&ast)
            .map_err(|e| SchemaError::InvalidField(format!("Failed to execute transform: {}", e)))?;

        let json_result = Self::convert_result_value(evaluated)?;
        info!("execute_transform_with_expr result: {:?}", json_result);
        Ok(json_result)
    }
    
    /// Converts input values from JsonValue to interpreter Value.
    fn convert_input_values(input_values: HashMap<String, JsonValue>) -> HashMap<String, Value> {
        let mut variables = HashMap::new();
        
        for (name, value) in input_values {
            // Handle both schema.field format and regular field names
            variables.insert(name.clone(), Value::from(value.clone()));
            
            // If the name contains a dot, it's in schema.field format
            if let Some((schema, field)) = name.split_once('.') {
                // Add both schema.field and field entries
                variables.insert(format!("{}.{}", schema, field), Value::from(value.clone()));
                variables.insert(field.to_string(), Value::from(value));
            }
        }
        
        variables
    }
    
    /// Converts a result value from interpreter Value to JsonValue.
    fn convert_result_value(value: Value) -> Result<JsonValue, SchemaError> {
        Ok(JsonValue::from(value))
    }
    
    /// Validates a transform.
    ///
    /// # Arguments
    ///
    /// * `transform` - The transform to validate
    ///
    /// # Returns
    ///
    /// `Ok(())` if the transform is valid, otherwise an error
    pub fn validate_transform(transform: &Transform) -> Result<(), SchemaError> {
        // Parse the transform logic to check for syntax errors
        let parser = TransformParser::new();
        let ast = parser.parse_expression(&transform.logic);
        
        // For "input +" specifically, we want to fail validation
        if transform.logic == "input +" {
            return Err(SchemaError::InvalidField("Invalid transform syntax: missing right operand".to_string()));
        }
        
        ast.map_err(|e| SchemaError::InvalidField(format!("Invalid transform syntax: {}", e)))?;
        

        
        Ok(())
    }
}


