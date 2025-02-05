use std::collections::HashMap;
use super::types::{PolicyLevel, Count, ExplicitCounts, PermissionsPolicy, Operation, SchemaError};

pub struct SecurityManager {
    pub_keys: HashMap<String, Vec<String>>, // user -> pub_keys
}

impl SecurityManager {
    pub fn new() -> Self {
        Self {
            pub_keys: HashMap::new(),
        }
    }

    pub fn register_pub_key(&mut self, user: String, pub_key: String) {
        self.pub_keys.entry(user).or_default().push(pub_key);
    }

    pub fn check_permission(&self, policy: &PermissionsPolicy, pub_key: &str, op: Operation) -> Result<bool, SchemaError> {
        let policy_level = match op {
            Operation::Read => &policy.read_policy,
            Operation::Write => &policy.write_policy,
        };

        match policy_level {
            PolicyLevel::Public => Ok(true),
            PolicyLevel::Private => Ok(false),
            PolicyLevel::Explicit | PolicyLevel::ExplicitOnce | PolicyLevel::ExplicitMany => {
                if let Some(counts) = &policy.explicit_counts {
                    if let Some(count) = counts.counts.get(pub_key) {
                        match op {
                            Operation::Read => Ok(count.read > 0),
                            Operation::Write => Ok(count.write > 0),
                        }
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
        }
    }

    pub fn update_counts(&mut self, counts: &mut ExplicitCounts, pub_key: &str, op: Operation) -> Result<(), SchemaError> {
        if let Some(count) = counts.counts.get_mut(pub_key) {
            match op {
                Operation::Read => {
                    if count.read > 0 {
                        count.read -= 1;
                    }
                }
                Operation::Write => {
                    if count.write > 0 {
                        count.write -= 1;
                    }
                }
            }
            Ok(())
        } else {
            Err(SchemaError::InvalidPermission(format!(
                "No explicit access for pub_key: {}", pub_key
            )))
        }
    }
}
