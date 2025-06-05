use crate::schema::types::errors::SchemaError;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Helper functions for acquiring read/write locks with consistent error handling
pub struct LockHelper;

impl LockHelper {
    /// Acquire a read lock with consistent error handling
    pub fn read_lock<'a, T>(lock: &'a RwLock<T>, _lock_name: &'a str) -> Result<RwLockReadGuard<'a, T>, SchemaError> {
        lock.read().map_err(|_| {
            SchemaError::InvalidData(format!("Failed to acquire {} lock", _lock_name))
        })
    }
    
    /// Acquire a write lock with consistent error handling
    pub fn write_lock<'a, T>(lock: &'a RwLock<T>, _lock_name: &'a str) -> Result<RwLockWriteGuard<'a, T>, SchemaError> {
        lock.write().map_err(|_| {
            SchemaError::InvalidData(format!("Failed to acquire {} write lock", _lock_name))
        })
    }
}