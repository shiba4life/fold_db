//! Simple arithmetic parser using Pest.

use pest::Parser;
use pest::iterators::Pair;
use pest_derive::Parser;

/// Parser for the simple arithmetic grammar.
#[derive(Parser)]
#[grammar = "src/schema/transform/simple_transform.pest"]
pub struct SimpleParser;

/// Evaluate a parsed Pest pair into an i64 result.
fn eval(pair: Pair<Rule>) -> i64 {
    match pair.as_rule() {
        Rule::calc => {
            // Unwrap the expression inside calc
            let inner = pair.into_inner().next().unwrap();
            eval(inner)
        }
        Rule::expr => {
            let mut inner = pair.into_inner();
            let mut result = eval(inner.next().unwrap());
            while let Some(op_pair) = inner.next() {
                let op = op_pair.as_str();
                let rhs = eval(inner.next().unwrap());
                if op == "+" {
                    result += rhs;
                } else {
                    result -= rhs;
                }
            }
            result
        }
        Rule::term => {
            let mut inner = pair.into_inner();
            let mut result = eval(inner.next().unwrap());
            while let Some(op_pair) = inner.next() {
                let op = op_pair.as_str();
                let rhs = eval(inner.next().unwrap());
                if op == "*" {
                    result *= rhs;
                } else {
                    result /= rhs;
                }
            }
            result
        }
        Rule::factor => {
            // factor = { number | "(" ~ expr ~ ")" }
            let inner = pair.into_inner().next().unwrap();
            eval(inner)
        }
        Rule::number => {
            pair.as_str().parse::<i64>().unwrap()
        }
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

/// Calculate the value of the input expression.
pub fn calculate(input: &str) -> Result<i64, Box<pest::error::Error<Rule>>> {
    // Try full parse with calc rule, then fallback to expr
    let mut pairs = SimpleParser::parse(Rule::calc, input)
        .or_else(|_| SimpleParser::parse(Rule::expr, input))
        .map_err(Box::new)?;
    let pair = pairs.next().unwrap();
    Ok(eval(pair))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_simple() {
        assert_eq!(calculate("2+3").unwrap(), 5);
        assert_eq!(calculate("10-4+2").unwrap(), 8);
    }

    #[test]
    fn test_calculate_precedence() {
        assert_eq!(calculate("2+3*4").unwrap(), 14);
        assert_eq!(calculate("(2+3)*4").unwrap(), 20);
        assert_eq!(calculate("18/3/3").unwrap(), 2);
    }
}
