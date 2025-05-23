use serde::Serialize;
use std::fmt;

/// Error type for operations on [`Fold`] structures.
///
/// Mirrors [`SchemaError`] and provides similar variants for fold-related
/// operations.
#[derive(Debug, Clone, Serialize)]
pub enum FoldError {
    /// The requested fold was not found
    NotFound(String),
    /// A field within the fold was invalid
    InvalidField(String),
    /// Permissions were invalid for an operation on the fold
    InvalidPermission(String),
    /// A transform related to the fold was invalid
    InvalidTransform(String),
    /// Data associated with the fold was invalid
    InvalidData(String),
    /// DSL associated with the fold was invalid
    InvalidDSL(String),
    /// Mapping between folds failed
    MappingError(String),
}

impl fmt::Display for FoldError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::NotFound(msg) => write!(f, "Fold not found: {msg}"),
            Self::InvalidField(msg) => write!(f, "Invalid field: {msg}"),
            Self::InvalidPermission(msg) => write!(f, "Invalid permission: {msg}"),
            Self::InvalidTransform(msg) => write!(f, "Invalid transform: {msg}"),
            Self::InvalidData(msg) => write!(f, "Invalid data: {msg}"),
            Self::InvalidDSL(msg) => write!(f, "Invalid DSL: {msg}"),
            Self::MappingError(msg) => write!(f, "Mapping error: {msg}"),
        }
    }
}

impl std::error::Error for FoldError {}

impl From<sled::Error> for FoldError {
    fn from(error: sled::Error) -> Self {
        FoldError::InvalidData(format!("Database error: {}", error))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_works() {
        let err = FoldError::InvalidField("bad".to_string());
        assert_eq!(err.to_string(), "Invalid field: bad");
    }

    #[test]
    fn sled_conversion() {
        let sled_err = sled::Error::Unsupported("x".into());
        let fold_err: FoldError = sled_err.into();
        assert!(matches!(fold_err, FoldError::InvalidData(_)));
    }
}
