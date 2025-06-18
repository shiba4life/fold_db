//! Type definitions for the unified transform execution system.
//!
//! This module contains all the core types used throughout the transform execution
//! system, including input/output types, state representations, and configuration
//! structures.

use crate::schema::types::Transform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Unique identifier for a transform.
pub type TransformId = String;

/// Unique identifier for an execution job.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub Uuid);

impl JobId {
    /// Creates a new job ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Creates a job ID from a string.
    pub fn from_string(s: &str) -> Result<Self, uuid::Error> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Transform definition used for registration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformDefinition {
    /// Unique identifier for the transform
    pub id: TransformId,
    /// The transform logic and configuration
    pub transform: Transform,
    /// Input field names that this transform depends on
    pub inputs: Vec<String>,
    /// Additional metadata for the transform
    pub metadata: HashMap<String, String>,
}

/// Input data for transform execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformInput {
    /// Input values keyed by field name
    pub values: HashMap<String, serde_json::Value>,
    /// Execution context and metadata
    pub context: ExecutionContext,
}

/// Output data from transform execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOutput {
    /// The computed result value
    pub value: serde_json::Value,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Context information for transform execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Schema name for the execution
    pub schema_name: String,
    /// Field name being computed
    pub field_name: String,
    /// Atom reference ID
    pub atom_ref: Option<String>,
    /// Execution timestamp
    pub timestamp: SystemTime,
    /// Additional context data
    pub additional_data: HashMap<String, String>,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            schema_name: String::new(),
            field_name: String::new(),
            atom_ref: None,
            timestamp: SystemTime::now(),
            additional_data: HashMap::new(),
        }
    }
}

/// Metadata about transform execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Execution duration
    pub duration: Duration,
    /// Number of input values processed
    pub input_count: usize,
    /// Transform version used
    pub transform_version: Option<String>,
    /// Additional execution metrics
    pub metrics: HashMap<String, f64>,
}

/// Metadata about a registered transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetadata {
    /// Transform ID
    pub id: TransformId,
    /// Human-readable name
    pub name: String,
    /// Transform description
    pub description: String,
    /// Input field names
    pub inputs: Vec<String>,
    /// Output field name
    pub output: String,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Last modification timestamp
    pub updated_at: SystemTime,
    /// Current execution status
    pub status: TransformStatus,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Status of a transform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformStatus {
    /// Transform is registered and ready for execution
    Ready,
    /// Transform is currently executing
    Executing,
    /// Transform execution failed
    Failed,
    /// Transform is disabled
    Disabled,
    /// Transform is being updated
    Updating,
}

impl fmt::Display for TransformStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransformStatus::Ready => write!(f, "Ready"),
            TransformStatus::Executing => write!(f, "Executing"),
            TransformStatus::Failed => write!(f, "Failed"),
            TransformStatus::Disabled => write!(f, "Disabled"),
            TransformStatus::Updating => write!(f, "Updating"),
        }
    }
}

/// Parameters for updating a transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformUpdate {
    /// New transform logic (optional)
    pub transform: Option<Transform>,
    /// New input fields (optional)
    pub inputs: Option<Vec<String>>,
    /// New metadata (optional)
    pub metadata: Option<HashMap<String, String>>,
    /// New status (optional)
    pub status: Option<TransformStatus>,
}

/// Transform registration parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRegistration {
    /// Transform definition
    pub definition: TransformDefinition,
    /// Input atom references
    pub input_arefs: Vec<String>,
    /// Input field names
    pub input_names: Vec<String>,
    /// Fields that trigger this transform
    pub trigger_fields: Vec<String>,
    /// Output atom reference
    pub output_aref: String,
    /// Schema name
    pub schema_name: String,
    /// Field name
    pub field_name: String,
}

