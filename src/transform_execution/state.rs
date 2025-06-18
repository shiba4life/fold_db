//! State management for the unified transform execution system.
//!
//! This module provides centralized state management for transforms, including
//! execution state, history tracking, and persistence. It serves as the single
//! source of truth for all transform state information.

use super::error::{TransformError, TransformResult};
use super::types::{
    ExecutionMetadata, JobId, JobStatus, TransformId, TransformMetrics,
    TransformStatus,
};
use crate::db_operations::DbOperations;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

/// Current state of a transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformState {
    /// Transform ID
    pub transform_id: TransformId,
    /// Current status
    pub status: TransformStatus,
    /// Last execution timestamp
    pub last_execution: Option<SystemTime>,
    /// Last execution result
    pub last_result: Option<String>,
    /// Last error message (if any)
    pub last_error: Option<String>,
    /// Number of successful executions
    pub success_count: u64,
    /// Number of failed executions
    pub failure_count: u64,
    /// Performance metrics
    pub metrics: TransformMetrics,
    /// Additional state data
    pub metadata: HashMap<String, String>,
    /// State creation timestamp
    pub created_at: SystemTime,
    /// State last update timestamp
    pub updated_at: SystemTime,
}

impl TransformState {
    /// Creates a new transform state.
    pub fn new(transform_id: TransformId) -> Self {
        let now = SystemTime::now();
        Self {
            transform_id,
            status: TransformStatus::Ready,
            last_execution: None,
            last_result: None,
            last_error: None,
            success_count: 0,
            failure_count: 0,
            metrics: TransformMetrics::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Updates the state after a successful execution.
    pub fn record_success(&mut self, execution_time: Duration, result: Option<String>) {
        self.status = TransformStatus::Ready;
        self.last_execution = Some(SystemTime::now());
        self.last_result = result;
        self.last_error = None;
        self.success_count += 1;
        self.updated_at = SystemTime::now();
        
        // Update metrics
        self.metrics.total_executions += 1;
        self.metrics.successful_executions += 1;
        self.metrics.last_execution = self.last_execution;
        
        // Update execution time metrics
        if self.metrics.total_executions == 1 {
            self.metrics.min_execution_time = execution_time;
            self.metrics.max_execution_time = execution_time;
            self.metrics.avg_execution_time = execution_time;
        } else {
            if execution_time < self.metrics.min_execution_time {
                self.metrics.min_execution_time = execution_time;
            }
            if execution_time > self.metrics.max_execution_time {
                self.metrics.max_execution_time = execution_time;
            }
            
            // Update average execution time
            let total_time = self.metrics.avg_execution_time.as_nanos() as u64 
                * (self.metrics.total_executions - 1) + execution_time.as_nanos() as u64;
            self.metrics.avg_execution_time = Duration::from_nanos(total_time / self.metrics.total_executions);
        }
    }

    /// Updates the state after a failed execution.
    pub fn record_failure(&mut self, error_message: String) {
        self.status = TransformStatus::Failed;
        self.last_execution = Some(SystemTime::now());
        self.last_result = None;
        self.last_error = Some(error_message);
        self.failure_count += 1;
        self.updated_at = SystemTime::now();
        
        // Update metrics
        self.metrics.total_executions += 1;
        self.metrics.failed_executions += 1;
        self.metrics.last_execution = self.last_execution;
    }

    /// Sets the transform status.
    pub fn set_status(&mut self, status: TransformStatus) {
        self.status = status;
        self.updated_at = SystemTime::now();
    }

    /// Gets the success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.success_count + self.failure_count;
        if total == 0 {
            0.0
        } else {
            (self.success_count as f64 / total as f64) * 100.0
        }
    }
}

/// Record of a transform execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Unique execution ID
    pub execution_id: String,
    /// Transform ID that was executed
    pub transform_id: TransformId,
    /// Job ID (if queued execution)
    pub job_id: Option<JobId>,
    /// Execution start time
    pub started_at: SystemTime,
    /// Execution completion time
    pub completed_at: Option<SystemTime>,
    /// Execution duration
    pub duration: Option<Duration>,
    /// Execution status
    pub status: JobStatus,
    /// Input data summary
    pub input_summary: String,
    /// Output result
    pub result: Option<serde_json::Value>,
    /// Error message (if failed)
    pub error_message: Option<String>,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

impl ExecutionRecord {
    /// Creates a new execution record.
    pub fn new(transform_id: TransformId, input_summary: String) -> Self {
        Self {
            execution_id: uuid::Uuid::new_v4().to_string(),
            transform_id,
            job_id: None,
            started_at: SystemTime::now(),
            completed_at: None,
            duration: None,
            status: JobStatus::Running,
            input_summary,
            result: None,
            error_message: None,
            metadata: ExecutionMetadata {
                duration: Duration::from_millis(0),
                input_count: 0,
                transform_version: None,
                metrics: HashMap::new(),
            },
        }
    }

