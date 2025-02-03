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

    pub fn get_field_value(&self, schema: &str, field: &str) -> Result<JsonValue, String> {
        let key = format!("{}:field:{}", schema, field);
        match self.db.get(key.as_bytes()) {
            Ok(Some(data)) => {
                let value_str = String::from_utf8(data.to_vec())
                    .map_err(|e| format!("Failed to decode value: {}", e))?;
                let raw_value: JsonValue = serde_json::from_str(&value_str)
                    .map_err(|e| format!("Failed to parse JSON: {}", e))?;
                Ok(serde_json::json!({
                    "value": raw_value
                }))
            },
            Ok(None) => Ok(serde_json::json!({
                "value": JsonValue::Null
            })),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    pub fn get_collection(&self, schema: &str, collection: &str) -> Result<Vec<JsonValue>, String> {
        let prefix = format!("{}:collection:{}:", schema, collection);
        let mut items = Vec::new();
        
        for result in self.db.scan_prefix(prefix.as_bytes()) {
            match result {
                Ok((_, data)) => {
                    let value_str = String::from_utf8(data.to_vec())
                        .map_err(|e| format!("Failed to decode value: {}", e))?;
                    let value: JsonValue = serde_json::from_str(&value_str)
                        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
                    items.push(value);
                },
                Err(e) => return Err(format!("Database error: {}", e)),
            }
        }
        
        Ok(items)
    }

    pub fn write_field(&self, schema: &str, field: &str, value: &JsonValue) -> Result<(), String> {
        let key = format!("{}:field:{}", schema, field);
        let wrapped_value = serde_json::json!({
            "value": value
        });
        let value_str = serde_json::to_string(&wrapped_value)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        
        self.db.insert(key.as_bytes(), value_str.as_bytes())
            .map(|_| ())
            .map_err(|e| format!("Database error: {}", e))?;
        self.db.flush()
            .map(|_| ())
            .map_err(|e| format!("Failed to flush database: {}", e))
    }

    pub fn write_collection(&self, schema: &str, collection: &str, item: &JsonValue) -> Result<(), String> {
        let id = match item.get("id") {
            Some(id) => id.to_string(),
            None => return Err("Collection items must have an 'id' field".to_string()),
        };
        
        let key = format!("{}:collection:{}:{}", schema, collection, id);
        let value_str = serde_json::to_string(item)
            .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
        
        self.db.insert(key.as_bytes(), value_str.as_bytes())
            .map(|_| ())
            .map_err(|e| format!("Database error: {}", e))?;
        self.db.flush()
            .map(|_| ())
            .map_err(|e| format!("Failed to flush database: {}", e))
    }
}
