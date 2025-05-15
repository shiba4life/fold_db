//! Tests for the better transform parser and interpreter.

use super::ast::{Expression, Operator, Value};
use super::parser::BetterParser;
use super::interpreter::Interpreter;
use std::collections::HashMap;

#[test]
fn test_parser_and_interpreter() {
    // Create a parser and interpreter
    let parser = BetterParser::new();
    let mut interpreter = Interpreter::new();
    
    // Test simple arithmetic
    let expr = parser.parse_expression("2 + 3").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(5.0));
    
    // Test operator precedence
    let expr = parser.parse_expression("2 + 3 * 4").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(14.0));
    
    // Test parentheses
    let expr = parser.parse_expression("(2 + 3) * 4").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(20.0));
    
    // Test variables
    let mut variables = HashMap::new();
    variables.insert("x".to_string(), Value::Number(10.0));
    variables.insert("y".to_string(), Value::Number(5.0));
    let mut interpreter = Interpreter::with_variables(variables);
    
    let expr = parser.parse_expression("x + y").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(15.0));
    
    // Test function calls
    let expr = parser.parse_expression("min(x, y)").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(5.0));
    
    // Test complex expressions
    let expr = parser.parse_expression("x > y && x + y == 15").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Boolean(true));
    
    // Test field access with a JSON object
    let mut variables = HashMap::new();
    let mut obj = HashMap::new();
    obj.insert("field".to_string(), serde_json::json!(42));
    variables.insert("obj".to_string(), Value::Object(obj));
    let mut interpreter = Interpreter::with_variables(variables);
    
    let expr = parser.parse_expression("obj.field").unwrap();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(42.0));
}

#[test]
fn test_parser_with_complex_expressions() {
    let parser = BetterParser::new();
    
    // Test a more complex expression
    let expr_str = "min(max(x, y) * 2, z + 10) > 5 && !isNegative";
    let expr = parser.parse_expression(expr_str).unwrap();
    
    // Verify the structure of the parsed expression
    match expr {
        Expression::BinaryOp { left, operator, right } => {
            assert_eq!(operator, Operator::And);
            
            // Check left side: min(max(x, y) * 2, z + 10) > 5
            match *left {
                Expression::BinaryOp { operator, .. } => {
                    assert_eq!(operator, Operator::GreaterThan);
                },
                _ => panic!("Expected BinaryOp for left side"),
            }
            
            // Check right side: !isNegative
            match *right {
                Expression::UnaryOp { .. } => {
                    // This is correct
                },
                _ => panic!("Expected UnaryOp for right side"),
            }
        },
        _ => panic!("Expected BinaryOp for top level"),
    }
    
    // Test another complex expression
    let expr_str = "if x > 0 then x * 2 else x / 2";
    assert!(parser.parse_expression(expr_str).is_err()); // Our parser doesn't support if-then-else yet
}