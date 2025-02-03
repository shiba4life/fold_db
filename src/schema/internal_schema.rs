use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use super::types::{ExplicitCounts, PermissionsPolicy};

/// The internal schema maps field names to aref UUIDs and manages permissions.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[derive(Default)]
pub struct InternalSchema {
    pub fields: HashMap<String, String>,
    pub policies: Option<HashMap<String, PermissionsPolicy>>,
    /// Map of public_key to its explicit permission counts.
    #[serde(skip)]
    #[serde(default)]
    explicit_permissions_data: HashMap<String, ExplicitCounts>,
}

impl InternalSchema {
    /// Initialize a new InternalSchema with empty maps
    pub fn new() -> Self {
        InternalSchema {
            fields: HashMap::new(),
            policies: None,
            explicit_permissions_data: HashMap::new(),
        }
    }

    /// Get a mutable reference to the explicit permissions map
    pub fn explicit_permissions(&mut self) -> &mut HashMap<String, ExplicitCounts> {
        &mut self.explicit_permissions_data
    }

    /// Add or update explicit permissions for a public key
    pub fn set_explicit_permissions(&mut self, public_key: String, counts: ExplicitCounts) {
        self.explicit_permissions_data.insert(public_key, counts);
    }

    /// Get explicit permissions for a public key if they exist
    pub fn get_explicit_permissions(&self, public_key: &str) -> Option<&ExplicitCounts> {
        self.explicit_permissions_data.get(public_key)
    }

    /// Get mutable explicit permissions for a public key if they exist
    pub fn get_explicit_permissions_mut(&mut self, public_key: &str) -> Option<&mut ExplicitCounts> {
        self.explicit_permissions_data.get_mut(public_key)
    }
}
