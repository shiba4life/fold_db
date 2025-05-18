use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub schema_name: String,
    pub fields: Vec<String>,
    pub pub_key: String,
    pub trust_distance: u32,
}

impl Query {
    #[must_use]
    pub const fn new(
        schema_name: String,
        fields: Vec<String>,
        pub_key: String,
        trust_distance: u32,
    ) -> Self {
        Self {
            schema_name,
            fields,
            pub_key,
            trust_distance,
        }
    }
}

#[derive(Debug, Clone, Serialize, ValueEnum)]
pub enum MutationType {
    Create,
    Update,
    Delete,
    #[clap(skip)]
    AddToCollection(String),
    #[clap(skip)]
    UpdateToCollection(String),
    #[clap(skip)]
    DeleteFromCollection(String),
}

impl<'de> Deserialize<'de> for MutationType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "create" => Ok(MutationType::Create),
            "update" => Ok(MutationType::Update),
            "delete" => Ok(MutationType::Delete),
            s if s.starts_with("add_to_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::AddToCollection(id))
            }
            s if s.starts_with("update_to_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::UpdateToCollection(id))
            }
            s if s.starts_with("delete_from_collection:") => {
                let id = s.split(':').nth(1).unwrap_or_default().to_string();
                Ok(MutationType::DeleteFromCollection(id))
            }
            _ => Err(serde::de::Error::custom("unknown mutation type")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mutation {
    pub schema_name: String,
    pub fields_and_values: HashMap<String, Value>,
    pub pub_key: String,
    pub trust_distance: u32,
    pub mutation_type: MutationType,
}

impl Mutation {
    #[must_use]
    pub const fn new(
        schema_name: String,
        fields_and_values: HashMap<String, Value>,
        pub_key: String,
        trust_distance: u32,
        mutation_type: MutationType,
    ) -> Self {
        Self {
            schema_name,
            fields_and_values,
            pub_key,
            trust_distance,
            mutation_type,
        }
    }
}
