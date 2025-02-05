use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "rule", rename_all = "lowercase")]
pub enum MappingRule {
    Rename { source_field: String, target_field: String },
    Drop { field: String },
    Add { target_field: String, value: Value },
    Map { source_field: String, target_field: String, function: String },
}
