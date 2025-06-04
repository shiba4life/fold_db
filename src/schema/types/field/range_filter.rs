use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Simple glob-style pattern matching (supports `*` and `?`)
pub fn matches_pattern(text: &str, pattern: &str) -> bool {
    let pattern_chars: Vec<char> = pattern.chars().collect();
    let text_chars: Vec<char> = text.chars().collect();

    match_recursive(&text_chars, &pattern_chars, 0, 0)
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
            if match_recursive(text, pattern, text_idx, pattern_idx + 1) {
                return true;
            }
            // Try matching one or more characters
            for i in text_idx..text.len() {
                if match_recursive(text, pattern, i + 1, pattern_idx + 1) {
                    return true;
                }
            }
            false
        }
        '?' => {
            // Match exactly one character
            if text_idx < text.len() {
                match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
            } else {
                false
            }
        }
        c => {
            // Match exact character
            if text_idx < text.len() && text[text_idx] == c {
                match_recursive(text, pattern, text_idx + 1, pattern_idx + 1)
            } else {
                false
            }
        }
    }
}
