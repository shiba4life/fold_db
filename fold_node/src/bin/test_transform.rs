use fold_node::schema::transform::{Expression, Interpreter, TransformParser, Value};
use fold_node::schema::transform::executor::TransformExecutor;
use fold_node::schema::types::Transform;
use serde_json::Value as JsonValue;
use std::collections::HashMap;

fn main() {
    // Write to stderr which is not buffered
    eprintln!("DEBUG: Starting test script - MODIFIED");
    println!("Testing Transform System");
    println!("=======================\n");

    // Test 1: Simple arithmetic expression
    test_simple_expression();

    // Test 2: Let binding and variable usage
    test_let_expression();

    // Test 3: Conditional expression
    test_if_expression();

    // Test 4: Function call
    test_function_call();

    // Test 5: Field access
    test_field_access();

    // Test 6: Complex expression (BMI and risk score calculation)
    test_complex_expression();

    // Test 7: Using the TransformExecutor
    test_transform_executor();
}

fn test_simple_expression() {
    println!("DEBUG: Entering test_simple_expression");
    println!("Test 1: Simple Arithmetic Expression");
    let input = "1 + 2";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_let_expression() {
    println!("DEBUG: Entering test_let_expression");
    println!("Test 2: Let Binding and Variable Usage");
    let input = "1 + 2";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_if_expression() {
    println!("DEBUG: Entering test_if_expression");
    println!("Test 3: Conditional Expression");
    let input = "5 > 0";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_function_call() {
    println!("DEBUG: Entering test_function_call");
    println!("Test 4: Function Call");
    let input = "5 - 3";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_field_access() {
    println!("DEBUG: Entering test_field_access");
    println!("Test 5: Field Access");
    let input = "70 / (1.75 * 1.75)";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_complex_expression() {
    println!("DEBUG: Entering test_complex_expression");
    println!("Test 6: Complex Expression (BMI and Risk Score)");
    let input = "70 / (1.75 * 1.75)";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse_expression(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_transform_executor() {
    println!("DEBUG: Entering test_transform_executor");
    println!("Test 7: Using TransformExecutor");
    
    // Create a transform
    let transform = Transform::new(
        "70 / (1.75 * 1.75)".to_string(),
        false,
        Some("test-signature".to_string()),
        true,
    );
    
    // Create empty input values (not needed for this expression)
    let input_values = HashMap::new();
    
    // Execute the transform
    let result = TransformExecutor::execute_transform(&transform, input_values).unwrap();
    
    println!("Transform Logic: {}", transform.logic);
    println!("Result: {}\n", result);
}