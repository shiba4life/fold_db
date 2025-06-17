//! Query and mutation command handlers
//! 
//! This module contains handlers for data operations including
//! queries, mutations, and executing operations from files.

use crate::{DataFoldNode, MutationType, Operation};
use log::info;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

/// Handle executing a query
pub fn handle_query(
    node: &mut DataFoldNode,
    schema: String,
    fields: Vec<String>,
    filter: Option<String>,
    output: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing query on schema: {}", schema);

    let filter_value = if let Some(filter_str) = filter {
        Some(serde_json::from_str(&filter_str)?)
    } else {
        None
    };

    let operation = Operation::Query {
        schema,
        fields,
        filter: filter_value,
    };

    let result = node.execute_operation(operation)?;

    if output == "json" {
        info!("{}", result);
    } else {
        info!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Handle executing a mutation
pub fn handle_mutate(
    node: &mut DataFoldNode,
    schema: String,
    mutation_type: MutationType,
    data: String,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing mutation on schema: {}", schema);

    let data_value: Value = serde_json::from_str(&data)?;

    let operation = Operation::Mutation {
        schema,
        data: data_value,
        mutation_type,
    };

    node.execute_operation(operation)?;
    info!("Mutation executed successfully");

    Ok(())
}

/// Handle executing an operation from file
pub fn handle_execute(
    path: PathBuf,
    node: &mut DataFoldNode,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Executing operation from file: {}", path.display());
    let operation_str = fs::read_to_string(path)?;
    let operation: Operation = serde_json::from_str(&operation_str)?;

    let result = node.execute_operation(operation)?;

    if !result.is_null() {
        info!("Result:");
        info!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        info!("Operation executed successfully");
    }

    Ok(())
}