    /// Marks the execution as completed successfully.
    pub fn complete_success(&mut self, result: serde_json::Value, metadata: ExecutionMetadata) {
        let now = SystemTime::now();
        self.completed_at = Some(now);
        self.duration = now.duration_since(self.started_at).ok();
        self.status = JobStatus::Completed;
        self.result = Some(result);
        self.metadata = metadata;
    }

    /// Marks the execution as failed.
    pub fn complete_failure(&mut self, error_message: String) {
        let now = SystemTime::now();
        self.completed_at = Some(now);
        self.duration = now.duration_since(self.started_at).ok();
        self.status = JobStatus::Failed;
        self.error_message = Some(error_message);
    }
}

/// Inner state store implementation.
pub(crate) struct StateStoreInner {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// In-memory state cache
    state_cache: RwLock<HashMap<TransformId, TransformState>>,
    /// Execution history cache
    history_cache: RwLock<HashMap<TransformId, Vec<ExecutionRecord>>>,
    /// Maximum history entries per transform
    max_history_entries: usize,
}

impl StateStoreInner {
    /// Creates a new state store inner.
    pub(crate) fn new(db_ops: Arc<DbOperations>) -> Self {
        Self {
            db_ops,
            state_cache: RwLock::new(HashMap::new()),
            history_cache: RwLock::new(HashMap::new()),
            max_history_entries: 1000, // Default limit
        }
    }

    /// Loads state from database into cache.
    pub(crate) fn load_state(&self, transform_id: &TransformId) -> TransformResult<Option<TransformState>> {
        match self.db_ops.get_from_tree(&self.db_ops.transforms_tree, &format!("state_{}", transform_id)) {
            Ok(state) => Ok(state),
            Err(e) => {
                error!("Failed to load transform state for {}: {}", transform_id, e);
                Err(TransformError::state(
                    format!("Failed to load state: {}", e),
                    "load_state"
                ))
            }
        }
    }

