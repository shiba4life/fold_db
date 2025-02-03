use super::internal_schema::InternalSchema;
use super::types::{Operation, PolicyLevel};

/// The SecurityManager checks permission for operations.
pub struct SecurityManager;

impl SecurityManager {
    /// Checks permission for a field based on its policy.
    pub fn check_permission(
        schema: &mut InternalSchema,
        field: &str,
        operation: Operation,
        distance: u32,
        explicit_permissions: bool,
        public_key: &str,
    ) -> bool {
        let policy = if let Some(policies) = &schema.policies {
            policies.get(field)
        } else {
            None
        };

        // Default: if no policy exists, allow the operation.
        if policy.is_none() {
            return true;
        }
        let policy = policy.unwrap();

        let level = match operation {
            Operation::Read => &policy.read_policy,
            Operation::Write => &policy.write_policy,
        };

        match level {
            PolicyLevel::Distance(max) => distance <= *max,
            PolicyLevel::Anyone => true,
            PolicyLevel::ExplicitOnce | PolicyLevel::ExplicitMany => {
                if explicit_permissions {
                    // Look up explicit permission by public key.
                    if let Some(counts) = schema.get_explicit_permissions_mut(public_key) {
                        match operation {
                            Operation::Read => counts.r.consume(),
                            Operation::Write => counts.w.consume(),
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
        }
    }

    /// Convenience function that checks a field's permission from the schema.
    pub fn check_field_permission(
        schema: &mut InternalSchema,
        field: &str,
        operation: Operation,
        distance: u32,
        explicit_permissions: bool,
        public_key: &str,
    ) -> bool {
        Self::check_permission(schema, field, operation, distance, explicit_permissions, public_key)
    }
}
