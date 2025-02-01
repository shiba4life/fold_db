pub const FIELD_QUERY: &str = r#"
    {
        username: field(schemaName: "user_profile", field: "username") {
            field
            value
        }
        bio: field(schemaName: "user_profile", field: "bio") {
            field
            value
        }
    }
"#;

pub const FIELD_HISTORY_QUERY: &str = r#"
    {
        username: fieldHistory(schemaName: "user_profile", field: "username")
        bio: fieldHistory(schemaName: "user_profile", field: "bio")
    }
"#;

pub const UPDATE_FIELDS_MUTATION: &str = r#"
    mutation {
        username: updateField(
            schemaName: "user_profile"
            field: "username"
            value: "\"new_username\""
            source: "user_update"
        )
        bio: updateField(
            schemaName: "user_profile"
            field: "bio"
            value: "\"Full-stack developer with a passion for Rust\""
            source: "user_update"
        )
    }
"#;