    /// Saves state to database.
    pub(crate) fn save_state(&self, state: &TransformState) -> TransformResult<()> {
        match self.db_ops.store_in_tree(
            &self.db_ops.transforms_tree,
            &format!("state_{}", state.transform_id),
            state
        ) {
            Ok(_) => {
                debug!("Saved state for transform {}", state.transform_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to save transform state for {}: {}", state.transform_id, e);
                Err(TransformError::state(
                    format!("Failed to save state: {}", e),
                    "save_state"
                ))
            }
        }
    }

    /// Loads execution history from database.
    fn load_history(&self, transform_id: &TransformId) -> TransformResult<Vec<ExecutionRecord>> {
        match self.db_ops.get_from_tree(&self.db_ops.transforms_tree, &format!("history_{}", transform_id)) {
            Ok(history) => Ok(history.unwrap_or_default()),
            Err(e) => {
                warn!("Failed to load execution history for {}: {}", transform_id, e);
                Ok(Vec::new()) // Return empty history on error
            }
        }
    }

    /// Saves execution history to database.
    fn save_history(&self, transform_id: &TransformId, history: &Vec<ExecutionRecord>) -> TransformResult<()> {
        match self.db_ops.store_in_tree(
            &self.db_ops.transforms_tree,
            &format!("history_{}", transform_id),
            history
        ) {
            Ok(_) => {
                debug!("Saved execution history for transform {}", transform_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to save execution history for {}: {}", transform_id, e);
                Err(TransformError::state(
                    format!("Failed to save history: {}", e),
                    "save_history"
                ))
            }
        }
    }
}

/// Centralized state store for transform execution.
pub struct TransformStateStore {
    inner: Arc<StateStoreInner>,
}

impl TransformStateStore {
    /// Creates a new transform state store.
    pub fn new(db_ops: Arc<DbOperations>) -> TransformResult<Self> {
        let inner = Arc::new(StateStoreInner::new(db_ops));
        Ok(Self { inner })
    }

    /// Gets the inner state store for sharing.
    pub(crate) fn inner(&self) -> Arc<StateStoreInner> {
        Arc::clone(&self.inner)
    }

    /// Gets the current state of a transform.
    pub fn get_state(&self, transform_id: TransformId) -> TransformResult<TransformState> {
        // Try cache first
        {
            let cache = self.inner.state_cache.read().unwrap();
            if let Some(state) = cache.get(&transform_id) {
                return Ok(state.clone());
            }
        }

        // Load from database
        match self.inner.load_state(&transform_id)? {
            Some(state) => {
                // Cache the loaded state
                {
                    let mut cache = self.inner.state_cache.write().unwrap();
                    cache.insert(transform_id.clone(), state.clone());
                }
                Ok(state)
            }
            None => {
                // Create new state
                let state = TransformState::new(transform_id.clone());
                self.set_state(transform_id, state.clone())?;
                Ok(state)
            }
        }
    }

    /// Sets the state of a transform.
    pub fn set_state(&self, transform_id: TransformId, state: TransformState) -> TransformResult<()> {
        // Save to database
        self.inner.save_state(&state)?;

        // Update cache
        {
            let mut cache = self.inner.state_cache.write().unwrap();
            cache.insert(transform_id, state);
        }

        Ok(())
    }

    /// Records a successful execution.
    pub fn record_success(
        &self,
        transform_id: TransformId,
        execution_time: Duration,
        result: Option<String>,
    ) -> TransformResult<()> {
        let mut state = self.get_state(transform_id.clone())?;
        state.record_success(execution_time, result);
        self.set_state(transform_id, state)
    }

    /// Records a failed execution.
    pub fn record_failure(
        &self,
        transform_id: TransformId,
        error_message: String,
    ) -> TransformResult<()> {
        let mut state = self.get_state(transform_id.clone())?;
        state.record_failure(error_message);
        self.set_state(transform_id, state)
    }

    /// Records an execution in the history.
    pub fn record_execution(&self, record: ExecutionRecord) -> TransformResult<()> {
        let transform_id = record.transform_id.clone();
        
        // Load current history
        let mut history = {
            let cache = self.inner.history_cache.read().unwrap();
            cache.get(&transform_id).cloned().unwrap_or_else(|| {
                self.inner.load_history(&transform_id).unwrap_or_default()
            })
        };

        // Add new record
        history.push(record);

        // Trim history if it exceeds the limit
        if history.len() > self.inner.max_history_entries {
            let skip_count = history.len() - self.inner.max_history_entries;
            history = history.into_iter().skip(skip_count).collect();
        }

        // Save to database
        self.inner.save_history(&transform_id, &history)?;

        // Update cache
        {
            let mut cache = self.inner.history_cache.write().unwrap();
            cache.insert(transform_id, history);
        }

        Ok(())
    }

    /// Gets the execution history for a transform.
    pub fn get_execution_history(
        &self,
        transform_id: TransformId,
        limit: Option<usize>,
    ) -> TransformResult<Vec<ExecutionRecord>> {
        // Try cache first
        let history = {
            let cache = self.inner.history_cache.read().unwrap();
            cache.get(&transform_id).cloned()
        };

        let history = match history {
            Some(cached_history) => cached_history,
            None => {
                // Load from database
                let loaded_history = self.inner.load_history(&transform_id)?;
                
                // Cache the loaded history
                {
                    let mut cache = self.inner.history_cache.write().unwrap();
                    cache.insert(transform_id.clone(), loaded_history.clone());
                }
                
                loaded_history
            }
        };

        // Apply limit if specified
        let result = match limit {
            Some(limit_count) => {
                if history.len() > limit_count {
                    let skip_count = history.len() - limit_count;
                    history.into_iter().skip(skip_count).collect()
                } else {
                    history
                }
            }
            None => history,
        };

        Ok(result)
    }

    /// Clears the state for a transform.
    pub fn clear_state(&self, transform_id: TransformId) -> TransformResult<()> {
        // Remove from cache
        {
            let mut state_cache = self.inner.state_cache.write().unwrap();
            state_cache.remove(&transform_id);
            
            let mut history_cache = self.inner.history_cache.write().unwrap();
            history_cache.remove(&transform_id);
        }

        // Remove from database
        let state_key = format!("state_{}", transform_id);
        let history_key = format!("history_{}", transform_id);
        
        if let Err(e) = self.inner.db_ops.transforms_tree.remove(state_key.as_bytes()) {
            warn!("Failed to remove state from database: {}", e);
        }
        
        if let Err(e) = self.inner.db_ops.transforms_tree.remove(history_key.as_bytes()) {
            warn!("Failed to remove history from database: {}", e);
        }

        info!("Cleared state and history for transform {}", transform_id);
        Ok(())
    }

    /// Gets states for all transforms.
    pub fn list_all_states(&self) -> TransformResult<Vec<TransformState>> {
        // For now, return empty list. In a real implementation, we'd need
        // to maintain an index of transform IDs or scan the database properly.
        Ok(Vec::new())
    }

    /// Gets metrics for all transforms.
    pub fn get_aggregate_metrics(&self) -> TransformResult<TransformMetrics> {
        let states = self.list_all_states()?;
        
        if states.is_empty() {
            return Ok(TransformMetrics::default());
        }
        
        let mut aggregate = TransformMetrics::default();
        let mut min_time = None;
        let mut max_time = Duration::from_millis(0);
        let mut total_avg_time_nanos = 0u64;
        
        for state in &states {
            let metrics = &state.metrics;
            aggregate.total_executions += metrics.total_executions;
            aggregate.successful_executions += metrics.successful_executions;
            aggregate.failed_executions += metrics.failed_executions;
            
            if metrics.total_executions > 0 {
                // Track min time
                if min_time.is_none() || metrics.min_execution_time < min_time.unwrap() {
                    min_time = Some(metrics.min_execution_time);
                }
                
                // Track max time
                if metrics.max_execution_time > max_time {
                    max_time = metrics.max_execution_time;
                }
                
                // Accumulate average times
                total_avg_time_nanos += metrics.avg_execution_time.as_nanos() as u64;
            }
            
            // Track latest execution
            if let Some(last_exec) = metrics.last_execution {
                if aggregate.last_execution.is_none() || last_exec > aggregate.last_execution.unwrap() {
                    aggregate.last_execution = Some(last_exec);
                }
            }
        }
        
        aggregate.min_execution_time = min_time.unwrap_or(Duration::from_millis(0));
        aggregate.max_execution_time = max_time;
        
        // Calculate overall average
        if !states.is_empty() {
            aggregate.avg_execution_time = Duration::from_nanos(total_avg_time_nanos / states.len() as u64);
        }
        
        Ok(aggregate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_db_ops() -> Arc<DbOperations> {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = sled::open(&db_path).unwrap();
        Arc::new(DbOperations::new(db).unwrap())
    }

    #[test]
    fn test_transform_state_creation() {
        let state = TransformState::new("test_transform".to_string());
        assert_eq!(state.transform_id, "test_transform");
        assert_eq!(state.status, TransformStatus::Ready);
        assert_eq!(state.success_count, 0);
        assert_eq!(state.failure_count, 0);
    }

    #[test]
    fn test_state_store_creation() {
        let db_ops = create_test_db_ops();
        let store = TransformStateStore::new(db_ops);
        assert!(store.is_ok());
    }

    #[test]
    fn test_get_set_state() {
        let db_ops = create_test_db_ops();
        let store = TransformStateStore::new(db_ops).unwrap();
        
        let transform_id = "test_transform".to_string();
        
        // Get initial state (should create new)
        let state = store.get_state(transform_id.clone()).unwrap();
        assert_eq!(state.transform_id, transform_id);
        assert_eq!(state.status, TransformStatus::Ready);
        
        // Modify and set state
        let mut modified_state = state;
        modified_state.status = TransformStatus::Executing;
        store.set_state(transform_id.clone(), modified_state).unwrap();
        
        // Get state again and verify
        let retrieved_state = store.get_state(transform_id).unwrap();
        assert_eq!(retrieved_state.status, TransformStatus::Executing);
    }

    #[test]
    fn test_record_success() {
        let db_ops = create_test_db_ops();
        let store = TransformStateStore::new(db_ops).unwrap();
        
        let transform_id = "test_transform".to_string();
        let execution_time = Duration::from_millis(100);
        
        store.record_success(transform_id.clone(), execution_time, Some("result".to_string())).unwrap();
        
        let state = store.get_state(transform_id).unwrap();
        assert_eq!(state.success_count, 1);
        assert_eq!(state.status, TransformStatus::Ready);
        assert!(state.last_execution.is_some());
        assert_eq!(state.last_result, Some("result".to_string()));
    }

    #[test]
    fn test_record_failure() {
        let db_ops = create_test_db_ops();
        let store = TransformStateStore::new(db_ops).unwrap();
        
        let transform_id = "test_transform".to_string();
        let error_message = "Test error".to_string();
        
        store.record_failure(transform_id.clone(), error_message.clone()).unwrap();
        
        let state = store.get_state(transform_id).unwrap();
        assert_eq!(state.failure_count, 1);
        assert_eq!(state.status, TransformStatus::Failed);
        assert_eq!(state.last_error, Some(error_message));
    }

    #[test]
    fn test_execution_record() {
        let mut record = ExecutionRecord::new("test_transform".to_string(), "input summary".to_string());
        assert_eq!(record.status, JobStatus::Running);
        
        let result = serde_json::json!({"value": 42});
        let metadata = ExecutionMetadata {
            duration: Duration::from_millis(100),
            input_count: 1,
            transform_version: Some("1.0".to_string()),
            metrics: HashMap::new(),
        };
        
        record.complete_success(result.clone(), metadata);
        assert_eq!(record.status, JobStatus::Completed);
        assert_eq!(record.result, Some(result));
        assert!(record.completed_at.is_some());
    }

    #[test]
    fn test_success_rate() {
        let mut state = TransformState::new("test_transform".to_string());
        assert_eq!(state.success_rate(), 0.0);
        
        state.success_count = 7;
        state.failure_count = 3;
        assert_eq!(state.success_rate(), 70.0);
    }
}