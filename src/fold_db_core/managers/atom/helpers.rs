//! Helper functions and utilities for AtomManager

use crate::atom::{Atom, AtomRef, AtomRefRange, AtomStatus};
use crate::db_operations::{DbOperations, EncryptionWrapper};
use serde_json::Value;
use std::sync::Arc;

/// Create a new atom in the database
pub fn create_atom(
    db_ops: &Arc<DbOperations>,
    schema_name: &str,
    source_pub_key: String,
    content: Value,
) -> Result<Atom, Box<dyn std::error::Error>> {
    db_ops
        .create_atom(
            schema_name,
            source_pub_key,
            None,
            content,
            Some(AtomStatus::Active),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Create a new atom in the database with encryption
pub fn create_atom_encrypted(
    db_ops: &Arc<DbOperations>,
    encryption_wrapper: &EncryptionWrapper,
    schema_name: &str,
    source_pub_key: String,
    content: Value,
) -> Result<Atom, Box<dyn std::error::Error>> {
    db_ops
        .create_atom_encrypted(
            encryption_wrapper,
            schema_name,
            source_pub_key,
            None,
            content,
            Some(AtomStatus::Active),
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Update an atom reference
pub fn update_atom_ref(
    db_ops: &Arc<DbOperations>,
    aref_uuid: &str,
    atom_uuid: String,
    source_pub_key: String,
) -> Result<AtomRef, Box<dyn std::error::Error>> {
    db_ops
        .update_atom_ref(aref_uuid, atom_uuid, source_pub_key)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Update an atom reference with encryption
pub fn update_atom_ref_encrypted(
    db_ops: &Arc<DbOperations>,
    encryption_wrapper: &EncryptionWrapper,
    aref_uuid: &str,
    atom_uuid: String,
    source_pub_key: String,
) -> Result<AtomRef, Box<dyn std::error::Error>> {
    db_ops
        .update_atom_ref_encrypted(encryption_wrapper, aref_uuid, atom_uuid, source_pub_key)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Update an atom reference range
pub fn update_atom_ref_range(
    db_ops: &Arc<DbOperations>,
    aref_uuid: &str,
    atom_uuid: String,
    key: String,
    source_pub_key: String,
) -> Result<AtomRefRange, Box<dyn std::error::Error>> {
    db_ops
        .update_atom_ref_range(aref_uuid, atom_uuid, key, source_pub_key)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Update an atom reference range with encryption
pub fn update_atom_ref_range_encrypted(
    db_ops: &Arc<DbOperations>,
    encryption_wrapper: &EncryptionWrapper,
    aref_uuid: &str,
    atom_uuid: String,
    key: String,
    source_pub_key: String,
) -> Result<AtomRefRange, Box<dyn std::error::Error>> {
    db_ops
        .update_atom_ref_range_encrypted(
            encryption_wrapper,
            aref_uuid,
            atom_uuid,
            key,
            source_pub_key,
        )
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)
}

/// Get atom history for a given atom reference
pub fn get_atom_history(
    db_ops: &Arc<DbOperations>,
    aref_uuid: &str,
) -> Result<Vec<Atom>, Box<dyn std::error::Error>> {
    // Load the atom ref from database
    let key = format!("ref:{}", aref_uuid);

    match db_ops.db().get(&key)? {
        Some(bytes) => {
            // Try to deserialize as AtomRef first
            if let Ok(atom_ref) = serde_json::from_slice::<AtomRef>(&bytes) {
                let atom_uuid = atom_ref.get_atom_uuid();

                // Get the current atom
                let atom_key = format!("atom:{}", atom_uuid);
                match db_ops.db().get(&atom_key)? {
                    Some(atom_bytes) => {
                        let atom: Atom = serde_json::from_slice(&atom_bytes)?;
                        Ok(vec![atom])
                    }
                    None => Ok(vec![]),
                }
            } else {
                // Try as AtomRefRange
                if let Ok(_range) = serde_json::from_slice::<AtomRefRange>(&bytes) {
                    // For ranges, we would need to iterate through all atoms in the range
                    // For now, return empty vector
                    Ok(vec![])
                } else {
                    Err("Failed to deserialize atom reference".into())
                }
            }
        }
        None => Ok(vec![]),
    }
}
