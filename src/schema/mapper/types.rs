use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "rule", rename_all = "lowercase")]
pub enum MappingRule {
    Rename {
        source_field: String,
        target_field: String,
    },
    Drop {
        field: String,
    },
    Map {
        source_field: String,
        target_field: String,
        function: Option<String>,
    },
}
