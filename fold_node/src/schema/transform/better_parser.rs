//! Better parser for the transform DSL.
//!
//! This module implements a cleaner, more robust parser for the transform DSL using PEST.
//! It converts a string of DSL code into an Abstract Syntax Tree (AST).

use super::ast::{Expression, Operator, UnaryOperator, Value};
use crate::schema::types::SchemaError;
use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

/// Parser for the transform DSL.
#[derive(Parser)]
#[grammar = "src/schema/transform/better_transform.pest"]
pub struct BetterParser;

impl BetterParser {
    /// Creates a new parser.
    pub fn new() -> Self {
        Self
    }
    
    /// Parses the input into an AST.
    pub fn parse_expression(&self, input: &str) -> Result<Expression, SchemaError> {
        // Parse the input using the complete_expr rule
        let pairs = Self::parse(Rule::complete_expr, input)
            .map_err(|e| SchemaError::InvalidField(format!("Parse error: {}", e)))?;
        
        // Get the expression from the parse result
        let expr_pair = pairs.into_iter().next().unwrap();
        
        // Convert the parse tree to an AST
        self.build_ast(expr_pair)
    }
    
    /// Builds an AST from a parse tree.
    fn build_ast(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        match pair.as_rule() {
            Rule::complete_expr => {
                // Get the expression inside the complete_expr
                let expr_pair = pair.into_inner().next().unwrap();
                self.build_ast(expr_pair)
            },
            Rule::expr => {
                // Get the expression inside the expr
                let inner_pair = pair.into_inner().next().unwrap();
                self.build_ast(inner_pair)
            },
            Rule::logic_expr => self.parse_logic_expr(pair),
            Rule::comp_expr => self.parse_comp_expr(pair),
            Rule::add_expr => self.parse_add_expr(pair),
            Rule::mul_expr => self.parse_mul_expr(pair),
            Rule::pow_expr => self.parse_pow_expr(pair),
            Rule::unary_expr => self.parse_unary_expr(pair),
            Rule::atom => self.parse_atom(pair),
            _ => Err(SchemaError::InvalidField(format!("Unexpected rule: {:?}", pair.as_rule()))),
        }
    }
    
    /// Parses a logic expression (&&, ||).
    fn parse_logic_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Parse the first comparison expression
        let first = pairs.next().unwrap();
        let mut expr = self.build_ast(first)?;
        
        // Parse any additional logic operations
        while let Some(op_pair) = pairs.next() {
            if op_pair.as_rule() == Rule::logic_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "&&" => Operator::And,
                    "||" => Operator::Or,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown logic operator: {}", op_str))),
                };
                
                let right_pair = pairs.next().unwrap();
                let right = self.build_ast(right_pair)?;
                
                expr = Expression::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// Parses a comparison expression (==, !=, <, <=, >, >=).
    fn parse_comp_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Parse the first addition expression
        let first = pairs.next().unwrap();
        let mut expr = self.build_ast(first)?;
        
        // Parse any additional comparison operations
        while let Some(op_pair) = pairs.next() {
            if op_pair.as_rule() == Rule::comp_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "==" => Operator::Equal,
                    "!=" => Operator::NotEqual,
                    "<" => Operator::LessThan,
                    "<=" => Operator::LessThanOrEqual,
                    ">" => Operator::GreaterThan,
                    ">=" => Operator::GreaterThanOrEqual,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown comparison operator: {}", op_str))),
                };
                
                let right_pair = pairs.next().unwrap();
                let right = self.build_ast(right_pair)?;
                
                expr = Expression::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// Parses an addition expression (+, -).
    fn parse_add_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Parse the first multiplication expression
        let first = pairs.next().unwrap();
        let mut expr = self.build_ast(first)?;
        
        // Parse any additional addition operations
        while let Some(op_pair) = pairs.next() {
            if op_pair.as_rule() == Rule::add_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "+" => Operator::Add,
                    "-" => Operator::Subtract,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown addition operator: {}", op_str))),
                };
                
                let right_pair = pairs.next().unwrap();
                let right = self.build_ast(right_pair)?;
                
                expr = Expression::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// Parses a multiplication expression (*, /).
    fn parse_mul_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Parse the first power expression
        let first = pairs.next().unwrap();
        let mut expr = self.build_ast(first)?;
        
        // Parse any additional multiplication operations
        while let Some(op_pair) = pairs.next() {
            if op_pair.as_rule() == Rule::mul_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "*" => Operator::Multiply,
                    "/" => Operator::Divide,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown multiplication operator: {}", op_str))),
                };
                
