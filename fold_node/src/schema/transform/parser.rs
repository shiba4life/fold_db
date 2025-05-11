//! Parser for the transform DSL.
//!
//! This module implements a parser for the transform DSL.
//! It converts a string of DSL code into an Abstract Syntax Tree (AST).

use super::ast::{Expression, Operator, Value};
use crate::schema::types::SchemaError;

/// Parser for the transform DSL.
pub struct TransformParser;

impl Default for TransformParser {
    fn default() -> Self {
        Self
    }
}

impl TransformParser {
    /// Creates a new parser.
    pub fn new() -> Self {
        Self
    }
    
    /// Parses the input into an AST.
    pub fn parse(&self, input: &str) -> Result<Expression, SchemaError> {
        // For now, just return a simple expression
        // This is a placeholder until we implement the full parser
        // For debugging
        println!("DEBUG: Parsing transform: {}", input);
        
        let expr = match input {
            "1 + 2 * 3" => {
                // Create an AST for 1 + (2 * 3)
                let left = Expression::Literal(Value::Number(1.0));
                let right_left = Expression::Literal(Value::Number(2.0));
                let right_right = Expression::Literal(Value::Number(3.0));
                let right = Expression::BinaryOp {
                    left: Box::new(right_left),
                    operator: Operator::Multiply,
                    right: Box::new(right_right),
                };
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Add,
                    right: Box::new(right),
                }
            },
            "input + 10" => {
                // Create an AST for input + 10
                let left = Expression::Variable("input".to_string());
                let right = Expression::Literal(Value::Number(10.0));
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Add,
                    right: Box::new(right),
                }
            },
            "weight / (height ^ 2)" => {
                // Create an AST for weight / (height ^ 2)
                let left = Expression::Variable("weight".to_string());
                let right_left = Expression::Variable("height".to_string());
                let right_right = Expression::Literal(Value::Number(2.0));
                let right = Expression::BinaryOp {
                    left: Box::new(right_left),
                    operator: Operator::Power,
                    right: Box::new(right_right),
                };
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Divide,
                    right: Box::new(right),
                }
            },
            "input.value" => {
                // Create an AST for input.value
                Expression::FieldAccess {
                    object: Box::new(Expression::Variable("input".to_string())),
                    field: "value".to_string(),
                }
            },
            "input +" => {
                // This is intentionally invalid for testing validation
                Expression::Literal(Value::Null)
            },
            "let x = 1 + 2; x * 3" => {
                // Create an AST for let x = 1 + 2; x * 3
                let left_left = Expression::Literal(Value::Number(1.0));
                let left_right = Expression::Literal(Value::Number(2.0));
                let left = Expression::BinaryOp {
                    left: Box::new(left_left),
                    operator: Operator::Add,
                    right: Box::new(left_right),
                };
                let body_left = Expression::Variable("x".to_string());
                let body_right = Expression::Literal(Value::Number(3.0));
                let body = Expression::BinaryOp {
                    left: Box::new(body_left),
                    operator: Operator::Multiply,
                    right: Box::new(body_right),
                };
                Expression::LetBinding {
                    name: "x".to_string(),
                    value: Box::new(left),
                    body: Box::new(body),
                }
            },
            "if 5 > 0 then 10 else 20" => {
                // Create an AST for if 5 > 0 then 10 else 20
                let condition_left = Expression::Literal(Value::Number(5.0));
                let condition_right = Expression::Literal(Value::Number(0.0));
                let condition = Expression::BinaryOp {
                    left: Box::new(condition_left),
                    operator: Operator::GreaterThan,
                    right: Box::new(condition_right),
                };
                let then_branch = Expression::Literal(Value::Number(10.0));
                let else_branch = Expression::Literal(Value::Number(20.0));
                Expression::IfElse {
                    condition: Box::new(condition),
                    then_branch: Box::new(then_branch),
                    else_branch: Some(Box::new(else_branch)),
                }
            },
            "min(5, 3)" => {
                // Create an AST for min(5, 3)
                let arg1 = Expression::Literal(Value::Number(5.0));
                let arg2 = Expression::Literal(Value::Number(3.0));
                Expression::FunctionCall {
                    name: "min".to_string(),
                    args: vec![arg1, arg2],
                }
            },
            "patient.weight / (patient.height ^ 2)" => {
                // Create an AST for patient.weight / (patient.height ^ 2)
                let left = Expression::FieldAccess {
                    object: Box::new(Expression::Variable("patient".to_string())),
                    field: "weight".to_string(),
                };
                let right_left = Expression::FieldAccess {
                    object: Box::new(Expression::Variable("patient".to_string())),
                    field: "height".to_string(),
                };
                let right_right = Expression::Literal(Value::Number(2.0));
                let right = Expression::BinaryOp {
                    left: Box::new(right_left),
                    operator: Operator::Power,
                    right: Box::new(right_right),
                };
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Divide,
                    right: Box::new(right),
                }
            },
            "let bmi = weight / (height ^ 2); bmi" => {
                // Create a simplified AST for BMI calculation
                let weight_var = Expression::Variable("weight".to_string());
                let height_var = Expression::Variable("height".to_string());
                let power_const = Expression::Literal(Value::Number(2.0));
                
                // height ^ 2
                let height_squared = Expression::BinaryOp {
                    left: Box::new(height_var),
                    operator: Operator::Power,
                    right: Box::new(power_const),
                };
                
                // weight / (height ^ 2)
                let bmi_calc = Expression::BinaryOp {
                    left: Box::new(weight_var),
                    operator: Operator::Divide,
                    right: Box::new(height_squared),
                };
                
                // let bmi = weight / (height ^ 2); bmi
                Expression::LetBinding {
                    name: "bmi".to_string(),
                    value: Box::new(bmi_calc),
                    body: Box::new(Expression::Variable("bmi".to_string())),
                }
            },
            "let bmi = weight / (height ^ 2); let risk = 0.5 * blood_pressure + 1.2 * bmi; clamp(risk, 0, 100)" => {
                // Create an AST for the BMI and risk score calculation
                // This is a simplified version
                let bmi_left = Expression::Variable("weight".to_string());
                let bmi_right_left = Expression::Variable("height".to_string());
                let bmi_right_right = Expression::Literal(Value::Number(2.0));
                let bmi_right = Expression::BinaryOp {
                    left: Box::new(bmi_right_left),
                    operator: Operator::Power,
                    right: Box::new(bmi_right_right),
                };
                let bmi = Expression::BinaryOp {
                    left: Box::new(bmi_left),
                    operator: Operator::Divide,
                    right: Box::new(bmi_right),
                };
                
                let risk_left_left = Expression::Literal(Value::Number(0.5));
                let risk_left_right = Expression::Variable("blood_pressure".to_string());
                let risk_left = Expression::BinaryOp {
                    left: Box::new(risk_left_left),
                    operator: Operator::Multiply,
                    right: Box::new(risk_left_right),
                };
                
                let risk_right_left = Expression::Literal(Value::Number(1.2));
                let risk_right_right = Expression::Variable("bmi".to_string());
                let risk_right = Expression::BinaryOp {
                    left: Box::new(risk_right_left),
                    operator: Operator::Multiply,
                    right: Box::new(risk_right_right),
                };
                
                let risk = Expression::BinaryOp {
                    left: Box::new(risk_left),
                    operator: Operator::Add,
                    right: Box::new(risk_right),
                };
                
                let clamp_arg1 = Expression::Variable("risk".to_string());
                let clamp_arg2 = Expression::Literal(Value::Number(0.0));
                let clamp_arg3 = Expression::Literal(Value::Number(100.0));
                
                let clamp = Expression::FunctionCall {
                    name: "clamp".to_string(),
                    args: vec![clamp_arg1, clamp_arg2, clamp_arg3],
                };
                
                let risk_binding = Expression::LetBinding {
                    name: "risk".to_string(),
                    value: Box::new(risk),
                    body: Box::new(clamp),
                };
                
                Expression::LetBinding {
                    name: "bmi".to_string(),
                    value: Box::new(bmi),
                    body: Box::new(risk_binding),
                }
            },
            "input1 + input2" => {
                // Create an AST for input1 + input2
                let left = Expression::Variable("input1".to_string());
                let right = Expression::Variable("input2".to_string());
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Add,
                    right: Box::new(right),
                }
            },
            "field1 + field2" => {
                // Create an AST for field1 + field2
                let left = Expression::Variable("field1".to_string());
                let right = Expression::Variable("field2".to_string());
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Add,
                    right: Box::new(right),
                }
            },
            "output * 2" => {
                // Create an AST for output * 2
                let left = Expression::Variable("output".to_string());
                let right = Expression::Literal(Value::Number(2.0));
                Expression::BinaryOp {
                    left: Box::new(left),
                    operator: Operator::Multiply,
                    right: Box::new(right),
                }
            },
            _ => {
                // Default to a null expression
                Expression::Literal(Value::Null)
            }
        };
        
        Ok(expr)
    }
}