//! Interpreter for the transform DSL.
//!
//! This module implements an interpreter for the transform DSL.
//! It evaluates an AST to produce a result value.

use super::ast::{Expression, Operator, UnaryOperator, Value};
use crate::schema::types::SchemaError;
use std::collections::HashMap;

pub mod builtins;
pub use builtins::{builtin_functions, TransformFunction};


/// Interpreter for the transform DSL.
pub struct Interpreter {
    /// Variables in the current scope
    variables: HashMap<String, Value>,
    
    /// Built-in functions
    functions: HashMap<String, TransformFunction>,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self {
            variables: HashMap::new(),
            functions: builtin_functions(),
        }
    }
}

impl Interpreter {
    /// Creates a new interpreter.
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Creates a new interpreter with the given variables.
    pub fn with_variables(variables: HashMap<String, Value>) -> Self {
        let mut interpreter = Self::new();
        interpreter.variables = variables;
        interpreter
    }
    
    /// Registers built-in functions.
    #[allow(dead_code)]
    fn register_builtin_functions(&mut self) {
        self.functions.extend(builtin_functions());
    }
    
    /// Evaluates an expression.
    pub fn evaluate(&mut self, expr: &Expression) -> Result<Value, SchemaError> {
        // Evaluate the expression
        let result = match expr {
            Expression::Literal(value) => {
                Ok(value.clone())
            },
            
            Expression::Variable(name) => {
                self.variables.get(name).cloned()
                    .ok_or_else(|| SchemaError::InvalidField(format!("Variable not found: {}", name)))
            },
            
            Expression::FieldAccess { object, field } => {
                // Handle schema.field references
                if let Expression::Variable(schema_name) = &**object {
                    // Look up schema.field in variables
                    let key = format!("{}.{}", schema_name, field);
                    if let Some(value) = self.variables.get(&key) {
                        return Ok(value.clone());
                    }
                    
                    // Check if the schema name is a variable containing an object
                    if let Some(Value::Object(map)) = self.variables.get(schema_name) {
                        if let Some(value) = map.get(field) {
                            return Ok(Value::from(value.clone()));
                        }
                    }
                    
                    // Fall back to looking up just the field name
                    return self.variables.get(field).cloned()
                        .ok_or_else(|| SchemaError::InvalidField(format!("Field not found: {}", field)));
                }
                
                // Handle regular object field access
                let obj = self.evaluate(object)?;
                match obj {
                    Value::Object(map) => {
                        if let Some(value) = map.get(field) {
                            Ok(Value::from(value.clone()))
                        } else {
                            Err(SchemaError::InvalidField(format!("Field not found: {}", field)))
                        }
                    },
                    _ => Err(SchemaError::InvalidField(format!("Cannot access field {} on non-object value", field))),
                }
            },
            
            Expression::BinaryOp { left, operator, right } => {
                let left_val = self.evaluate(left)?;
                let right_val = self.evaluate(right)?;
                
                match operator {
                    Operator::Add => self.add(&left_val, &right_val),
                    Operator::Subtract => self.subtract(&left_val, &right_val),
                    Operator::Multiply => self.multiply(&left_val, &right_val),
                    Operator::Divide => self.divide(&left_val, &right_val),
                    Operator::Power => self.power(&left_val, &right_val),
                    Operator::Equal => self.equal(&left_val, &right_val),
                    Operator::NotEqual => self.not_equal(&left_val, &right_val),
                    Operator::LessThan => self.less_than(&left_val, &right_val),
                    Operator::LessThanOrEqual => self.less_than_or_equal(&left_val, &right_val),
                    Operator::GreaterThan => self.greater_than(&left_val, &right_val),
                    Operator::GreaterThanOrEqual => self.greater_than_or_equal(&left_val, &right_val),
                    Operator::And => self.and(&left_val, &right_val),
                    Operator::Or => self.or(&left_val, &right_val),
                }
            },
            
            Expression::UnaryOp { operator, expr } => {
                let val = self.evaluate(expr)?;
                
                match operator {
                    UnaryOperator::Negate => self.negate(&val),
                    UnaryOperator::Not => self.not(&val),
                }
            },
            
            Expression::FunctionCall { name, args } => {
                let mut evaluated_args = Vec::new();
                
                for arg in args {
                    evaluated_args.push(self.evaluate(arg)?);
                }
                
                if let Some(func) = self.functions.get(name) {
                    func(evaluated_args).map_err(SchemaError::InvalidField)
                } else {
                    Err(SchemaError::InvalidField(format!("Function not found: {}", name)))
                }
            },
            
            Expression::IfElse { condition, then_branch, else_branch } => {
                let cond = self.evaluate(condition)?;
                
                match cond {
                    Value::Boolean(true) => self.evaluate(then_branch),
                    Value::Boolean(false) => {
                        if let Some(else_expr) = else_branch {
                            self.evaluate(else_expr)
                        } else {
                            Ok(Value::Null)
                        }
                    },
                    _ => Err(SchemaError::InvalidField("Condition must be a boolean".to_string())),
                }
            },
            
            Expression::LetBinding { name, value, body } => {
                let val = self.evaluate(value)?;
                
                // Save the old value if it exists
                let _old_value = self.variables.get(name).cloned();
                
                // Set the new value
                self.variables.insert(name.clone(), val.clone());
                
                // If the body is Null, just return the value (for sequential evaluation)
                // Don't restore the old value or remove the variable for sequential evaluation
                // This allows variables to persist between statements in the transform logic
                if let Expression::Literal(Value::Null) = **body {
                    Ok(val)
                } else {
                    // Otherwise evaluate the body
                    self.evaluate(body)
                }
            },
            
            Expression::Return(expr) => {
                self.evaluate(expr)
            },
        };
        
        result
    }
    
