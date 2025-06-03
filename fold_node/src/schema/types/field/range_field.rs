use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::atom::AtomRefRange;
use crate::fees::types::config::FieldPaymentConfig;
use crate::permissions::types::policy::PermissionsPolicy;
use crate::schema::types::field::common::FieldCommon;
use crate::impl_field;

use crate::schema::types::field::range_filter::{RangeFilter, RangeFilterResult, matches_pattern};
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
                if let Some(values) = atom_ref_range.get_atom_uuids(key) {
                    // Return all atom UUIDs for this key, not just the first one
                    for (i, atom_uuid) in values.iter().enumerate() {
                        let match_key = if values.len() == 1 {
                            key.clone()
                        } else {
                            format!("{}_{}", key, i)
                        };
                        matches.insert(match_key, atom_uuid.clone());
                    }
                }
            }
            RangeFilter::KeyPrefix(prefix) => {
                for (key, values) in &atom_ref_range.atom_uuids {
                    if key.starts_with(prefix) {
                        // For backward compatibility, use the first UUID if available
                        if let Some(first_value) = values.first() {
                            matches.insert(key.clone(), first_value.clone());
                        }
                    }
                }
            }
            RangeFilter::KeyRange { start, end } => {
                for (key, values) in &atom_ref_range.atom_uuids {
                    if key >= start && key < end {
                        // For backward compatibility, use the first UUID if available
                        if let Some(first_value) = values.first() {
                            matches.insert(key.clone(), first_value.clone());
                        }
                    }
                }
            }
            RangeFilter::Value(target_value) => {
                for (key, values) in &atom_ref_range.atom_uuids {
                    // Check if any of the values match the target
                    if values.contains(target_value) {
                        // For backward compatibility, use the first UUID if available
                        if let Some(first_value) = values.first() {
                            matches.insert(key.clone(), first_value.clone());
                        }
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
                for (key, values) in &atom_ref_range.atom_uuids {
                    if matches_pattern(key, pattern) {
                        // For backward compatibility, use the first UUID if available
                        if let Some(first_value) = values.first() {
                            matches.insert(key.clone(), first_value.clone());
                        }
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

