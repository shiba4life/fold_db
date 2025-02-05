use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct InternalSchema {
    pub fields: HashMap<String, String>,
}

impl InternalSchema {
    pub fn new() -> Self {
        Self {
            fields: HashMap::new(),
        }
    }
}
