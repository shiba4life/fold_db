Below is an updated FoldDB implementation where internal schemas are stored in a key–value map (i.e. a HashMap) keyed by schema names. Each internal schema maps field names to their corresponding aref_uuids. The FoldDB module now exposes a get_field_value method that accepts a schema name and a field name. This method looks up the appropriate internal schema, finds the aref_uuid for the field, resolves that to the latest atom, and returns its JSON content. GraphQL resolvers call this method to fetch data without exposing internal UUIDs.

// folddb.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sled;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
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

// InternalSchema maps field names to aref_uuids.
#[derive(Serialize, Deserialize, Debug)]
pub struct InternalSchema {
    pub fields: HashMap<String, String>,
}

pub struct FoldDB {
    pub db: sled::Db,
    // Maps a schema name to its internal schema definition.
    pub internal_schemas: HashMap<String, InternalSchema>,
}

impl FoldDB {
    pub fn new(path: &str, internal_schemas: HashMap<String, InternalSchema>) -> sled::Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db, internal_schemas })
    }

    pub fn get_latest_atom(&self, aref_uuid: &str) -> Result<Atom, Box<dyn std::error::Error>> {
        let aref_bytes = self
            .db
            .get(aref_uuid.as_bytes())?
            .ok_or("AtomRef not found")?;
        let aref: AtomRef = serde_json::from_slice(&aref_bytes)?;
        let atom_bytes = self
            .db
            .get(aref.latest_atom.as_bytes())?
            .ok_or("Atom not found")?;
        let atom: Atom = serde_json::from_slice(&atom_bytes)?;
        Ok(atom)
    }

    // Given a schema name and field name, look up the aref_uuid and resolve the field value.
    pub fn get_field_value(&self, schema_name: &str, field: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let internal_schema = self.internal_schemas
            .get(schema_name)
            .ok_or("Internal schema not found")?;
        let aref_uuid = internal_schema.fields
            .get(field)
            .ok_or("Field not found in internal schema")?;
        let atom = self.get_latest_atom(aref_uuid)?;
        // Assume the atom's content is JSON encoded.
        let content: Value = serde_json::from_str(&atom.content)?;
        Ok(content)
    }
}

// main.rs
use async_graphql::*;
use async_graphql::{Context, Schema};
use std::collections::HashMap;
use std::sync::Arc;

mod folddb;
use folddb::{FoldDB, InternalSchema};

#[derive(SimpleObject)]
struct FieldData {
    field: String,
    value: String, // Returns the field value as a JSON string.
}

struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn field(&self, ctx: &Context<'_>, schema_name: String, field: String) -> Result<FieldData> {
        // Retrieve the FoldDB instance from context.
        let fold_db = ctx.data::<Arc<FoldDB>>()?;
        // Use FoldDB to resolve the field value based on the provided schema and field name.
        let value = fold_db
            .get_field_value(&schema_name, &field)
            .map_err(|e| Error::new(e.to_string()))?;
        Ok(FieldData { field, value: value.to_string() })
    }
}

type MySchema = Schema<QueryRoot, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build internal schemas mapping schema names to field-to-aref_uuid mappings.
    let mut internal_schemas = HashMap::new();

    // Example: "user_profile" schema with two fields mapped to their aref_uuids.
    let mut user_profile_fields = HashMap::new();
    user_profile_fields.insert("username".to_string(), "aref-uuid-for-username".to_string());
    user_profile_fields.insert("bio".to_string(), "aref-uuid-for-bio".to_string());
    internal_schemas.insert("user_profile".to_string(), InternalSchema { fields: user_profile_fields });

    // Initialize FoldDB with the internal schemas.
    let fold_db = Arc::new(FoldDB::new("fold_db", internal_schemas)?);

    // Build the GraphQL schema, injecting FoldDB into the context.
    let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription)
        .data(fold_db)
        .finish();

    // Example GraphQL query to fetch the "username" field from the "user_profile" schema.
    let query = r#"
        {
            field(schemaName: "user_profile", field: "username") {
                field
                value
            }
        }
    "#;

    let response = schema.execute(query).await;
    println!("{}", serde_json::to_string_pretty(&response.data)?);
    Ok(())
}

In this design, FoldDB internally holds a key–value map of internal schemas, each mapping field names to aref_uuids. GraphQL resolvers then pass the desired schema name and field name to FoldDB, which automatically resolves the aref_uuid and corresponding atom chain, returning only the content (as JSON) to the client.