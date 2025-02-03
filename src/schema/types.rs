use serde::{Deserialize, Serialize};
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub enum SchemaError {
    InvalidData(String),
    InvalidDSL(String),
    MappingError(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SchemaError::InvalidData(msg) => write!(f, "Invalid data: {}", msg),
            SchemaError::InvalidDSL(msg) => write!(f, "Invalid DSL: {}", msg),
            SchemaError::MappingError(msg) => write!(f, "Mapping error: {}", msg),
        }
    }
}

impl Error for SchemaError {}

impl From<SchemaError> for String {
    fn from(error: SchemaError) -> String {
        match error {
            SchemaError::InvalidData(msg) => format!("Invalid data: {}", msg),
            SchemaError::InvalidDSL(msg) => format!("Invalid DSL: {}", msg),
            SchemaError::MappingError(msg) => format!("Mapping error: {}", msg),
        }
    }
}


/// Represents either a limited number of operations or unlimited.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Count {
    Limited(u32),
    Unlimited, // represents "n"
}

impl Count {
    /// Attempts to consume one unit of permission.
    /// Returns true if permission was granted, false otherwise.
    pub fn consume(&mut self) -> bool {
        match self {
            Count::Limited(n) => {
                if *n > 0 {
                    *n -= 1;
                    true
                } else {
                    false
                }
            },
            Count::Unlimited => true,
        }
    }
}

/// Structure that tracks explicit read and write counts for a given public key.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExplicitCounts {
    pub r: Count,
    pub w: Count,
}

/// Different types of policy levels.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum PolicyLevel {
    Distance(u32),
    Anyone,
    ExplicitOnce,  // implies a single use (Limited(1))
    ExplicitMany,  // implies explicit permission with a count, or Unlimited ("n")
}

/// Permissions policy for a field.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PermissionsPolicy {
    pub read_policy: PolicyLevel,
    pub write_policy: PolicyLevel,
}

impl PermissionsPolicy {
    pub fn default_allow() -> Self {
        PermissionsPolicy {
            read_policy: PolicyLevel::Anyone,
            write_policy: PolicyLevel::Anyone,
        }
    }
}

/// Define an Operation enum for permission checking.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Operation {
    Read,
    Write,
}
