use fold_node::schema::transform::{TransformParser, Interpreter, Value, Expression};
use fold_node::schema::types::Transform;
use fold_node::schema::transform::executor::TransformExecutor;
use std::collections::HashMap;
use serde_json::Value as JsonValue;

fn main() {
    println!("DEBUG: Starting test script");
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
    let input = "1 + 2 * 3";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_let_expression() {
    println!("DEBUG: Entering test_let_expression");
    println!("Test 2: Let Binding and Variable Usage");
    let input = "let x = 1 + 2; x * 3";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_if_expression() {
    println!("DEBUG: Entering test_if_expression");
    println!("Test 3: Conditional Expression");
    let input = "if 5 > 0 then 10 else 20";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_function_call() {
    println!("Test 4: Function Call");
    let input = "min(5, 3)";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_field_access() {
    println!("Test 5: Field Access");
    let input = "patient.weight / (patient.height ^ 2)";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    // Create variables with object values
    let mut variables = HashMap::new();
    let mut patient = HashMap::new();
    
    patient.insert("weight".to_string(), JsonValue::Number(serde_json::Number::from_f64(70.0).unwrap()));
    patient.insert("height".to_string(), JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap()));
    
    variables.insert("patient".to_string(), Value::Object(patient));
    
    let mut interpreter = Interpreter::with_variables(variables);
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_complex_expression() {
    println!("Test 6: Complex Expression (BMI and Risk Score)");
    let input = "let bmi = weight / (height ^ 2); let risk = 0.5 * blood_pressure + 1.2 * bmi; clamp(risk, 0, 100)";
    println!("Expression: {}", input);
    
    let parser = TransformParser::new();
    let expr = parser.parse(input).unwrap();
    
    // Create variables
    let mut variables = HashMap::new();
    variables.insert("weight".to_string(), Value::Number(70.0)); // 70 kg
    variables.insert("height".to_string(), Value::Number(1.75)); // 1.75 m
    variables.insert("blood_pressure".to_string(), Value::Number(120.0)); // 120 mmHg
    
    let mut interpreter = Interpreter::with_variables(variables);
    let result = interpreter.evaluate(&expr).unwrap();
    
    println!("Result: {}\n", result);
}

fn test_transform_executor() {
    println!("Test 7: Using TransformExecutor");
    
    // Create a transform
    let transform = Transform::new(
        "let bmi = weight / (height ^ 2); let risk = 0.5 * blood_pressure + 1.2 * bmi; clamp(risk, 0, 100)".to_string(),
        false,
        Some("test-signature".to_string()),
        true,
    );
    
    // Create input values
    let mut input_values = HashMap::new();
    input_values.insert("weight".to_string(), JsonValue::Number(serde_json::Number::from_f64(70.0).unwrap()));
    input_values.insert("height".to_string(), JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap()));
    input_values.insert("blood_pressure".to_string(), JsonValue::Number(serde_json::Number::from_f64(120.0).unwrap()));
    
    // Execute the transform
    let result = TransformExecutor::execute_transform(&transform, input_values).unwrap();
    
    println!("Transform Logic: {}", transform.logic);
    println!("Result: {}\n", result);
}