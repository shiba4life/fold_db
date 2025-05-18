use fold_node::transform::{parser::TransformParser, ast::{Expression, Operator, Value}};

#[test]
fn parse_simple_arithmetic() {
    let parser = TransformParser::new();
    let expr = parser.parse_expression("1 + 1").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Literal(Value::Number(1.0))),
        operator: Operator::Add,
        right: Box::new(Expression::Literal(Value::Number(1.0))),
    });
}

#[test]
fn parse_variable_expression() {
    let parser = TransformParser::new();
    let expr = parser.parse_expression("input_field + 1").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Variable("input_field".to_string())),
        operator: Operator::Add,
        right: Box::new(Expression::Literal(Value::Number(1.0))),
    });
}

#[test]
fn parse_cross_schema_reference() {
    let parser = TransformParser::new();
    let expr = parser.parse_expression("SchemaA.a_test_field + 5").unwrap();
    let expected = Expression::BinaryOp {
        left: Box::new(Expression::FieldAccess {
            object: Box::new(Expression::Variable("SchemaA".to_string())),
            field: "a_test_field".to_string(),
        }),
        operator: Operator::Add,
        right: Box::new(Expression::Literal(Value::Number(5.0))),
    };
    assert_eq!(expr, expected);
}

#[test]
fn parse_invalid_expression_returns_error() {
    let parser = TransformParser::new();
    assert!(parser.parse_expression("1 +").is_err());
    assert!(parser.parse_expression("foo(").is_err());
}
