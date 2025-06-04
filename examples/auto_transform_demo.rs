//! Automatic Transform Generation Demo
//!
//! This example demonstrates how to automatically generate and parse transforms.

use fold_node::transform::{Interpreter, TransformParser, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Automatic Transform Generation Demo");
    eprintln!("=================================\n");

    // Create a parser
    let parser = TransformParser::new();

    // Generate a transform automatically based on input/output types
    let auto_transform = generate_transform(
        "temperature_converter",
        "celsius_to_fahrenheit",
        "input.celsius * 9/5 + 32",
    );

    eprintln!("Generated Transform:\n{}", auto_transform);

    // Parse and test the generated transform
    match parser.parse_transform(&auto_transform) {
        Ok(transform) => {
            eprintln!("Transform parsed successfully!");
            eprintln!("Name: {}", transform.name);

            // Create test input
            let mut variables = HashMap::new();
            let mut input = HashMap::new();
            input.insert("celsius".to_string(), serde_json::json!(25.0)); // 25°C
            variables.insert("input".to_string(), Value::Object(input));

            // Evaluate the transform
            let mut interpreter = Interpreter::with_variables(variables);
            for (i, stmt) in transform.logic.iter().enumerate() {
                eprintln!("Evaluating statement {}: {:?}", i + 1, stmt);
                match interpreter.evaluate(stmt) {
                    Ok(result) => eprintln!("Result (should be 77°F): {:?}", result),
                    Err(e) => eprintln!("Evaluation error: {:?}", e),
                }
            }
        }
        Err(e) => eprintln!("Parse error: {:?}", e),
    }

    Ok(())
}

fn generate_transform(name: &str, output_name: &str, logic_expr: &str) -> String {
    format!(
        r#"
    transform {} {{
        output: "{}"
        reversible: false
        logic: {{
            {};
        }}
    }}
    "#,
        name, output_name, logic_expr
    )
}
