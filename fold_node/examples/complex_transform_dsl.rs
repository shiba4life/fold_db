//! Complex Transform DSL Example
//!
//! This example demonstrates a more complex transform DSL with multiple expressions,
//! including let statements and return statements.

use fold_node::transform::{TransformParser, Interpreter, Value};
use std::collections::HashMap;
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Complex Transform DSL Example");
    println!("============================\n");

    // Create a parser
    let parser = TransformParser::new();
    println!("Parser created");

    // Complex transform with multiple expressions
    let complex_transform = r#"
    transform calculate_bmi {
      output: "bmi"
      reversible: false
      signature: sha256sum("v1.0.0")
      
      logic: {
        let height_m = input.height / 100;  // Convert cm to m
        let bmi = input.weight / (height_m ^ 2);
        return bmi;
      }
    }
    "#;
    
    println!("\nComplex transform with let and return statements:\n{}", complex_transform);
    
    match parser.parse_transform(complex_transform) {
        Ok(transform) => {
            println!("Transform parsed successfully!");
            println!("Transform name: {}", transform.name);
            println!("Output name: {}", transform.output_name);
            println!("Reversible: {}", transform.reversible);
            if let Some(sig) = &transform.signature {
                println!("Signature: {}", sig);
            }
            println!("Logic statements: {}", transform.logic.len());
            
            // Print each statement
            for (i, stmt) in transform.logic.iter().enumerate() {
                println!("Statement {}: {:?}", i+1, stmt);
            }
            
            // Create input object for evaluation
            let mut variables = HashMap::new();
            let mut input = HashMap::new();
            input.insert("height".to_string(), json!(175));  // 175 cm
            input.insert("weight".to_string(), json!(70));   // 70 kg
            variables.insert("input".to_string(), Value::Object(input));
            
            // Try to evaluate each logic statement
            let mut interpreter = Interpreter::with_variables(variables);
            for (i, stmt) in transform.logic.iter().enumerate() {
                println!("\nEvaluating statement {}: {:?}", i+1, stmt);
                match interpreter.evaluate(stmt) {
                    Ok(result) => println!("  Result: {:?}", result),
                    Err(e) => println!("  Evaluation error: {:?}", e),
                }
            }
        },
        Err(e) => println!("Parse error: {:?}", e),
    }
    
    // Risk score transform with more complex logic
    let risk_score_transform = r#"
    transform calculate_risk_score {
      output: "risk_score"
      reversible: false
      signature: sha256sum("v1.0.1")
      
      logic: {
        let base_score = 50;
        let age_factor = input.age / 10;
        let bmi_factor = (input.bmi - 25) * 2;
        let bp_factor = (input.systolic - 120) / 10;
        let risk = base_score + age_factor + bmi_factor + bp_factor;
        return clamp(risk, 0, 100);
      }
    }
    "#;
    
    println!("\nRisk score transform with complex logic:\n{}", risk_score_transform);
    
    match parser.parse_transform(risk_score_transform) {
        Ok(transform) => {
            println!("Transform parsed successfully!");
            println!("Transform name: {}", transform.name);
            println!("Output name: {}", transform.output_name);
            println!("Logic statements: {}", transform.logic.len());
            
            // Create input object for evaluation
            let mut variables = HashMap::new();
            let mut input = HashMap::new();
            input.insert("age".to_string(), json!(45));
            input.insert("bmi".to_string(), json!(28));
            input.insert("systolic".to_string(), json!(130));
            variables.insert("input".to_string(), Value::Object(input));
            
            // Try to evaluate each logic statement
            let mut interpreter = Interpreter::with_variables(variables);
            for (i, stmt) in transform.logic.iter().enumerate() {
                println!("\nEvaluating statement {}: {:?}", i+1, stmt);
                match interpreter.evaluate(stmt) {
                    Ok(result) => println!("  Result: {:?}", result),
                    Err(e) => println!("  Evaluation error: {:?}", e),
                }
            }
        },
        Err(e) => println!("Parse error: {:?}", e),
    }
    
    Ok(())
}