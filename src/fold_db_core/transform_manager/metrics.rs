//! Metrics, logging, and diagnostics for transform management
//!
//! This module provides:
//! - Logging utilities for transform operations
//! - Performance monitoring and metrics
//! - Diagnostic tools for debugging
//! - System health checks

use log::{debug, error, info, warn};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

/// Logging helper for transform manager operations
pub struct LoggingHelper;

impl LoggingHelper {
    /// Log transform registration details
    pub fn log_transform_registration(transform_id: &str, inputs: &[String], output: &str) {
        info!(
            "📝 Registering transform '{}' with inputs: {:?}, output: '{}'",
            transform_id, inputs, output
        );
        debug!("Transform registration details: ID={}, input_count={}, output={}", 
               transform_id, inputs.len(), output);
    }

    /// Log field mapping creation
    pub fn log_field_mapping_creation(field_key: &str, transform_id: &str) {
        info!("🔗 Creating field mapping: '{}' -> transform '{}'", field_key, transform_id);
        debug!("Field mapping: field={}, transform={}", field_key, transform_id);
    }

    /// Log verification result with optional details
    pub fn log_verification_result(item_type: &str, id: &str, details: Option<&str>) {
        match details {
            Some(detail_str) => info!("✅ VERIFIED {}: {} - {}", item_type, id, detail_str),
            None => info!("✅ VERIFIED {}: {}", item_type, id),
        }
        debug!("Verification: type={}, id={}, details={:?}", item_type, id, details);
    }

    /// Log atom reference operations
    pub fn log_atom_ref_operation(ref_uuid: &str, atom_uuid: &str, operation: &str) {
        info!("🔗 AtomRef {}: {} -> atom {}", operation, ref_uuid, atom_uuid);
        debug!("AtomRef operation: op={}, ref={}, atom={}", operation, ref_uuid, atom_uuid);
    }

    /// Log the current state of field mappings
    pub fn log_field_mappings_state(mappings: &HashMap<String, HashSet<String>>, context: &str) {
        info!("🔍 DEBUG: Field mappings state in context '{}' ({} entries):", context, mappings.len());
        for (field_key, transforms) in mappings {
            info!("  📋 '{}' -> {:?}", field_key, transforms);
        }
        if mappings.is_empty() {
            warn!("⚠️ DEBUG: No field mappings found in context '{}'!", context);
        }
    }

    /// Log collection state for debugging
    pub fn log_collection_state<T: std::fmt::Debug>(
        collection: &HashMap<String, T>,
        collection_name: &str,
        context: &str,
    ) {
        info!("🔍 DEBUG: {} state in context '{}' ({} entries):", 
              collection_name, context, collection.len());
        for (key, value) in collection {
            debug!("  📋 '{}' -> {:?}", key, value);
        }
        if collection.is_empty() {
            warn!("⚠️ DEBUG: No entries found in {} for context '{}'!", collection_name, context);
        }
    }

    /// Log transform execution start
    pub fn log_transform_execution_start(transform_id: &str, inputs: &HashMap<String, serde_json::Value>) {
        info!("🚀 Starting execution of transform '{}'", transform_id);
        debug!("Transform execution inputs: {:?}", inputs);
    }

    /// Log transform execution completion
    pub fn log_transform_execution_completion(
        transform_id: &str, 
        result: &Result<serde_json::Value, crate::schema::types::SchemaError>,
        duration: Duration,
    ) {
        match result {
            Ok(value) => {
                info!("✅ Transform '{}' executed successfully in {:?}", transform_id, duration);
                debug!("Transform result: {:?}", value);
            }
            Err(error) => {
                error!("❌ Transform '{}' failed after {:?}: {}", transform_id, duration, error);
            }
        }
    }

    /// Log system state summary
    pub fn log_system_state_summary(
        registered_count: usize,
        field_mapping_count: usize,
        aref_mapping_count: usize,
    ) {
        info!("📊 Transform Manager State Summary:");
        info!("  📝 Registered transforms: {}", registered_count);
        info!("  🔗 Field mappings: {}", field_mapping_count);
        info!("  🔗 Atom reference mappings: {}", aref_mapping_count);
    }

    /// Log error with context
    pub fn log_error_with_context(operation: &str, error: &dyn std::error::Error, context: &str) {
        error!("❌ Error in {} (context: {}): {}", operation, context, error);
        debug!("Error details: operation={}, context={}, error={:?}", operation, context, error);
    }

    /// Log warning with context
    pub fn log_warning_with_context(operation: &str, message: &str, context: &str) {
        warn!("⚠️ Warning in {} (context: {}): {}", operation, context, message);
        debug!("Warning details: operation={}, context={}, message={}", operation, context, message);
    }
}

