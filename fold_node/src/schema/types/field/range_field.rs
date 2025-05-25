use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::atom::AtomRefRange;
use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::field::common::FieldCommon;
use crate::impl_field;

/// Range filter operations for querying range fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RangeFilter {
    /// Filter by exact key match
    Key(String),
    /// Filter by key prefix
    KeyPrefix(String),
    /// Filter by key range (inclusive start, exclusive end)
    KeyRange { start: String, end: String },
    /// Filter by value match
    Value(String),
    /// Filter by multiple keys
    Keys(Vec<String>),
    /// Filter by key pattern (simple glob-style matching)
    KeyPattern(String),
}

/// Result of a range filter operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeFilterResult {
    pub matches: HashMap<String, String>,
    pub total_count: usize,
}

/// Field storing a range of values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RangeField {
    pub inner: FieldCommon,
    pub atom_ref_range: Option<AtomRefRange>,
}

impl RangeField {
    #[must_use]
    pub fn new(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: None,
        }
    }

    /// Creates a new RangeField with an AtomRefRange
    #[must_use]
    pub fn new_with_range(
        permission_policy: PermissionsPolicy,
        payment_config: FieldPaymentConfig,
        field_mappers: HashMap<String, String>,
        source_pub_key: String,
    ) -> Self {
        Self {
            inner: FieldCommon::new(permission_policy, payment_config, field_mappers),
            atom_ref_range: Some(AtomRefRange::new(source_pub_key)),
        }
    }

    /// Returns a reference to the AtomRefRange if it exists
    pub fn atom_ref_range(&self) -> Option<&AtomRefRange> {
        self.atom_ref_range.as_ref()
    }

    /// Returns a mutable reference to the AtomRefRange if it exists
    pub fn atom_ref_range_mut(&mut self) -> Option<&mut AtomRefRange> {
        self.atom_ref_range.as_mut()
    }

    /// Sets the AtomRefRange for this field
    pub fn set_atom_ref_range(&mut self, atom_ref_range: AtomRefRange) {
        self.atom_ref_range = Some(atom_ref_range);
    }

    /// Initializes the AtomRefRange if it doesn't exist
    pub fn ensure_atom_ref_range(&mut self, source_pub_key: String) -> &mut AtomRefRange {
        if self.atom_ref_range.is_none() {
            self.atom_ref_range = Some(AtomRefRange::new(source_pub_key));
        }
        self.atom_ref_range.as_mut().unwrap()
    }

    /// Applies a range filter to the field's data
    pub fn apply_filter(&self, filter: &RangeFilter) -> RangeFilterResult {
        let empty_result = RangeFilterResult {
            matches: HashMap::new(),
            total_count: 0,
        };

        let Some(atom_ref_range) = &self.atom_ref_range else {
            return empty_result;
        };

        let mut matches = HashMap::new();

        match filter {
            RangeFilter::Key(key) => {
                if let Some(value) = atom_ref_range.get_atom_uuid(key) {
                    matches.insert(key.clone(), value.clone());
                }
            }
            RangeFilter::KeyPrefix(prefix) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if key.starts_with(prefix) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::KeyRange { start, end } => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if key >= start && key < end {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::Value(target_value) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if value == target_value {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::Keys(keys) => {
                for key in keys {
                    if let Some(value) = atom_ref_range.get_atom_uuid(key) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
            RangeFilter::KeyPattern(pattern) => {
                for (key, value) in &atom_ref_range.atom_uuids {
                    if Self::matches_pattern(key, pattern) {
                        matches.insert(key.clone(), value.clone());
                    }
                }
            }
        }

        RangeFilterResult {
            total_count: matches.len(),
            matches,
        }
    }

    /// Applies a filter from a JSON Value (for use with Operation::Query filter)
    pub fn apply_json_filter(&self, filter_value: &Value) -> Result<RangeFilterResult, String> {
        let filter: RangeFilter = serde_json::from_value(filter_value.clone())
            .map_err(|e| format!("Invalid range filter format: {}", e))?;
        Ok(self.apply_filter(&filter))
    }

    /// Simple glob-style pattern matching (supports * and ?)
    fn matches_pattern(text: &str, pattern: &str) -> bool {
        let pattern_chars: Vec<char> = pattern.chars().collect();
        let text_chars: Vec<char> = text.chars().collect();
        
        Self::match_recursive(&text_chars, &pattern_chars, 0, 0)
    }

    fn match_recursive(text: &[char], pattern: &[char], text_idx: usize, pattern_idx: usize) -> bool {
        // If we've reached the end of both strings, it's a match
        if pattern_idx >= pattern.len() && text_idx >= text.len() {
            return true;
        }
        
        // If we've reached the end of pattern but not text, no match
        if pattern_idx >= pattern.len() {
            return false;
        }
        
        match pattern[pattern_idx] {
            '*' => {
                // Try matching zero characters
                if Self::match_recursive(text, pattern, text_idx, pattern_idx + 1) {
                    return true;
                }
                // Try matching one or more characters
                for i in text_idx..text.len() {
                    if Self::match_recursive(text, pattern, i + 1, pattern_idx + 1) {
                        return true;
                    }
                }
                false
            }
            '?' => {
                // Match exactly one character
                if text_idx < text.len() {
                    Self::match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
                } else {
                    false
                }
            }
            c => {
                // Match exact character
                if text_idx < text.len() && text[text_idx] == c {
                    Self::match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
                } else {
                    false
                }
            }
        }
    }

    /// Gets all keys in the range (useful for pagination or listing)
    pub fn get_all_keys(&self) -> Vec<String> {
        self.atom_ref_range
            .as_ref()
            .map(|range| range.atom_uuids.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets a subset of keys within a range (useful for pagination)
    pub fn get_keys_in_range(&self, start: &str, end: &str) -> Vec<String> {
        self.atom_ref_range
            .as_ref()
            .map(|range| {
                let start_string = start.to_string();
                let end_string = end.to_string();
                range.atom_uuids
                    .keys()
                    .filter(|key| **key >= start_string && **key < end_string)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Gets the total count of items in the range
    pub fn count(&self) -> usize {
        self.atom_ref_range
            .as_ref()
            .map(|range| range.atom_uuids.len())
            .unwrap_or(0)
    }
}

impl_field!(RangeField);