/// Current status of the execution queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    /// Number of jobs pending execution
    pub pending: usize,
    /// Number of jobs currently executing
    pub running: usize,
    /// Number of completed jobs
    pub completed: usize,
    /// Number of failed jobs
    pub failed: usize,
    /// Queue capacity
    pub capacity: usize,
    /// Average execution time
    pub avg_execution_time: Duration,
}

impl Default for QueueStatus {
    fn default() -> Self {
        Self {
            pending: 0,
            running: 0,
            completed: 0,
            failed: 0,
            capacity: 100,
            avg_execution_time: Duration::from_millis(0),
        }
    }
}

/// Execution job information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionJob {
    /// Unique job identifier
    pub id: JobId,
    /// Transform ID to execute
    pub transform_id: TransformId,
    /// Input data for execution
    pub input: TransformInput,
    /// Current job status
    pub status: JobStatus,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Start time (if started)
    pub started_at: Option<SystemTime>,
    /// Completion time (if completed)
    pub completed_at: Option<SystemTime>,
    /// Retry count
    pub retry_count: u32,
    /// Error message (if failed)
    pub error_message: Option<String>,
}

/// Status of an execution job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job is queued for execution
    Queued,
    /// Job is currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed with error
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Job is waiting for retry
    Retrying,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Queued => write!(f, "Queued"),
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Completed => write!(f, "Completed"),
            JobStatus::Failed => write!(f, "Failed"),
            JobStatus::Cancelled => write!(f, "Cancelled"),
            JobStatus::Retrying => write!(f, "Retrying"),
        }
    }
}

/// Filter criteria for listing transforms.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TransformFilter {
    /// Filter by status
    pub status: Option<TransformStatus>,
    /// Filter by schema name
    pub schema_name: Option<String>,
    /// Filter by field name pattern
    pub field_pattern: Option<String>,
    /// Filter by creation date range
    pub created_after: Option<SystemTime>,
    /// Filter by creation date range
    pub created_before: Option<SystemTime>,
}


/// Configuration for retry behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Performance metrics for transforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformMetrics {
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average execution time
    pub avg_execution_time: Duration,
    /// Minimum execution time
    pub min_execution_time: Duration,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Last execution timestamp
    pub last_execution: Option<SystemTime>,
}

impl Default for TransformMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            avg_execution_time: Duration::from_millis(0),
            min_execution_time: Duration::from_millis(0),
            max_execution_time: Duration::from_millis(0),
            last_execution: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_id_creation() {
        let job_id = JobId::new();
        assert!(!job_id.to_string().is_empty());
        
        let job_id2 = JobId::new();
        assert_ne!(job_id, job_id2);
    }

    #[test]
    fn test_job_id_from_string() {
        let uuid_str = "550e8400-e29b-41d4-a716-446655440000";
        let job_id = JobId::from_string(uuid_str).unwrap();
        assert_eq!(job_id.to_string(), uuid_str);
    }

    #[test]
    fn test_transform_status_display() {
        assert_eq!(TransformStatus::Ready.to_string(), "Ready");
        assert_eq!(TransformStatus::Executing.to_string(), "Executing");
        assert_eq!(TransformStatus::Failed.to_string(), "Failed");
    }

    #[test]
    fn test_job_status_display() {
        assert_eq!(JobStatus::Queued.to_string(), "Queued");
        assert_eq!(JobStatus::Running.to_string(), "Running");
        assert_eq!(JobStatus::Completed.to_string(), "Completed");
    }

    #[test]
    fn test_execution_context_default() {
        let context = ExecutionContext::default();
        assert!(context.schema_name.is_empty());
        assert!(context.field_name.is_empty());
        assert!(context.atom_ref.is_none());
    }

    #[test]
    fn test_queue_status_default() {
        let status = QueueStatus::default();
        assert_eq!(status.pending, 0);
        assert_eq!(status.running, 0);
        assert_eq!(status.completed, 0);
        assert_eq!(status.failed, 0);
        assert_eq!(status.capacity, 100);
    }
}