/// Performance monitoring for transform operations
pub struct PerformanceMonitor {
    operation_times: HashMap<String, Vec<Duration>>,
    start_times: HashMap<String, Instant>,
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            operation_times: HashMap::new(),
            start_times: HashMap::new(),
        }
    }

    /// Start timing an operation
    pub fn start_timing(&mut self, operation_id: &str) {
        self.start_times.insert(operation_id.to_string(), Instant::now());
        debug!("⏱️ Started timing operation: {}", operation_id);
    }

    /// End timing an operation and record the duration
    pub fn end_timing(&mut self, operation_id: &str) -> Option<Duration> {
        if let Some(start_time) = self.start_times.remove(operation_id) {
            let duration = start_time.elapsed();
            self.operation_times
                .entry(operation_id.to_string())
                .or_default()
                .push(duration);
            
            debug!("⏱️ Completed timing operation: {} in {:?}", operation_id, duration);
            info!("📊 Operation '{}' took {:?}", operation_id, duration);
            Some(duration)
        } else {
            warn!("⚠️ Attempted to end timing for unknown operation: {}", operation_id);
            None
        }
    }

    /// Get average time for an operation
    pub fn get_average_time(&self, operation_id: &str) -> Option<Duration> {
        self.operation_times.get(operation_id).and_then(|times| {
            if times.is_empty() {
                None
            } else {
                let total_nanos: u64 = times.iter().map(|d| d.as_nanos() as u64).sum();
                let avg_nanos = total_nanos / times.len() as u64;
                Some(Duration::from_nanos(avg_nanos))
            }
        })
    }

    /// Log performance summary
    pub fn log_performance_summary(&self) {
        info!("📊 Performance Summary:");
        for (operation, times) in &self.operation_times {
            if let Some(avg) = self.get_average_time(operation) {
                info!(
                    "  🔄 {}: {} operations, avg {:?}",
                    operation,
                    times.len(),
                    avg
                );
            }
        }
    }

    /// Reset all timing data
    pub fn reset(&mut self) {
        self.operation_times.clear();
        self.start_times.clear();
        info!("🔄 Performance monitor reset");
    }
}

/// Diagnostic utilities for system health checks
pub struct DiagnosticHelper;

impl DiagnosticHelper {
    /// Perform health check on transform manager state
    pub fn health_check_state(
        registered_transforms: &HashMap<String, crate::schema::types::Transform>,
        field_to_transforms: &HashMap<String, HashSet<String>>,
        transform_to_fields: &HashMap<String, HashSet<String>>,
        aref_to_transforms: &HashMap<String, HashSet<String>>,
        transform_to_arefs: &HashMap<String, HashSet<String>>,
    ) -> HealthCheckResult {
        let mut result = HealthCheckResult::new();

        // Check for orphaned field mappings
        for (field, transforms) in field_to_transforms {
            for transform_id in transforms {
                if !registered_transforms.contains_key(transform_id) {
                    result.add_warning(format!(
                        "Field '{}' references unregistered transform '{}'",
                        field, transform_id
                    ));
                }
            }
        }

        // Check for inconsistent bidirectional mappings
        for (transform_id, fields) in transform_to_fields {
            for field in fields {
                if let Some(transforms) = field_to_transforms.get(field) {
                    if !transforms.contains(transform_id) {
                        result.add_error(format!(
                            "Inconsistent mapping: transform '{}' claims field '{}' but field doesn't reference transform",
                            transform_id, field
                        ));
                    }
                } else {
                    result.add_error(format!(
                        "Inconsistent mapping: transform '{}' claims field '{}' but field has no mappings",
                        transform_id, field
                    ));
                }
            }
        }

        // Check for orphaned atom reference mappings
        for (aref, transforms) in aref_to_transforms {
            for transform_id in transforms {
                if !registered_transforms.contains_key(transform_id) {
                    result.add_warning(format!(
                        "Atom reference '{}' references unregistered transform '{}'",
                        aref, transform_id
                    ));
                }
            }
        }

        // Log summary
        info!("🏥 Health check completed: {} errors, {} warnings", 
              result.errors.len(), result.warnings.len());

        result
    }

    /// Check database consistency
    pub fn check_database_consistency(
        db_ops: &std::sync::Arc<crate::db_operations::DbOperations>,
    ) -> Result<ConsistencyCheckResult, crate::schema::types::SchemaError> {
        let mut result = ConsistencyCheckResult::new();

        // Check if all registered transforms exist in database
        let db_transform_ids = db_ops.list_transforms()?;
        
        for transform_id in &db_transform_ids {
            match db_ops.get_transform(transform_id) {
                Ok(Some(_)) => {
                    result.add_success(format!("Transform '{}' found in database", transform_id));
                }
                Ok(None) => {
                    result.add_error(format!("Transform '{}' listed but not found in database", transform_id));
                }
                Err(e) => {
                    result.add_error(format!("Failed to load transform '{}': {}", transform_id, e));
                }
            }
        }

        info!("🔍 Database consistency check completed: {} successes, {} errors", 
              result.successes.len(), result.errors.len());

        Ok(result)
    }
}

/// Health check result structure
pub struct HealthCheckResult {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl HealthCheckResult {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn is_healthy(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn log_results(&self) {
        if self.is_healthy() {
            info!("✅ Health check passed");
        } else {
            error!("❌ Health check failed with {} errors", self.errors.len());
        }

        for error in &self.errors {
            error!("  ❌ {}", error);
        }

        for warning in &self.warnings {
            warn!("  ⚠️ {}", warning);
        }
    }
}

/// Database consistency check result
pub struct ConsistencyCheckResult {
    pub successes: Vec<String>,
    pub errors: Vec<String>,
}

impl ConsistencyCheckResult {
    pub fn new() -> Self {
        Self {
            successes: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_success(&mut self, success: String) {
        self.successes.push(success);
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn is_consistent(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn log_results(&self) {
        if self.is_consistent() {
            info!("✅ Database consistency check passed");
        } else {
            error!("❌ Database consistency check failed with {} errors", self.errors.len());
        }

        for error in &self.errors {
            error!("  ❌ {}", error);
        }
    }
}