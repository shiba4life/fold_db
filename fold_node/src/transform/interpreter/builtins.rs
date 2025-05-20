use std::collections::HashMap;

use super::super::ast::Value;

/// Type for function implementations in the interpreter
pub type TransformFunction = Box<dyn Fn(Vec<Value>) -> Result<Value, String>>;

/// Returns the default set of built-in functions for the interpreter.
pub fn builtin_functions() -> HashMap<String, TransformFunction> {
    let mut functions: HashMap<String, TransformFunction> = HashMap::new();

    // Math functions
    functions.insert("min".to_string(), Box::new(|args| {
        if args.len() != 2 {
            return Err("min() requires exactly 2 arguments".to_string());
        }

        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err("min() requires numeric arguments".to_string()),
        };

        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => return Err("min() requires numeric arguments".to_string()),
        };

        Ok(Value::Number(a.min(b)))
    }));

    functions.insert("max".to_string(), Box::new(|args| {
        if args.len() != 2 {
            return Err("max() requires exactly 2 arguments".to_string());
        }

        let a = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err("max() requires numeric arguments".to_string()),
        };

        let b = match &args[1] {
            Value::Number(n) => *n,
            _ => return Err("max() requires numeric arguments".to_string()),
        };

        Ok(Value::Number(a.max(b)))
    }));

    functions.insert("clamp".to_string(), Box::new(|args| {
        if args.len() != 3 {
            return Err("clamp() requires exactly 3 arguments".to_string());
        }

        let value = match &args[0] {
            Value::Number(n) => *n,
            _ => return Err("clamp() requires numeric arguments".to_string()),
        };

        let min = match &args[1] {
            Value::Number(n) => *n,
            _ => return Err("clamp() requires numeric arguments".to_string()),
        };

        let max = match &args[2] {
            Value::Number(n) => *n,
            _ => return Err("clamp() requires numeric arguments".to_string()),
        };

        Ok(Value::Number(value.max(min).min(max)))
    }));

    // String functions
    functions.insert("concat".to_string(), Box::new(|args| {
        let mut result = String::new();

        for arg in args {
            match arg {
                Value::String(s) => result.push_str(&s),
                _ => return Err("concat() requires string arguments".to_string()),
            }
        }

        Ok(Value::String(result))
    }));

    // Type conversion functions
    functions.insert("to_string".to_string(), Box::new(|args| {
        if args.len() != 1 {
            return Err("to_string() requires exactly 1 argument".to_string());
        }

        let result = match &args[0] {
            Value::Number(n) => n.to_string(),
            Value::Boolean(b) => b.to_string(),
            Value::String(s) => s.clone(),
            Value::Null => "null".to_string(),
            Value::Object(_) => "<object>".to_string(),
            Value::Array(_) => "<array>".to_string(),
        };

        Ok(Value::String(result))
    }));

    functions.insert("to_number".to_string(), Box::new(|args| {
        if args.len() != 1 {
            return Err("to_number() requires exactly 1 argument".to_string());
        }

        let result = match &args[0] {
            Value::Number(n) => *n,
            Value::Boolean(b) => if *b { 1.0 } else { 0.0 },
            Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
            Value::Null => 0.0,
            Value::Object(_) => 0.0,
            Value::Array(_) => 0.0,
        };

        Ok(Value::Number(result))
    }));

    functions.insert("to_boolean".to_string(), Box::new(|args| {
        if args.len() != 1 {
            return Err("to_boolean() requires exactly 1 argument".to_string());
        }

        let result = match &args[0] {
            Value::Number(n) => *n != 0.0,
            Value::Boolean(b) => *b,
            Value::String(s) => !s.is_empty(),
            Value::Null => false,
            Value::Object(_) => true,
            Value::Array(_) => true,
        };

        Ok(Value::Boolean(result))
    }));

    functions
}
