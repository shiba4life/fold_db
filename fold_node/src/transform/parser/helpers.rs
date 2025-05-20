use super::grammar::{TransformParser, Rule};
use crate::transform::ast::{Expression, TransformDeclaration, Value};
use crate::schema::types::SchemaError;
use pest::iterators::Pair;

impl TransformParser {
    pub(super) fn parse_field_access(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
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
    pub(super) fn parse_function_call(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
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
        /// Parses a statement.
        #[allow(dead_code)]
    pub(super) fn parse_stmt(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
            let inner = pair.into_inner().next().unwrap();
            
            match inner.as_rule() {
                Rule::let_stmt => self.parse_let_stmt(inner),
                Rule::return_stmt => self.parse_return_stmt(inner),
                Rule::if_stmt => self.parse_if_stmt(inner),
                Rule::expr_stmt => self.parse_expr_stmt(inner),
                _ => Err(SchemaError::InvalidField(format!("Unexpected rule in statement: {:?}", inner.as_rule()))),
            }
        }
        
        /// Parses an if statement.
        #[allow(dead_code)]
    pub(super) fn parse_if_stmt(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the condition
            let condition_pair = pairs.next().unwrap();
            let condition = self.build_ast(condition_pair)?;
            
            // Get the then branch statements
            let mut then_stmts = Vec::new();
            
            // Parse each statement in the then block
            for stmt_pair in pairs.by_ref() {
                if stmt_pair.as_rule() == Rule::stmt {
                    then_stmts.push(self.parse_stmt(stmt_pair)?);
                } else {
                    // We've reached the else branch or the end
                    break;
                }
            }
            
            // Create a sequence of expressions for the then branch
            let then_expr = if then_stmts.is_empty() {
                Expression::Literal(Value::Null)
            } else {
                let mut result = then_stmts.remove(0);
                for stmt in then_stmts {
                    // For simplicity, we'll just use a let binding with a dummy variable
                    // to sequence the expressions
                    result = Expression::LetBinding {
                        name: "_".to_string(),
                        value: Box::new(result),
                        body: Box::new(stmt),
                    };
                }
                result
            };
            
            // Check if there's an else branch
            let else_expr = if pairs.peek().is_some() {
                let mut else_stmts = Vec::new();
                
                // Parse each statement in the else block
                for stmt_pair in pairs {
                    if stmt_pair.as_rule() == Rule::stmt {
                        else_stmts.push(self.parse_stmt(stmt_pair)?);
                    } else {
                        break;
                    }
                }
                
                // Create a sequence of expressions for the else branch
                if !else_stmts.is_empty() {
                    let mut result = else_stmts.remove(0);
                    for stmt in else_stmts {
                        // For simplicity, we'll just use a let binding with a dummy variable
                        // to sequence the expressions
                        result = Expression::LetBinding {
                            name: "_".to_string(),
                            value: Box::new(result),
                            body: Box::new(stmt),
                        };
                    }
                    Some(Box::new(result))
                } else {
                    None
                }
            } else {
                None
            };
            
            // Create the if-else expression
            Ok(Expression::IfElse {
                condition: Box::new(condition),
                then_branch: Box::new(then_expr),
                else_branch: else_expr,
            })
        }
        
        /// Parses a let statement.
        #[allow(dead_code)]
    pub(super) fn parse_let_stmt(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the variable name
            let name = pairs.next().unwrap().as_str().to_string();
            
            // Get the value expression
            let value_pair = pairs.next().unwrap();
            let value = self.build_ast(value_pair)?;
            
            // Create a let binding with a dummy body (will be replaced later)
            Ok(Expression::LetBinding {
                name,
                value: Box::new(value),
                body: Box::new(Expression::Literal(Value::Null)),
            })
        }
        
        /// Parses a return statement.
        #[allow(dead_code)]
    pub(super) fn parse_return_stmt(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the return expression
            let expr_pair = pairs.next().unwrap();
            let expr = self.build_ast(expr_pair)?;
            
            Ok(Expression::Return(Box::new(expr)))
        }
        
        /// Parses an expression statement.
        #[allow(dead_code)]
    pub(super) fn parse_expr_stmt(&self, pair: Pair<Rule>) -> Result<Expression, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the expression
            let expr_pair = pairs.next().unwrap();
            self.build_ast(expr_pair)
        }
        
        /// Builds a TransformDeclaration from a parse tree.
    pub(super) fn build_transform_decl(&self, pair: Pair<Rule>) -> Result<TransformDeclaration, SchemaError> {
            match pair.as_rule() {
                Rule::complete_transform => {
                    // Get the transform_decl inside the complete_transform
                    let decl_pair = pair.into_inner().next().unwrap();
                    self.build_transform_decl(decl_pair)
                },
                Rule::transform_decl => {
                    let mut pairs = pair.into_inner();
                    
                    // Get the transform name
                    let name = pairs.next().unwrap().as_str().to_string();
                    
                    let mut reversible = false;
                    let mut signature = None;
                    let mut logic = Vec::new();
                    
                    // Parse the transform components
                    for pair in pairs {
                        match pair.as_rule() {
                            Rule::reversible_decl => {
                                reversible = self.parse_reversible_decl(pair)?;
                            },
                            Rule::signature_decl => {
                                signature = Some(self.parse_signature_decl(pair)?);
                            },
                            Rule::logic_decl => {
                                logic = self.parse_logic_decl(pair)?;
                            },
                            _ => return Err(SchemaError::InvalidField(format!("Unexpected rule in transform declaration: {:?}", pair.as_rule()))),
                        }
                    }
                    
                    // Create the transform declaration
                    Ok(TransformDeclaration {
                        name,
                        reversible,
                        signature,
                        logic,
                    })
                },
                _ => Err(SchemaError::InvalidField(format!("Unexpected rule: {:?}", pair.as_rule()))),
            }
        }
        
        /// Parses a reversible declaration.
    pub(super) fn parse_reversible_decl(&self, pair: Pair<Rule>) -> Result<bool, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the boolean value
            let bool_pair = pairs.next().unwrap();
            match bool_pair.as_str() {
                "true" => Ok(true),
                "false" => Ok(false),
                _ => Err(SchemaError::InvalidField(format!("Invalid boolean: {}", bool_pair.as_str()))),
            }
        }
        
