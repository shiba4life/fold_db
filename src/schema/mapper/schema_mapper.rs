use serde::{Deserialize, Serialize};
use super::types::MappingRule;

/// SchemaMapper supports mapping data from multiple source schemas to a target schema
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaMapper {
    /// List of source schema names
    pub source_schema_name: String,
    /// Target schema name
    pub target_schema_name: String,
    /// Mapping rules to apply
    pub rules: Vec<MappingRule>,
}

impl SchemaMapper {
    /// Create a new SchemaMapper
    pub fn new(source_schema_name: String, target_schema_name: String, rules: Vec<MappingRule>) -> Self {
        Self {
            source_schema_name,
            target_schema_name,
            rules,
        }
    }
}
