use fold_node::transform::{
    Expression, Operator, Value, Interpreter,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[test]
fn test_evaluate_simple_expression() {
    // Create a simple binary expression manually
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::Literal(Value::Number(1.0))),
        operator: Operator::Add,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Number(2.0))),
            operator: Operator::Multiply,
            right: Box::new(Expression::Literal(Value::Number(3.0))),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();

    match result {
        Value::Number(n) => assert_eq!(n, 7.0), // 1 + (2 * 3) = 7
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_evaluate_let_expression() {
    // Create a let expression manually
    let expr = Expression::LetBinding {
        name: "x".to_string(),
        value: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Number(1.0))),
            operator: Operator::Add,
            right: Box::new(Expression::Literal(Value::Number(2.0))),
        }),
        body: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Variable("x".to_string())),
            operator: Operator::Multiply,
            right: Box::new(Expression::Literal(Value::Number(3.0))),
        }),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();

    match result {
        Value::Number(n) => assert_eq!(n, 9.0), // (1 + 2) * 3 = 9
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_evaluate_if_expression() {
    // Create an if expression manually
    let expr = Expression::IfElse {
        condition: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Number(5.0))),
            operator: Operator::GreaterThan,
            right: Box::new(Expression::Literal(Value::Number(0.0))),
        }),
        then_branch: Box::new(Expression::Literal(Value::Number(10.0))),
        else_branch: Some(Box::new(Expression::Literal(Value::Number(20.0)))),
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();

    match result {
        Value::Number(n) => assert_eq!(n, 10.0), // 5 > 0, so result is 10
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_evaluate_function_call() {
    // Create a function call expression manually
    let expr = Expression::FunctionCall {
        name: "min".to_string(),
        args: vec![
            Expression::Literal(Value::Number(5.0)),
            Expression::Literal(Value::Number(3.0)),
        ],
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();

    match result {
        Value::Number(n) => assert_eq!(n, 3.0), // min(5, 3) = 3
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_evaluate_field_access() {
    // Create a field access expression manually
    let expr = Expression::FieldAccess {
        object: Box::new(Expression::Variable("input".to_string())),
        field: "value".to_string(),
    };

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
    // Create a complex expression manually
    let expr = Expression::LetBinding {
        name: "bmi".to_string(),
        value: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Variable("weight".to_string())),
            operator: Operator::Divide,
            right: Box::new(Expression::BinaryOp {
                left: Box::new(Expression::Variable("height".to_string())),
                operator: Operator::Power,
                right: Box::new(Expression::Literal(Value::Number(2.0))),
            }),
        }),
        body: Box::new(Expression::LetBinding {
            name: "risk".to_string(),
            value: Box::new(Expression::BinaryOp {
                left: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Literal(Value::Number(0.5))),
                    operator: Operator::Multiply,
                    right: Box::new(Expression::Variable("blood_pressure".to_string())),
                }),
                operator: Operator::Add,
                right: Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Literal(Value::Number(1.2))),
                    operator: Operator::Multiply,
                    right: Box::new(Expression::Variable("bmi".to_string())),
                }),
            }),
            body: Box::new(Expression::FunctionCall {
                name: "clamp".to_string(),
                args: vec![
                    Expression::Variable("risk".to_string()),
                    Expression::Literal(Value::Number(0.0)),
                    Expression::Literal(Value::Number(100.0)),
                ],
            }),
        }),
    };

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
        }
        _ => panic!("Expected number"),
    }
}
