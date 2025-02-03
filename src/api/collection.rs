use serde_json::Value as JsonValue;

pub struct CollectionOptions<'a> {
    pub sort_field: Option<&'a String>,
    pub sort_order: Option<&'a String>,
    pub limit: Option<usize>,
}

pub fn process_collection(mut items: Vec<JsonValue>, opts: &CollectionOptions) -> Vec<JsonValue> {
    // Apply sorting if requested
    if let Some(sort_field) = opts.sort_field {
        items.sort_by(|a, b| {
            let a_val = a.get(sort_field).and_then(|v| v.as_str()).unwrap_or("");
            let b_val = b.get(sort_field).and_then(|v| v.as_str()).unwrap_or("");
            let cmp = a_val.cmp(b_val);
            if let Some(order) = opts.sort_order {
                if order.to_lowercase() == "desc" {
                    cmp.reverse()
                } else {
                    cmp
                }
            } else {
                cmp
            }
        });
    }

    // Apply limit if specified
    if let Some(limit) = opts.limit {
        items.truncate(limit);
    }

    items
}
