use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AtomRefStatus {
    Active,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRefUpdate {
    pub(crate) timestamp: DateTime<Utc>,
    pub(crate) status: AtomRefStatus,
    pub(crate) source_pub_key: String,
}