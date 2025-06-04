use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "transform/transform.pest"]
pub struct TransformParser;
