//! Test for transform logic parsing
//!
//! This example focuses on testing the parsing of the logic block in transform declarations.

use fold_node::schema::transform::{TransformParser, Interpreter, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Transform Logic Block Test");
    println!("========================\n");

    // Create a parser
    let parser = TransformParser::new();
    println!("Parser created");

    // Test expression parsing
    let expr_str = "input.value * 2";
    println!("\nTesting expression parsing: '{}'", expr_str);
    match parser.parse_expression(expr_str) {
        Ok(expr) => {
            println!("Expression parsed successfully!");
            println!("AST: {:?}", expr);
        },
        Err(e) => println!("Expression parse error: {:?}", e),
    }

    // Test transform with simple logic
    let transform_str = r#"
    transform simple_transform {
      output: Field<Float> as "result"
      logic: { input.value * 2; }
    }
    "#;
    
    println!("\nTesting transform with logic: \n{}", transform_str);
    match parser.parse_transform(transform_str) {
        Ok(transform) => {
            println!("Transform parsed successfully!");
            println!("Transform name: {}", transform.name);
            println!("Output name: {}", transform.output_name);
            println!("Logic statements: {}", transform.logic.len());
            
            for (i, stmt) in transform.logic.iter().enumerate() {
                println!("Statement {}: {:?}", i+1, stmt);
            }
        },
        Err(e) => println!("Transform parse error: {:?}", e),
    }

    // Let's look at the parser's handling of expressions in the logic block
    println!("\nLet's debug the logic block parsing:");
    let logic_block = "{ input.value * 2; }";
    println!("Logic block: {}", logic_block);
    
    // Create a minimal transform to test
    let minimal_transform = format!(r#"
    transform test {{
      output: Field<Float> as "test"
      logic: {}
    }}
    "#, logic_block);
    
    println!("\nMinimal transform:\n{}", minimal_transform);
    
    match parser.parse_transform(&minimal_transform) {
        Ok(transform) => {
            println!("Transform parsed successfully!");
            println!("Logic statements: {}", transform.logic.len());
            for (i, stmt) in transform.logic.iter().enumerate() {
                println!("Statement {}: {:?}", i+1, stmt);
            }
        },
        Err(e) => println!("Transform parse error: {:?}", e),
    }

    // Let's look at the parser implementation for logic_decl
    println!("\nChecking the parser implementation for logic_decl:");
    println!("According to the grammar, logic_decl = {{ \"logic\" ~ \":\" ~ \"{{\" ~ (expr ~ \";\")* ~ \"}}\" }}");
    println!("But the parser implementation only handles let_stmt and return_stmt, not regular expressions.");
    
    Ok(())
}