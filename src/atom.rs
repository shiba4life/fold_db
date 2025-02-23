use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// An immutable data container that represents a single version of content in the database.
/// 
/// Atoms are the fundamental building blocks of the database's immutable data storage system.
/// Each Atom contains:
/// - A unique identifier
/// - The source schema that defines its structure
/// - The public key of the creator
/// - Creation timestamp
/// - Optional reference to a previous version
/// - The actual content data
/// 
/// Atoms form a chain of versions through their `prev_atom_uuid` references, enabling
/// complete version history tracking. Once created, an Atom's content cannot be modified,
/// ensuring data immutability.
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
    /// Creates a new Atom with the given parameters.
    /// 
    /// # Arguments
    /// 
    /// * `source_schema_name` - Name of the schema that defines this Atom's structure
    /// * `source_pub_key` - Public key of the entity creating this Atom
    /// * `prev_atom_uuid` - Optional UUID of the previous version of this content
    /// * `content` - The actual data content stored in this Atom
    /// 
    /// # Returns
    /// 
    /// A new Atom instance with a generated UUID and current timestamp
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

    /// Returns a reference to the Atom's content.
    /// 
    /// This method provides read-only access to the stored data,
    /// maintaining the immutability principle.
    #[must_use]
    pub const fn content(&self) -> &Value {
        &self.content
    }

    /// Applies a transformation to the Atom's content and returns the result.
    /// 
    /// Currently supports:
    /// - "lowercase": Converts string content to lowercase
    /// 
    /// If the transformation is not recognized or cannot be applied,
    /// returns a clone of the original content.
    /// 
    /// # Arguments
    /// 
    /// * `transform` - The name of the transformation to apply
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

    /// Returns the unique identifier of this Atom.
    /// 
    /// This UUID uniquely identifies this specific version of the data
    /// and is used by AtomRefs to point to the current version.
    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Returns the name of the schema that defines this Atom's structure.
    /// 
    /// The schema name is used to validate the content structure and
    /// determine applicable permissions and payment requirements.
    #[must_use]
    pub fn source_schema_name(&self) -> &str {
        &self.source_schema_name
    }

    /// Returns the public key of the entity that created this Atom.
    /// 
    /// This is used for authentication and permission validation
    /// when accessing or modifying the data.
    #[must_use]
    pub fn source_pub_key(&self) -> &str {
        &self.source_pub_key
    }

    /// Returns the timestamp when this Atom was created.
    /// 
    /// This timestamp is used for auditing and version history tracking.
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    /// Returns the UUID of the previous version of this data, if any.
    /// 
    /// This forms the chain of version history, allowing traversal
    /// through all previous versions of the data.
    #[must_use]
    pub const fn prev_atom_uuid(&self) -> Option<&String> {
        self.prev_atom_uuid.as_ref()
    }
}

/// A mutable reference to one or more Atom versions.
/// 
/// AtomRefs provide a level of indirection that enables atomic updates
/// to data while maintaining the immutable nature of Atoms. They track:
/// - A unique identifier for the reference itself
/// - A mapping of keys to Atom UUIDs
/// - The timestamp of the last update
/// 
/// The mapping supports both array-like access (using "0".."n" as keys)
/// and map-like access (using arbitrary string keys). The default key "0"
/// maintains backward compatibility with single-atom references.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomRef {
    uuid: String,
    atom_uuids: HashMap<String, String>,
    updated_at: DateTime<Utc>,
}

impl AtomRef {
    /// Creates a new AtomRef pointing to the specified Atom using the default key "0".
    /// 
    /// # Arguments
    /// 
    /// * `atom_uuid` - UUID of the Atom this reference should point to
    /// 
    /// # Returns
    /// 
    /// A new AtomRef instance with a generated UUID and current timestamp
    #[must_use]
    pub fn new(atom_uuid: String) -> Self {
        let mut atom_uuids = HashMap::new();
        atom_uuids.insert("0".to_string(), atom_uuid);
        Self {
            uuid: Uuid::new_v4().to_string(),
            atom_uuids,
            updated_at: Utc::now(),
        }
    }

