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

