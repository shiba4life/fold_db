use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atom {
    uuid: String,
    source_schema_name: String,
    source_pub_key: String,
    created_at: DateTime<Utc>,
    prev_atom_uuid: Option<String>,
    content: Value,
}

impl Atom {
    #[must_use]
    pub fn new(
        source_schema_name: String,
        source_pub_key: String,
        prev_atom_uuid: Option<String>,
        content: Value,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            source_schema_name,
            source_pub_key,
            created_at: Utc::now(),
            prev_atom_uuid,
            content,
        }
    }

    #[must_use]
    pub const fn content(&self) -> &Value {
        &self.content
    }

    #[must_use]
    pub fn get_transformed_content(&self, transform: &str) -> Value {
        match transform {
            "lowercase" => {
                if let Value::String(s) = &self.content {
                    Value::String(s.to_lowercase())
                } else {
                    self.content.clone()
                }
            }
            _ => self.content.clone(),
        }
    }

    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    #[must_use]
    pub fn source_schema_name(&self) -> &str {
        &self.source_schema_name
    }

    #[must_use]
    pub fn source_pub_key(&self) -> &str {
        &self.source_pub_key
    }

    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    #[must_use]
    pub const fn prev_atom_uuid(&self) -> Option<&String> {
        self.prev_atom_uuid.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRef {
    uuid: String,
    atom_uuid: Option<String>,
    updated_at: DateTime<Utc>,
}

impl AtomRef {
    #[must_use]
    pub fn new(atom_uuid: String) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuid: Some(atom_uuid),
            updated_at: Utc::now(),
        }
    }

    pub fn set_atom_uuid(&mut self, atom_uuid: String) {
        self.atom_uuid = Some(atom_uuid);
    }

    #[must_use]
    pub const fn get_atom_uuid(&self) -> Option<&String> {
        self.atom_uuid.as_ref()
    }

    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_atom_creation() {
        let content = json!({
            "name": "test",
            "value": 42
        });

        let atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            None,
            content.clone(),
        );

        assert_eq!(atom.source_schema_name(), "test_schema");
        assert_eq!(atom.source_pub_key(), "test_key");
        assert_eq!(atom.prev_atom_uuid(), None);
        assert_eq!(atom.content(), &content);
        assert!(atom.uuid().len() > 0);
        assert!(atom.created_at() <= Utc::now());
    }

    #[test]
    fn test_atom_with_prev_reference() {
        let first_atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            None,
            json!({"version": 1}),
        );

        let second_atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            Some(first_atom.uuid().to_string()),
            json!({"version": 2}),
        );

        assert_eq!(
            second_atom.prev_atom_uuid(),
            Some(&first_atom.uuid().to_string())
        );
    }

    #[test]
    fn test_atom_ref_creation_and_update() {
        let atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            None,
            json!({"test": true}),
        );

        let atom_ref = AtomRef::new(atom.uuid().to_string());
        assert_eq!(atom_ref.get_atom_uuid(), Some(&atom.uuid().to_string()));

        let new_atom = Atom::new(
            "test_schema".to_string(),
            "test_key".to_string(),
            Some(atom.uuid().to_string()),
            json!({"test": false}),
        );

        let mut updated_ref = atom_ref.clone();
        updated_ref.set_atom_uuid(new_atom.uuid().to_string());

        assert_eq!(
            updated_ref.get_atom_uuid(),
            Some(&new_atom.uuid().to_string())
        );
        assert!(updated_ref.updated_at() >= atom_ref.updated_at());
    }
}
