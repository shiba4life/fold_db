use fold_node::transform::{
    Expression, Operator, Value, Transform, TransformExecutor, TransformParser,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[test]
fn test_execute_complex_transform() {
    // Create a complex transform (BMI calculation) with a manually constructed expression
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
        body: Box::new(Expression::Variable("bmi".to_string())),
    };

    let transform = Transform::new_with_expr(
        "let bmi = weight / (height ^ 2); bmi".to_string(),
        expr,
        "test.bmi".to_string(),
    );

    // Create input values
    let mut input_values = HashMap::new();
    input_values.insert(
        "weight".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(70.0).unwrap()),
    );
    input_values.insert(
        "height".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap()),
    );

    // Execute the transform
    let result =
        TransformExecutor::execute_transform_with_expr(&transform, input_values).unwrap();

    // Check the result (BMI = 70 / (1.75^2) = 70 / 3.0625 = 22.857)
    match result {
        JsonValue::Number(n) => {
            let value = n.as_f64().unwrap();
            assert!((value - 22.857).abs() < 0.001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_execute_transform_with_field_access() {
    // Create a transform that accesses object fields with a manually constructed expression
    let expr = Expression::BinaryOp {
        left: Box::new(Expression::FieldAccess {
            object: Box::new(Expression::Variable("patient".to_string())),
            field: "weight".to_string(),
        }),
        operator: Operator::Divide,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::FieldAccess {
                object: Box::new(Expression::Variable("patient".to_string())),
                field: "height".to_string(),
            }),
            operator: Operator::Power,
            right: Box::new(Expression::Literal(Value::Number(2.0))),
        }),
    };

    let transform = Transform::new_with_expr(
        "patient.weight / (patient.height ^ 2)".to_string(),
        expr,
        "test.bmi".to_string(),
    );

    // Create input values with nested objects
    let mut input_values = HashMap::new();

    let mut patient = serde_json::Map::new();
    patient.insert(
        "weight".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(70.0).unwrap()),
    );
    patient.insert(
        "height".to_string(),
        JsonValue::Number(serde_json::Number::from_f64(1.75).unwrap()),
    );

    input_values.insert("patient".to_string(), JsonValue::Object(patient));

    // Execute the transform
    let result =
        TransformExecutor::execute_transform_with_expr(&transform, input_values).unwrap();

    // Check the result (BMI = 70 / (1.75^2) = 70 / 3.0625 = 22.857)
    match result {
        JsonValue::Number(n) => {
            let value = n.as_f64().unwrap();
            assert!((value - 22.857).abs() < 0.001);
        }
        _ => panic!("Expected number"),
    }
}

#[test]
fn test_execute_transform_with_provider_inputs_handling() {
    let parser = TransformParser::new();
    let expr = parser.parse_expression("a + b").unwrap();
    let base_transform = Transform::new_with_expr("a + b".to_string(), expr, "test.out".to_string());

    // Case 1: explicit inputs provided, dependency analysis should not run
    let mut transform = base_transform.clone();
    transform.set_inputs(vec!["a".to_string()]);

    let provider = |name: &str| -> Result<JsonValue, Box<dyn std::error::Error>> {
        match name {
            "a" => Ok(JsonValue::from(2)),
            other => panic!("unexpected input request: {}", other),
        }
    };
    // Evaluation should fail because 'b' is missing but provider should not panic
    assert!(TransformExecutor::execute_transform_with_provider(&transform, provider).is_err());

    // Case 2: no explicit inputs, analysis should request both 'a' and 'b'
    let provider = |name: &str| -> Result<JsonValue, Box<dyn std::error::Error>> {
        match name {
            "a" => Ok(JsonValue::from(2)),
            "b" => Ok(JsonValue::from(3)),
            other => panic!("unexpected input request: {}", other),
        }
    };

    let result = TransformExecutor::execute_transform_with_provider(&base_transform, provider).unwrap();
    assert_eq!(result, JsonValue::from(5.0));
}

#[test]
fn test_validate_transform() {
    // Valid transform
    let transform = Transform::new("input + 10".to_string(), "test.output".to_string());

    assert!(TransformExecutor::validate_transform(&transform).is_ok());

    // Invalid transform (syntax error)
    let invalid_transform = Transform::new("input +".to_string(), "test.output".to_string());

    assert!(TransformExecutor::validate_transform(&invalid_transform).is_err());

    // No signature validation errors expected anymore
}