        /// Parses a signature declaration.
    pub(super) fn parse_signature_decl(&self, pair: Pair<Rule>) -> Result<String, SchemaError> {
            let mut pairs = pair.into_inner();
            
            // Get the signature expression
            let sig_expr_pair = pairs.next().unwrap();
            
            match sig_expr_pair.as_rule() {
                Rule::string => self.parse_string_literal(sig_expr_pair),
                Rule::function_call => {
                    // For function calls like sha256sum("v1.0.3"), just return the raw text
                    Ok(sig_expr_pair.as_str().to_string())
                },
                _ => Err(SchemaError::InvalidField(format!("Invalid signature: {:?}", sig_expr_pair.as_rule()))),
            }
        }
        
        /// Parses a logic declaration.
    pub(super) fn parse_logic_decl(&self, pair: Pair<Rule>) -> Result<Vec<Expression>, SchemaError> {
            let pairs = pair.into_inner();
            let mut exprs = Vec::new();
            
            // Parse each statement in the logic block
            for stmt_pair in pairs {
                if stmt_pair.as_rule() == Rule::stmt {
                    let inner_pair = stmt_pair.into_inner().next().unwrap();
                    
                    match inner_pair.as_rule() {
                        Rule::let_stmt => {
                            // Parse let statement
                            let mut inner_pairs = inner_pair.into_inner();
                            let name = inner_pairs.next().unwrap().as_str().to_string();
                            let expr_pair = inner_pairs.next().unwrap();
                            let expr = self.build_ast(expr_pair)?;
                            
                            exprs.push(Expression::LetBinding {
                                name,
                                value: Box::new(expr),
                                body: Box::new(Expression::Literal(Value::Null)), // Placeholder
                            });
                        },
                        Rule::return_stmt => {
                            // Parse return statement
                            let expr_pair = inner_pair.into_inner().next().unwrap();
                            let expr = self.build_ast(expr_pair)?;
                            
                            exprs.push(Expression::Return(Box::new(expr)));
                        },
                        Rule::expr_stmt => {
                            // Parse expression statement
                            let expr_pair = inner_pair.into_inner().next().unwrap();
                            let expr = self.build_ast(expr_pair)?;
                            exprs.push(expr);
                        },
                        _ => {
                            // Skip other rules
                        }
                    }
                }
            }
            
            Ok(exprs)
        }    
        
        /// Parses a string literal.
    pub(super) fn parse_string_literal(&self, pair: Pair<Rule>) -> Result<String, SchemaError> {
            // Remove the surrounding quotes
            let s = pair.as_str();
            let s = &s[1..s.len()-1];
            Ok(s.to_string())
        }
    }

