//! Interpreter for the transform DSL.
//!
//! This module implements an interpreter for the transform DSL.
//! It evaluates an AST to produce a result value.

use super::ast::{Expression, Operator, UnaryOperator, Value};
use crate::schema::types::SchemaError;
use std::collections::HashMap;

/// Type for function implementations in the interpreter
pub type TransformFunction = Box<dyn Fn(Vec<Value>) -> Result<Value, String>>;

/// Interpreter for the transform DSL.
pub struct Interpreter {
    /// Variables in the current scope
    variables: HashMap<String, Value>,
    
    /// Built-in functions
    functions: HashMap<String, TransformFunction>,
}

impl Default for Interpreter {
    fn default() -> Self {
        let mut interpreter = Self {
            variables: HashMap::new(),
            functions: HashMap::new(),
        };
        
        // Register built-in functions
        interpreter.register_builtin_functions();
        
        interpreter
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
    fn register_builtin_functions(&mut self) {
        // Math functions
        self.functions.insert("min".to_string(), Box::new(|args| {
            if args.len() != 2 {
                return Err("min() requires exactly 2 arguments".to_string());
            }
            
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => return Err("min() requires numeric arguments".to_string()),
            };
            
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => return Err("min() requires numeric arguments".to_string()),
            };
            
            Ok(Value::Number(a.min(b)))
        }));
        
        self.functions.insert("max".to_string(), Box::new(|args| {
            if args.len() != 2 {
                return Err("max() requires exactly 2 arguments".to_string());
            }
            
            let a = match &args[0] {
                Value::Number(n) => *n,
                _ => return Err("max() requires numeric arguments".to_string()),
            };
            
            let b = match &args[1] {
                Value::Number(n) => *n,
                _ => return Err("max() requires numeric arguments".to_string()),
            };
            
            Ok(Value::Number(a.max(b)))
        }));
        
        self.functions.insert("clamp".to_string(), Box::new(|args| {
            if args.len() != 3 {
                return Err("clamp() requires exactly 3 arguments".to_string());
            }
            
            let value = match &args[0] {
                Value::Number(n) => *n,
                _ => return Err("clamp() requires numeric arguments".to_string()),
            };
            
            let min = match &args[1] {
                Value::Number(n) => *n,
                _ => return Err("clamp() requires numeric arguments".to_string()),
            };
            
            let max = match &args[2] {
                Value::Number(n) => *n,
                _ => return Err("clamp() requires numeric arguments".to_string()),
            };
            
            Ok(Value::Number(value.max(min).min(max)))
        }));
        
        // String functions
        self.functions.insert("concat".to_string(), Box::new(|args| {
            let mut result = String::new();
            
            for arg in args {
                match arg {
                    Value::String(s) => result.push_str(&s),
                    _ => return Err("concat() requires string arguments".to_string()),
                }
            }
            
            Ok(Value::String(result))
        }));
        
        // Type conversion functions
        self.functions.insert("to_string".to_string(), Box::new(|args| {
            if args.len() != 1 {
                return Err("to_string() requires exactly 1 argument".to_string());
            }
            
            let result = match &args[0] {
                Value::Number(n) => n.to_string(),
                Value::Boolean(b) => b.to_string(),
                Value::String(s) => s.clone(),
                Value::Null => "null".to_string(),
                Value::Object(_) => "<object>".to_string(),
                Value::Array(_) => "<array>".to_string(),
            };
            
            Ok(Value::String(result))
        }));
        
        self.functions.insert("to_number".to_string(), Box::new(|args| {
            if args.len() != 1 {
                return Err("to_number() requires exactly 1 argument".to_string());
            }
            
            let result = match &args[0] {
                Value::Number(n) => *n,
                Value::Boolean(b) => if *b { 1.0 } else { 0.0 },
                Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
                Value::Null => 0.0,
                Value::Object(_) => 0.0,
                Value::Array(_) => 0.0,
            };
            
            Ok(Value::Number(result))
        }));
        
