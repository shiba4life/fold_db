use serde_json::Value as JsonValue;
use std::path::Path;

pub struct Store {
    db: sled::Db,
}

impl Store {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, sled::Error> {
        let db = sled::open(path)?;
        Ok(Store { db })
    }

    pub fn get_field_value(
        &self,
        schema_name: &str,
        field: &str,
    ) -> Result<JsonValue, String> {
        let key = format!("{}:field:{}", schema_name, field);
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                let value_str = String::from_utf8(data.to_vec())
                    .map_err(|e| format!("Failed to decode value: {}", e))?;
                serde_json::from_str(&value_str)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))
            },
            Ok(None) => Ok(serde_json::Value::Null),
            Err(e) => Err(format!("Database error: {}", e))
        }
    }

    pub fn write_field(
        &self,
        schema_name: &str,
        field: &str,
        value: &JsonValue,
    ) -> Result<(), String> {
        let key = format!("{}:field:{}", schema_name, field);
        let value_str = serde_json::to_string(&serde_json::json!({ "value": value }))
            .map_err(|e| format!("Failed to serialize value: {}", e))?;
        
        self.db.insert(key.as_bytes(), value_str.as_bytes())
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }

    pub fn get_collection(
        &self,
        schema_name: &str,
        collection: &str,
    ) -> Result<Vec<JsonValue>, String> {
        let prefix = format!("{}:collection:{}:", schema_name, collection);
        let mut items = Vec::new();
        
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            let (_, value_bytes) = result.map_err(|e| format!("Database error: {}", e))?;
            let value_str = String::from_utf8(value_bytes.to_vec())
                .map_err(|e| format!("Failed to decode value: {}", e))?;
            let value = serde_json::from_str(&value_str)
                .map_err(|e| format!("Failed to parse JSON: {}", e))?;
            items.push(value);
        }
        
        Ok(items)
    }

    pub fn write_collection(
        &self,
        schema_name: &str,
        collection: &str,
        item: &JsonValue,
    ) -> Result<(), String> {
        let id = match item.get("id") {
            Some(id) => id.to_string(),
            None => return Err("Collection items must have an 'id' field".to_string()),
        };
        
        let key = format!("{}:collection:{}:{}", schema_name, collection, id);
        let value_str = serde_json::to_string(&item)
            .map_err(|e| format!("Failed to serialize value: {}", e))?;
        
        self.db.insert(key.as_bytes(), value_str.as_bytes())
            .map_err(|e| format!("Database error: {}", e))?;
        
        Ok(())
    }
}
