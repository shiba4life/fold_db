//! Configuration constants for the transform manager system
//!
//! This module centralizes all configuration keys and constants used
//! throughout the transform management system to avoid duplication
//! and ensure consistency.

/// Key for storing the mapping from atom reference UUIDs to transforms
pub const AREF_TO_TRANSFORMS_KEY: &str = "map_aref_to_transforms";

/// Key for storing the mapping from transform IDs to their dependent atom reference UUIDs
pub const TRANSFORM_TO_AREFS_KEY: &str = "map_transform_to_arefs";

/// Key for storing the mapping from transform IDs to input field names keyed by atom ref UUID
pub const TRANSFORM_INPUT_NAMES_KEY: &str = "map_transform_input_names";

/// Key for storing the mapping from schema.field keys to transforms triggered by them
pub const FIELD_TO_TRANSFORMS_KEY: &str = "map_field_to_transforms";

/// Key for storing the mapping from transform IDs to the fields that trigger them
pub const TRANSFORM_TO_FIELDS_KEY: &str = "map_transform_to_fields";

/// Key for storing the mapping from transform IDs to their output atom reference UUIDs
pub const TRANSFORM_OUTPUTS_KEY: &str = "map_transform_outputs";

/// Default transform system actor identifier
pub const TRANSFORM_SYSTEM_ACTOR: &str = "transform_system";

/// Default timeout for transform execution in milliseconds
pub const TRANSFORM_EXECUTION_TIMEOUT_MS: u64 = 30000;

/// Maximum number of retry attempts for failed transforms
pub const MAX_TRANSFORM_RETRIES: u32 = 3;

/// Batch size for bulk transform operations
pub const TRANSFORM_BATCH_SIZE: usize = 100;