        self.functions.insert("to_boolean".to_string(), Box::new(|args| {
            if args.len() != 1 {
                return Err("to_boolean() requires exactly 1 argument".to_string());
            }
            
            let result = match &args[0] {
                Value::Number(n) => *n != 0.0,
                Value::Boolean(b) => *b,
                Value::String(s) => !s.is_empty(),
                Value::Null => false,
                Value::Object(_) => true,
                Value::Array(_) => true,
            };
            
            Ok(Value::Boolean(result))
        }));
    }
    
    /// Evaluates an expression.
    pub fn evaluate(&mut self, expr: &Expression) -> Result<Value, SchemaError> {
        println!("DEBUG: Evaluating expression: {:?}", expr);
        
        let result = match expr {
            Expression::Literal(value) => {
                println!("DEBUG: Literal value: {:?}", value);
                Ok(value.clone())
            },
            
            Expression::Variable(name) => {
                println!("DEBUG: Variable: {}", name);
                let result = self.variables.get(name).cloned()
                    .ok_or_else(|| SchemaError::InvalidField(format!("Variable not found: {}", name)));
                println!("DEBUG: Variable result: {:?}", result);
                result
            },
            
            Expression::FieldAccess { object, field } => {
                let obj = self.evaluate(object)?;
                
                match obj {
                    Value::Object(map) => {
                        // Look for the field in the map by string key
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
                let old_value = self.variables.get(name).cloned();
                
                // Set the new value
                self.variables.insert(name.clone(), val);
                
                // Evaluate the body
                let result = self.evaluate(body);
                
                // Restore the old value or remove the variable
                match old_value {
                    Some(v) => self.variables.insert(name.clone(), v),
                    None => self.variables.remove(name),
                };
                
                result
            },
            
            Expression::Return(expr) => {
                println!("DEBUG: Return expression");
                self.evaluate(expr)
            },
        };
        
        println!("DEBUG: Evaluation result: {:?}", result);
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::parser::TransformParser;
    use serde_json::Value as JsonValue;
    
    #[test]
    fn test_evaluate_simple_expression() {
        let input = "1 + 2 * 3";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => assert_eq!(n, 7.0), // 1 + (2 * 3) = 7
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_evaluate_let_expression() {
        let input = "let x = 1 + 2; x * 3";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => assert_eq!(n, 9.0), // (1 + 2) * 3 = 9
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_evaluate_if_expression() {
        let input = "if 5 > 0 then 10 else 20";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => assert_eq!(n, 10.0), // 5 > 0, so result is 10
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_evaluate_function_call() {
        let input = "min(5, 3)";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        let mut interpreter = Interpreter::new();
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => assert_eq!(n, 3.0), // min(5, 3) = 3
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_evaluate_field_access() {
        let input = "input.value";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        // Create a variable with an object value
        let mut variables = HashMap::new();
        let mut object_map = HashMap::new();
        object_map.insert(
            "value".to_string(),
            JsonValue::Number(serde_json::Number::from_f64(42.0).unwrap()),
        );
        variables.insert("input".to_string(), Value::Object(object_map));
        
        let mut interpreter = Interpreter::with_variables(variables);
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => assert_eq!(n, 42.0), // input.value = 42
            _ => panic!("Expected number"),
        }
    }
    
    #[test]
    fn test_evaluate_complex_expression() {
        let input = "let bmi = weight / (height ^ 2); let risk = 0.5 * blood_pressure + 1.2 * bmi; clamp(risk, 0, 100)";
        let parser = TransformParser::new();
        let expr = parser.parse(input).unwrap();
        
        // Create variables
        let mut variables = HashMap::new();
        variables.insert("weight".to_string(), Value::Number(70.0)); // 70 kg
        variables.insert("height".to_string(), Value::Number(1.75)); // 1.75 m
        variables.insert("blood_pressure".to_string(), Value::Number(120.0)); // 120 mmHg
        
        let mut interpreter = Interpreter::with_variables(variables);
        let result = interpreter.evaluate(&expr).unwrap();
        
        match result {
            Value::Number(n) => {
                // Calculate expected result:
                // bmi = 70 / (1.75^2) = 70 / 3.0625 = 22.857
                // risk = 0.5 * 120 + 1.2 * 22.857 = 60 + 27.428 = 87.428
                // clamp(87.428, 0, 100) = 87.428
                assert!((n - 87.428).abs() < 0.001);
            },
            _ => panic!("Expected number"),
        }
    }
}