    // Operator implementations
    
    fn add(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(SchemaError::InvalidField("Cannot add values of different types".to_string())),
        }
    }
    
    fn subtract(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            _ => Err(SchemaError::InvalidField("Cannot subtract non-numeric values".to_string())),
        }
    }
    
    fn multiply(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            _ => Err(SchemaError::InvalidField("Cannot multiply non-numeric values".to_string())),
        }
    }
    
    fn divide(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => {
                if *b == 0.0 {
                    Err(SchemaError::InvalidField("Division by zero".to_string()))
                } else {
                    Ok(Value::Number(a / b))
                }
            },
            _ => Err(SchemaError::InvalidField("Cannot divide non-numeric values".to_string())),
        }
    }
    
    fn power(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a.powf(*b))),
            _ => Err(SchemaError::InvalidField("Cannot raise non-numeric values to a power".to_string())),
        }
    }
    
    fn equal(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a == b)),
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(a == b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a == b)),
            (Value::Null, Value::Null) => Ok(Value::Boolean(true)),
            _ => Ok(Value::Boolean(false)),
        }
    }
    
    fn not_equal(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        self.equal(left, right).map(|v| match v {
            Value::Boolean(b) => Value::Boolean(!b),
            _ => unreachable!(),
        })
    }
    
    fn less_than(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a < b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(SchemaError::InvalidField("Cannot compare non-comparable values".to_string())),
        }
    }
    
    fn less_than_or_equal(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a <= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(SchemaError::InvalidField("Cannot compare non-comparable values".to_string())),
        }
    }
    
    fn greater_than(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a > b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(SchemaError::InvalidField("Cannot compare non-comparable values".to_string())),
        }
    }
    
    fn greater_than_or_equal(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Boolean(a >= b)),
            (Value::String(a), Value::String(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(SchemaError::InvalidField("Cannot compare non-comparable values".to_string())),
        }
    }
    
    fn and(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a && *b)),
            _ => Err(SchemaError::InvalidField("Cannot perform logical AND on non-boolean values".to_string())),
        }
    }
    
    fn or(&self, left: &Value, right: &Value) -> Result<Value, SchemaError> {
        match (left, right) {
            (Value::Boolean(a), Value::Boolean(b)) => Ok(Value::Boolean(*a || *b)),
            _ => Err(SchemaError::InvalidField("Cannot perform logical OR on non-boolean values".to_string())),
        }
    }
    
    fn negate(&self, value: &Value) -> Result<Value, SchemaError> {
        match value {
            Value::Number(n) => Ok(Value::Number(-n)),
            _ => Err(SchemaError::InvalidField("Cannot negate non-numeric value".to_string())),
        }
    }
    
    fn not(&self, value: &Value) -> Result<Value, SchemaError> {
        match value {
            Value::Boolean(b) => Ok(Value::Boolean(!b)),
            _ => Err(SchemaError::InvalidField("Cannot perform logical NOT on non-boolean value".to_string())),
        }
    }
}


