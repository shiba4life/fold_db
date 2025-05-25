use fold_node::schema::types::Transform;
use fold_node::transform::ast::{Expression, Operator, TransformDeclaration};

#[test]
    fn test_transform_from_declaration() {
        let declaration = TransformDeclaration {
            name: "test_transform".to_string(),
            logic: vec![
                Expression::Return(Box::new(Expression::BinaryOp {
                    left: Box::new(Expression::Variable("field1".to_string())),
                    operator: Operator::Add,
                    right: Box::new(Expression::Variable("field2".to_string())),
                })),
            ],
            reversible: false,
            signature: None,
        };

        let transform = Transform::from_declaration(declaration);

        assert_eq!(transform.logic, "return (field1 + field2)"); // Removed trailing semicolon
        assert_eq!(transform.output, "test.test_transform"); // Output derived from declaration name
        assert!(transform.parsed_expression.is_none());
    }

    #[test]
    fn test_output_field() {
        let transform = Transform::new(
            "return x + 1".to_string(),
            "test.number".to_string(),
        );

        assert_eq!(transform.get_output(), "test.number");
    }