                let right_pair = pairs.next().unwrap();
                let right = self.build_ast(right_pair)?;
                
                expr = Expression::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// Parses a power expression (^).
    fn parse_pow_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Parse the first unary expression
        let first = pairs.next().unwrap();
        let mut expr = self.build_ast(first)?;
        
        // Parse any additional power operations
        while let Some(op_pair) = pairs.next() {
            if op_pair.as_rule() == Rule::pow_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "^" => Operator::Power,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown power operator: {}", op_str))),
                };
                
                let right_pair = pairs.next().unwrap();
                let right = self.build_ast(right_pair)?;
                
                expr = Expression::BinaryOp {
                    left: Box::new(expr),
                    operator: op,
                    right: Box::new(right),
                };
            }
        }
        
        Ok(expr)
    }
    
    /// Parses a unary expression (-, !).
    fn parse_unary_expr(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Collect all unary operators
        let mut unary_ops = Vec::new();
        while let Some(op_pair) = pairs.peek() {
            if op_pair.as_rule() == Rule::unary_op {
                let op_str = op_pair.as_str();
                let op = match op_str {
                    "-" => UnaryOperator::Negate,
                    "!" => UnaryOperator::Not,
                    _ => return Err(SchemaError::InvalidField(format!("Unknown unary operator: {}", op_str))),
                };
                unary_ops.push(op);
                pairs.next(); // Consume the operator
            } else {
                break;
            }
        }
        
        // Parse the atom
        let atom_pair = pairs.next().unwrap();
        let mut expr = self.build_ast(atom_pair)?;
        
        // Apply unary operators in reverse order (right to left)
        for op in unary_ops.into_iter().rev() {
            expr = Expression::UnaryOp {
                operator: op,
                expr: Box::new(expr),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses an atom (number, string, boolean, null, function call, field access, identifier, or parenthesized expression).
    fn parse_atom(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let inner = pair.into_inner().next().unwrap();
        
        match inner.as_rule() {
            Rule::number => {
                let n = inner.as_str().parse::<f64>()
                    .map_err(|e| SchemaError::InvalidField(format!("Invalid number: {}", e)))?;
                Ok(Expression::Literal(Value::Number(n)))
            },
            Rule::string => {
                // Remove the surrounding quotes
                let s = inner.as_str();
                let s = &s[1..s.len()-1];
                Ok(Expression::Literal(Value::String(s.to_string())))
            },
            Rule::boolean => {
                match inner.as_str() {
                    "true" => Ok(Expression::Literal(Value::Boolean(true))),
                    "false" => Ok(Expression::Literal(Value::Boolean(false))),
                    _ => Err(SchemaError::InvalidField(format!("Invalid boolean: {}", inner.as_str()))),
                }
            },
            Rule::null => {
                Ok(Expression::Literal(Value::Null))
            },
            Rule::function_call => {
                self.parse_function_call(inner)
            },
            Rule::field_access => {
                self.parse_field_access(inner)
            },
            Rule::identifier => {
                Ok(Expression::Variable(inner.as_str().to_string()))
            },
            Rule::expr => {
                self.build_ast(inner)
            },
            _ => Err(SchemaError::InvalidField(format!("Unexpected rule in atom: {:?}", inner.as_rule()))),
        }
    }
    
    /// Parses a field access expression (obj.field).
    fn parse_field_access(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Get the object identifier
        let obj_ident = pairs.next().unwrap();
        let obj_expr = Expression::Variable(obj_ident.as_str().to_string());
        
        // Get the field identifiers
        let mut expr = obj_expr;
        for field_pair in pairs {
            expr = Expression::FieldAccess {
                object: Box::new(expr),
                field: field_pair.as_str().to_string(),
            };
        }
        
        Ok(expr)
    }
    
    /// Parses a function call expression (func(arg1, arg2, ...)).
    fn parse_function_call(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
        let mut pairs = pair.into_inner();
        
        // Get the function name
        let func_name = pairs.next().unwrap().as_str().to_string();
        
        // Get the arguments
        let mut args = Vec::new();
        for arg_pair in pairs {
            args.push(self.build_ast(arg_pair)?);
        }
        
        Ok(Expression::FunctionCall {
            name: func_name,
            args,
        })
    }
}

impl Default for BetterParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_simple_arithmetic() {
        let parser = BetterParser::new();
        
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
        let parser = BetterParser::new();
        
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
        let parser = BetterParser::new();
        
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
        let parser = BetterParser::new();
        
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
        let parser = BetterParser::new();
        
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
        let parser = BetterParser::new();
        
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
}