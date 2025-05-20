use fold_node::transform::{ast::{Expression, Value}, Interpreter};

#[test]
fn evaluate_min_function() {
    let expr = Expression::FunctionCall {
        name: "min".to_string(),
        args: vec![
            Expression::Literal(Value::Number(5.0)),
            Expression::Literal(Value::Number(3.0)),
        ],
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::Number(3.0));
}

#[test]
fn evaluate_concat_function() {
    let expr = Expression::FunctionCall {
        name: "concat".to_string(),
        args: vec![
            Expression::Literal(Value::String("Hello".into())),
            Expression::Literal(Value::String("World".into())),
        ],
    };

    let mut interpreter = Interpreter::new();
    let result = interpreter.evaluate(&expr).unwrap();
    assert_eq!(result, Value::String("HelloWorld".into()));
}
