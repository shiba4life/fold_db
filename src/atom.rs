use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Atom {
    pub uuid: String,
    pub content: String,
    #[serde(rename = "type")]
    pub type_field: String,
    pub source: String,
    pub created_at: DateTime<Utc>,
    pub prev: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AtomRef {
    pub latest_atom: String,
}
