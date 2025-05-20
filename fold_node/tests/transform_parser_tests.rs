use fold_node::transform::{parser::TransformParser, ast::{Expression, Operator, UnaryOperator, Value}};

#[test]
fn test_parse_simple_arithmetic() {
    let parser = TransformParser::new();
    
    // Test basic arithmetic
    let expr = parser.parse_expression("2 + 3").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Literal(Value::Number(2.0))),
        operator: Operator::Add,
        right: Box::new(Expression::Literal(Value::Number(3.0))),
    });
    
    // Test operator precedence
    let expr = parser.parse_expression("2 + 3 * 4").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Literal(Value::Number(2.0))),
        operator: Operator::Add,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Number(3.0))),
            operator: Operator::Multiply,
            right: Box::new(Expression::Literal(Value::Number(4.0))),
        }),
    });
    
    // Test parentheses
    let expr = parser.parse_expression("(2 + 3) * 4").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Literal(Value::Number(2.0))),
            operator: Operator::Add,
            right: Box::new(Expression::Literal(Value::Number(3.0))),
        }),
        operator: Operator::Multiply,
        right: Box::new(Expression::Literal(Value::Number(4.0))),
    });
}

#[test]
fn test_parse_variables() {
    let parser = TransformParser::new();
    
    // Test variable reference
    let expr = parser.parse_expression("x").unwrap();
    assert_eq!(expr, Expression::Variable("x".to_string()));
    
    // Test variable in expression
    let expr = parser.parse_expression("x + y").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Variable("x".to_string())),
        operator: Operator::Add,
        right: Box::new(Expression::Variable("y".to_string())),
    });
}

#[test]
fn test_parse_field_access() {
    let parser = TransformParser::new();
    
    // Test field access
    let expr = parser.parse_expression("obj.field").unwrap();
    assert_eq!(expr, Expression::FieldAccess {
        object: Box::new(Expression::Variable("obj".to_string())),
        field: "field".to_string(),
    });
    
    // Test nested field access
    let expr = parser.parse_expression("obj.field1.field2").unwrap();
    assert_eq!(expr, Expression::FieldAccess {
        object: Box::new(Expression::FieldAccess {
            object: Box::new(Expression::Variable("obj".to_string())),
            field: "field1".to_string(),
        }),
        field: "field2".to_string(),
    });
}

#[test]
fn test_parse_function_calls() {
    let parser = TransformParser::new();
    
    // Test function call with no arguments
    let expr = parser.parse_expression("func()").unwrap();
    assert_eq!(expr, Expression::FunctionCall {
        name: "func".to_string(),
        args: vec![],
    });
    
    // Test function call with arguments
    let expr = parser.parse_expression("add(1, 2)").unwrap();
    assert_eq!(expr, Expression::FunctionCall {
        name: "add".to_string(),
        args: vec![
            Expression::Literal(Value::Number(1.0)),
            Expression::Literal(Value::Number(2.0)),
        ],
    });
    
    // Test function call with complex arguments
    let expr = parser.parse_expression("max(x + y, z * 2)").unwrap();
    assert_eq!(expr, Expression::FunctionCall {
        name: "max".to_string(),
        args: vec![
            Expression::BinaryOp {
                left: Box::new(Expression::Variable("x".to_string())),
                operator: Operator::Add,
                right: Box::new(Expression::Variable("y".to_string())),
            },
            Expression::BinaryOp {
                left: Box::new(Expression::Variable("z".to_string())),
                operator: Operator::Multiply,
                right: Box::new(Expression::Literal(Value::Number(2.0))),
            },
        ],
    });
}

#[test]
fn test_parse_comparison_and_logic() {
    let parser = TransformParser::new();
    
    // Test comparison
    let expr = parser.parse_expression("x > 5").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::Variable("x".to_string())),
        operator: Operator::GreaterThan,
        right: Box::new(Expression::Literal(Value::Number(5.0))),
    });
    
    // Test logical operators
    let expr = parser.parse_expression("x > 5 && y < 10").unwrap();
    assert_eq!(expr, Expression::BinaryOp {
        left: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Variable("x".to_string())),
            operator: Operator::GreaterThan,
            right: Box::new(Expression::Literal(Value::Number(5.0))),
        }),
        operator: Operator::And,
        right: Box::new(Expression::BinaryOp {
            left: Box::new(Expression::Variable("y".to_string())),
            operator: Operator::LessThan,
            right: Box::new(Expression::Literal(Value::Number(10.0))),
        }),
    });
}

#[test]
fn test_parse_unary_operators() {
    let parser = TransformParser::new();
    
    // Test negation
    let expr = parser.parse_expression("-x").unwrap();
    assert_eq!(expr, Expression::UnaryOp {
        operator: UnaryOperator::Negate,
        expr: Box::new(Expression::Variable("x".to_string())),
    });
    
    // Test logical not
    let expr = parser.parse_expression("!x").unwrap();
    assert_eq!(expr, Expression::UnaryOp {
        operator: UnaryOperator::Not,
        expr: Box::new(Expression::Variable("x".to_string())),
    });
    
    // Test multiple unary operators
    let expr = parser.parse_expression("!-x").unwrap();
    assert_eq!(expr, Expression::UnaryOp {
        operator: UnaryOperator::Not,
        expr: Box::new(Expression::UnaryOp {
            operator: UnaryOperator::Negate,
            expr: Box::new(Expression::Variable("x".to_string())),
        }),
    });
}

#[test]
fn test_parse_transform_declaration() {
    let parser = TransformParser::new();
    
    let transform_code = r#"
    transform my_transform {
      logic: {
        return field1 + field2;
      }
    }
    "#;

    let decl = parser.parse_transform(transform_code).expect("Failed to parse transform declaration");

    // Verify the parsed declaration
    assert_eq!(decl.name, "my_transform");
    assert_eq!(decl.logic.len(), 1);
    match &decl.logic[0] {
        Expression::Return(expr) => {
            match &**expr {
                Expression::BinaryOp { left, operator, right } => {
                    assert_eq!(*operator, Operator::Add); // Dereference operator
                    match &**left {
                        Expression::Variable(name) => assert_eq!(name, "field1"),
                        _ => panic!("Expected Variable for left side"),
                    }
                    match &**right {
                        Expression::Variable(name) => assert_eq!(name, "field2"),
                        _ => panic!("Expected Variable for right side"),
                    }
                },
                _ => panic!("Expected BinaryOp inside Return"),
            }
        },
        _ => panic!("Expected Return expression"),
    }
}

