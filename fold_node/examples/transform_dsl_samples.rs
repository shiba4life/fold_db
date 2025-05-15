//! Sample DSLs for the transform module
//!
//! This example demonstrates various transform DSL samples and parses them.

use fold_node::schema::transform::{TransformParser, Interpreter, Value};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    eprintln!("Transform DSL Samples");
    eprintln!("====================\n");

    // Create a parser
    let parser = TransformParser::new();
    eprintln!("Parser created");

    // Sample 1: Simple arithmetic expression
    let sample1 = "2 + 3 * 4";
    test_expression(1, "Simple Arithmetic", sample1, &parser);

    // Sample 2: Expression with variables
    let sample2 = "x + y * z";
    test_expression_with_variables(2, "Variables", sample2, &parser);

    // Sample 3: Function calls
    let sample3 = "min(max(x, y), z + 10)";
    test_expression_with_variables(3, "Function Calls", sample3, &parser);

    // Sample 4: Field access
    let sample4 = "patient.height / (patient.weight ^ 2)";
    test_expression_with_object(4, "Field Access (BMI calculation)", sample4, &parser);

    // Sample 5: Simple transform declaration
    let sample5 = r#"
    transform simple_transform {
      input: Fold<PatientVitals>
      output: Field<Float> as "result"
      reversible: false
      
      logic: {
        input.value * 2;
      }
    }
    "#;
    test_transform(5, "Simple Transform", sample5, &parser);
    
    // Sample 6: Another simple transform
    let sample6 = r#"
    transform add_values {
      input: Fold<Values>
      output: Field<Float> as "sum"
      reversible: false
      
      logic: {
        input.value1 + input.value2;
      }
    }
    "#;
    test_transform(6, "Add Values Transform", sample6, &parser);

    // Let's examine the parse tree for the logic block
    eprintln!("\nExamining logic block parsing:");
    match parser.parse_transform(r#"
    transform test {
      output: Field<Float> as "test"
      logic: { input.value * 2; }
    }
    "#) {
        Ok(transform) => {
            eprintln!("Logic statements count: {}", transform.logic.len());
            for (i, stmt) in transform.logic.iter().enumerate() {
                eprintln!("Statement {}: {:?}", i+1, stmt);
            }
        },
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
    
    // Let's look at the parser's handling of expressions in the logic block
    eprintln!("\nLet's debug the logic block parsing:");
    let expr_str = "input.value * 2";
    match parser.parse_expression(expr_str) {
        Ok(expr) => {
            eprintln!("Expression parsed successfully: {:?}", expr);
            
            // Now let's see if we can create a transform with this expression
            let transform_str = format!(r#"
            transform expr_test {{
              output: Field<Float> as "test"
              logic: {{ {}; }}
            }}
            "#, expr_str);
            
            eprintln!("Transform with expression:\n{}", transform_str);
            
            match parser.parse_transform(&transform_str) {
                Ok(transform) => {
                    eprintln!("Transform parsed successfully!");
                    eprintln!("Logic statements: {}", transform.logic.len());
                    for (i, stmt) in transform.logic.iter().enumerate() {
                        eprintln!("Statement {}: {:?}", i+1, stmt);
                    }
                },
                Err(e) => eprintln!("Transform parse error: {:?}", e),
            }
        },
        Err(e) => eprintln!("Expression parse error: {:?}", e),
    }

    Ok(())
}

fn test_expression(id: usize, name: &str, expr_str: &str, parser: &TransformParser) {
    eprintln!("Sample {}: {}", id, name);
    eprintln!("Expression: {}", expr_str);
    
    match parser.parse_expression(expr_str) {
        Ok(expr) => {
            eprintln!("Parsed successfully!");
            eprintln!("AST: {:?}", expr);
            
            // Evaluate with empty variables
            let mut interpreter = Interpreter::new();
            match interpreter.evaluate(&expr) {
                Ok(result) => eprintln!("Result: {:?}", result),
                Err(e) => eprintln!("Evaluation error: {:?}", e),
            }
        },
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
    eprintln!();
}

fn test_expression_with_variables(id: usize, name: &str, expr_str: &str, parser: &TransformParser) {
    eprintln!("Sample {}: {}", id, name);
    eprintln!("Expression: {}", expr_str);
    
    match parser.parse_expression(expr_str) {
        Ok(expr) => {
            eprintln!("Parsed successfully!");
            eprintln!("AST: {:?}", expr);
            
            // Create variables
            let mut variables = HashMap::new();
            variables.insert("x".to_string(), Value::Number(10.0));
            variables.insert("y".to_string(), Value::Number(5.0));
            variables.insert("z".to_string(), Value::Number(15.0));
            
            // Evaluate with variables
            let mut interpreter = Interpreter::with_variables(variables);
            match interpreter.evaluate(&expr) {
                Ok(result) => eprintln!("Result with x=10, y=5, z=15: {:?}", result),
                Err(e) => eprintln!("Evaluation error: {:?}", e),
            }
        },
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
    eprintln!();
}

fn test_expression_with_object(id: usize, name: &str, expr_str: &str, parser: &TransformParser) {
    eprintln!("Sample {}: {}", id, name);
    eprintln!("Expression: {}", expr_str);
    
    match parser.parse_expression(expr_str) {
        Ok(expr) => {
            eprintln!("Parsed successfully!");
            eprintln!("AST: {:?}", expr);
            
            // Create patient object
            let mut variables = HashMap::new();
            let mut patient = HashMap::new();
            patient.insert("height".to_string(), serde_json::json!(175));  // 175 cm
            patient.insert("weight".to_string(), serde_json::json!(70));   // 70 kg
            patient.insert("age".to_string(), serde_json::json!(35));      // 35 years
            patient.insert("bmi".to_string(), serde_json::json!(22.9));    // pre-calculated BMI
            variables.insert("patient".to_string(), Value::Object(patient));
            
            // Evaluate with patient object
            let mut interpreter = Interpreter::with_variables(variables);
            match interpreter.evaluate(&expr) {
                Ok(result) => eprintln!("Result: {:?}", result),
                Err(e) => eprintln!("Evaluation error: {:?}", e),
            }
        },
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
    eprintln!();
}

fn test_transform(id: usize, name: &str, transform_str: &str, parser: &TransformParser) {
    eprintln!("Sample {}: {}", id, name);
    eprintln!("Transform:\n{}", transform_str);
    
    match parser.parse_transform(transform_str) {
        Ok(transform) => {
            eprintln!("Parsed successfully!");
            eprintln!("Transform name: {}", transform.name);
            eprintln!("Output name: {}", transform.output_name);
            eprintln!("Reversible: {}", transform.reversible);
            if let Some(sig) = transform.signature {
                eprintln!("Signature: {}", sig);
            }
            eprintln!("Logic statements: {}", transform.logic.len());
            
            // Create input object for evaluation
            let mut variables = HashMap::new();
            let mut input = HashMap::new();
            
            // Add appropriate fields based on the transform name
            match transform.name.as_str() {
                "simple_transform" => {
                    input.insert("value".to_string(), serde_json::json!(10));
                },
                "add_values" => {
                    input.insert("value1".to_string(), serde_json::json!(5));
                    input.insert("value2".to_string(), serde_json::json!(7));
                },
                _ => {}
            }
            
            // This is needed to ensure the main function is properly recognized
            #[allow(dead_code)]
            fn dummy() {}
            
            variables.insert("input".to_string(), Value::Object(input));
            
            // Try to evaluate each logic statement
            let mut interpreter = Interpreter::with_variables(variables);
            for (i, stmt) in transform.logic.iter().enumerate() {
                eprintln!("Evaluating statement {}: {:?}", i+1, stmt);
                match interpreter.evaluate(stmt) {
                    Ok(result) => eprintln!("  Result: {:?}", result),
                    Err(e) => eprintln!("  Evaluation error: {:?}", e),
                }
            }
        },
        Err(e) => eprintln!("Parse error: {:?}", e),
    }
    eprintln!();
}