    /// Updates the reference at the default key "0" to point to a new Atom version.
    /// 
    /// This maintains backward compatibility with the previous single-reference implementation.
    /// 
    /// # Arguments
    /// 
    /// * `atom_uuid` - UUID of the new Atom version to reference
    pub fn set_atom_uuid(&mut self, atom_uuid: String) {
        self.set_atom_uuid_with_key("0".to_string(), atom_uuid);
    }

    /// Updates or adds a reference at the specified key to point to an Atom version.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key to associate with the Atom UUID
    /// * `atom_uuid` - UUID of the Atom version to reference
    pub fn set_atom_uuid_with_key(&mut self, key: String, atom_uuid: String) {
        self.atom_uuids.insert(key, atom_uuid);
        self.updated_at = Utc::now();
    }

    /// Returns the UUID of the Atom referenced by the default key "0".
    /// 
    /// This maintains backward compatibility with the previous single-reference implementation.
    /// Returns None if no Atom is referenced at the default key.
    #[must_use]
    pub fn get_atom_uuid(&self) -> Option<&String> {
        self.atom_uuids.get("0")
    }

    /// Returns the UUID of the Atom referenced by the specified key.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key whose associated Atom UUID should be returned
    /// 
    /// Returns None if no Atom is referenced at the specified key.
    #[must_use]
    pub fn get_atom_uuid_by_key(&self, key: &str) -> Option<&String> {
        self.atom_uuids.get(key)
    }

    /// Removes the reference at the specified key.
    /// 
    /// # Arguments
    /// 
    /// * `key` - The key whose associated Atom UUID should be removed
    /// 
    /// Returns the removed Atom UUID if it existed, None otherwise.
    pub fn remove_atom_uuid(&mut self, key: &str) -> Option<String> {
        let result = self.atom_uuids.remove(key);
        if result.is_some() {
            self.updated_at = Utc::now();
        }
        result
    }

    /// Returns the unique identifier of this AtomRef.
    /// 
    /// This UUID identifies the reference itself, not the Atoms it points to.
    #[must_use]
    pub fn uuid(&self) -> &str {
        &self.uuid
    }

    /// Returns the timestamp of the last update to this reference.
    /// 
    /// This timestamp is updated whenever references are modified, added, or removed.
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

        // Test default key behavior
        let atom_ref = AtomRef::new(atom.uuid().to_string());
        assert_eq!(atom_ref.get_atom_uuid(), Some(&atom.uuid().to_string()));
        assert_eq!(atom_ref.get_atom_uuid_by_key("0"), Some(&atom.uuid().to_string()));

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

    #[test]
    fn test_atom_ref_with_multiple_keys() {
        let atoms: Vec<_> = (0..3)
            .map(|i| {
                Atom::new(
                    "test_schema".to_string(),
                    "test_key".to_string(),
                    None,
                    json!({ "index": i }),
                )
            })
            .collect();

        // Test array-like usage
        let mut array_ref = AtomRef::new(atoms[0].uuid().to_string());
        array_ref.set_atom_uuid_with_key("1".to_string(), atoms[1].uuid().to_string());
        array_ref.set_atom_uuid_with_key("2".to_string(), atoms[2].uuid().to_string());

        assert_eq!(array_ref.get_atom_uuid_by_key("0"), Some(&atoms[0].uuid().to_string()));
        assert_eq!(array_ref.get_atom_uuid_by_key("1"), Some(&atoms[1].uuid().to_string()));
        assert_eq!(array_ref.get_atom_uuid_by_key("2"), Some(&atoms[2].uuid().to_string()));

        // Test map-like usage
        let mut map_ref = AtomRef::new(atoms[0].uuid().to_string());
        map_ref.set_atom_uuid_with_key("first".to_string(), atoms[1].uuid().to_string());
        map_ref.set_atom_uuid_with_key("second".to_string(), atoms[2].uuid().to_string());

        assert_eq!(map_ref.get_atom_uuid(), Some(&atoms[0].uuid().to_string())); // Default key
        assert_eq!(map_ref.get_atom_uuid_by_key("first"), Some(&atoms[1].uuid().to_string()));
        assert_eq!(map_ref.get_atom_uuid_by_key("second"), Some(&atoms[2].uuid().to_string()));

        // Test removal
        assert_eq!(map_ref.remove_atom_uuid("first"), Some(atoms[1].uuid().to_string()));
        assert_eq!(map_ref.get_atom_uuid_by_key("first"), None);
    }
}
