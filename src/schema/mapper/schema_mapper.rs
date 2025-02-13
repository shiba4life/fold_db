use super::types::MappingRule;
use serde::{Deserialize, Serialize};

/// `SchemaMapper` supports mapping data from source schema to the schema it belongs to
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SchemaMapper {
    /// Source schema name
    pub source_schema_name: String,
    /// Mapping rules to apply
    pub rules: Vec<MappingRule>,
}

impl SchemaMapper {
    /// Create a new `SchemaMapper`
    #[must_use]
    pub const fn new(
        source_schema_name: String,
        rules: Vec<MappingRule>,
    ) -> Self {
        Self {
            source_schema_name,
            rules,
        }
    }
}
