//! Core execution engine for the unified transform execution system.
//!
//! This module contains the main execution engine that coordinates transform
//! execution, registration, queuing, and orchestration. It integrates with
//! the existing transform system while providing a unified interface.

use super::config::ConfigLoaderInner;
use super::error::{TransformError, TransformResult};
use super::state::{ExecutionRecord, StateStoreInner, TransformState};
use super::types::{
    ExecutionJob, ExecutionMetadata, JobId, JobStatus, QueueStatus,
    TransformDefinition, TransformId, TransformInput, TransformMetadata, TransformOutput,
    TransformStatus, TransformUpdate,
};
use crate::db_operations::DbOperations;
use crate::schema::types::{Transform};
use crate::transform::executor::TransformExecutor as LegacyTransformExecutor;
use log::{error, info, warn};
use serde_json::Value as JsonValue;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// Core execution engine for transforms.
#[allow(dead_code)]
pub struct TransformEngine {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// State store reference
    state_store: Arc<StateStoreInner>,
    /// Configuration loader reference
    config_loader: Arc<ConfigLoaderInner>,
    /// Transform executor
    executor: TransformExecutor,
    /// Transform orchestrator
    orchestrator: TransformOrchestrator,
    /// Transform registration manager
    registration: TransformRegistration,
}

impl TransformEngine {
    /// Creates a new transform engine.
    pub(crate) fn new(
        db_ops: Arc<DbOperations>,
        state_store: Arc<StateStoreInner>,
        config_loader: Arc<ConfigLoaderInner>,
    ) -> TransformResult<Self> {
        let executor = TransformExecutor::new(Arc::clone(&db_ops), Arc::clone(&state_store))?;
        let orchestrator = TransformOrchestrator::new(
            Arc::clone(&db_ops),
            Arc::clone(&state_store),
            Arc::clone(&config_loader),
        )?;
        let registration = TransformRegistration::new(Arc::clone(&db_ops), Arc::clone(&state_store))?;

        Ok(Self {
            db_ops,
            state_store,
            config_loader,
            executor,
            orchestrator,
            registration,
        })
    }

    /// Registers a new transform.
    pub fn register_transform(&self, definition: TransformDefinition) -> TransformResult<TransformId> {
        self.registration.register_transform(definition)
    }

    /// Executes a transform synchronously.
    pub fn execute_transform(
        &self,
        id: TransformId,
        input: TransformInput,
    ) -> TransformResult<TransformOutput> {
        self.executor.execute_transform(id, input)
    }

    /// Lists all registered transforms.
    pub fn list_transforms(&self, filter: Option<&str>) -> Vec<TransformMetadata> {
        self.registration.list_transforms(filter)
    }

    /// Updates an existing transform.
    pub fn update_transform(&self, id: TransformId, update: TransformUpdate) -> TransformResult<()> {
        self.registration.update_transform(id, update)
    }

    /// Removes a transform from the system.
    pub fn remove_transform(&self, id: TransformId) -> TransformResult<()> {
        self.registration.remove_transform(id)
    }

    /// Enqueues a transform for asynchronous execution.
    pub fn enqueue_execution(&self, id: TransformId, input: TransformInput) -> TransformResult<JobId> {
        self.orchestrator.enqueue_execution(id, input)
    }

    /// Gets the current queue status.
    pub fn get_queue_status(&self) -> QueueStatus {
        self.orchestrator.get_queue_status()
    }

    /// Retries a failed job.
    pub fn retry_failed(&self, job_id: JobId) -> TransformResult<()> {
        self.orchestrator.retry_failed(job_id)
    }
}

/// Transform executor for individual transform executions.
pub struct TransformExecutor {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// State store reference
    state_store: Arc<StateStoreInner>,
}

impl TransformExecutor {
    /// Creates a new transform executor.
    pub(crate) fn new(
        db_ops: Arc<DbOperations>,
        state_store: Arc<StateStoreInner>,
    ) -> TransformResult<Self> {
        Ok(Self { db_ops, state_store })
    }

    /// Executes a transform with the given input.
    pub fn execute_transform(
        &self,
        transform_id: TransformId,
        input: TransformInput,
    ) -> TransformResult<TransformOutput> {
        let start_time = Instant::now();
        info!("🚀 Starting transform execution: {}", transform_id);

        // Load transform from database
        let transform = self.load_transform(&transform_id)?;

        // Create execution record
        let input_summary = format!("{} inputs", input.values.len());
        let mut execution_record = ExecutionRecord::new(transform_id.clone(), input_summary);

        // Update transform state to executing
        if let Ok(mut state) = self.load_transform_state(&transform_id) {
            state.set_status(TransformStatus::Executing);
            if let Err(e) = self.save_transform_state(&state) {
                warn!("Failed to update transform state: {}", e);
            }
        }

        // Execute the transform
        let result = self.execute_transform_logic(&transform, &input);

        let execution_time = start_time.elapsed();

        match result {
            Ok(output) => {
                info!("✅ Transform execution completed successfully: {}", transform_id);
                
                // Record success in state
                if let Ok(mut state) = self.load_transform_state(&transform_id) {
                    state.record_success(execution_time, Some(output.value.to_string()));
                    if let Err(e) = self.save_transform_state(&state) {
                        warn!("Failed to update transform state after success: {}", e);
                    }
                }

                // Complete execution record
                execution_record.complete_success(output.value.clone(), output.metadata.clone());
                if let Err(e) = self.save_execution_record(execution_record) {
                    warn!("Failed to save execution record: {}", e);
                }

                Ok(output)
            }
            Err(error) => {
                error!("❌ Transform execution failed: {} - {}", transform_id, error);
                
                // Record failure in state
                if let Ok(mut state) = self.load_transform_state(&transform_id) {
                    state.record_failure(error.to_string());
                    if let Err(e) = self.save_transform_state(&state) {
                        warn!("Failed to update transform state after failure: {}", e);
                    }
                }

                // Complete execution record with failure
                execution_record.complete_failure(error.to_string());
                if let Err(e) = self.save_execution_record(execution_record) {
                    warn!("Failed to save execution record: {}", e);
                }

                Err(error)
            }
        }
    }

    /// Executes the actual transform logic.
    fn execute_transform_logic(
        &self,
        transform: &Transform,
        input: &TransformInput,
    ) -> TransformResult<TransformOutput> {
        let start_time = Instant::now();

        // Convert input to legacy format
        let input_values: HashMap<String, JsonValue> = input.values.clone();

        // Execute using the legacy transform executor
        let result = LegacyTransformExecutor::execute_transform(transform, input_values)
            .map_err(|e| TransformError::SchemaError {
                message: e.to_string(),
            })?;

        let execution_time = start_time.elapsed();

        // Create output metadata
        let metadata = ExecutionMetadata {
            duration: execution_time,
            input_count: input.values.len(),
            transform_version: None,
            metrics: HashMap::new(),
        };

        Ok(TransformOutput {
            value: result,
            metadata,
        })
    }

    /// Loads a transform from the database.
    fn load_transform(&self, transform_id: &TransformId) -> TransformResult<Transform> {
        self.db_ops
            .get_transform(transform_id)
            .map_err(|e| TransformError::database(e.to_string(), "get_transform"))?
            .ok_or_else(|| TransformError::not_found(transform_id.clone(), "execute_transform"))
    }

    /// Loads transform state.
    fn load_transform_state(&self, transform_id: &TransformId) -> TransformResult<TransformState> {
        self.state_store
            .load_state(transform_id)
            .and_then(|opt| opt.ok_or_else(|| {
                TransformError::state("Transform state not found".to_string(), "load_state")
            }))
    }

    /// Saves transform state.
    fn save_transform_state(&self, state: &TransformState) -> TransformResult<()> {
        self.state_store.save_state(state)
    }

    /// Saves execution record.
    fn save_execution_record(&self, record: ExecutionRecord) -> TransformResult<()> {
        let transform_id = record.transform_id.clone();
        let history_key = format!("history_{}", transform_id);
        
        // Load existing history
        let mut history: Vec<ExecutionRecord> = self.db_ops
            .get_from_tree(&self.db_ops.transforms_tree, &history_key)
            .unwrap_or(None)
            .unwrap_or_default();
        
        // Add new record
        history.push(record);
        
        // Trim if necessary (keep last 1000 records)
        if history.len() > 1000 {
            let skip_count = history.len() - 1000;
            history = history.into_iter().skip(skip_count).collect();
        }
        
        // Save back to database
        self.db_ops
            .store_in_tree(&self.db_ops.transforms_tree, &history_key, &history)
            .map_err(|e| TransformError::database(e.to_string(), "save_execution_record"))
    }
}

/// Transform orchestrator for queue management and async execution.
#[allow(dead_code)]
pub struct TransformOrchestrator {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// State store reference
    state_store: Arc<StateStoreInner>,
    /// Configuration loader reference
    config_loader: Arc<ConfigLoaderInner>,
    /// Execution queue
    queue: Arc<Mutex<VecDeque<ExecutionJob>>>,
    /// Job tracking
    jobs: Arc<RwLock<HashMap<JobId, ExecutionJob>>>,
    /// Queue statistics
    stats: Arc<RwLock<QueueStatus>>,
    /// Executor reference
    executor: Arc<TransformExecutor>,
}

impl TransformOrchestrator {
    /// Creates a new transform orchestrator.
    pub(crate) fn new(
        db_ops: Arc<DbOperations>,
        state_store: Arc<StateStoreInner>,
        config_loader: Arc<ConfigLoaderInner>,
    ) -> TransformResult<Self> {
        let executor = Arc::new(TransformExecutor::new(
            Arc::clone(&db_ops),
            Arc::clone(&state_store),
        )?);

        let orchestrator = Self {
            db_ops,
            state_store,
            config_loader,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(QueueStatus::default())),
            executor,
        };

        // Start the queue processing thread
        orchestrator.start_queue_processor();

        Ok(orchestrator)
    }

    /// Enqueues a transform for execution.
    pub fn enqueue_execution(&self, transform_id: TransformId, input: TransformInput) -> TransformResult<JobId> {
        let job_id = JobId::new();
        let job = ExecutionJob {
            id: job_id.clone(),
            transform_id,
            input,
            status: JobStatus::Queued,
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            retry_count: 0,
            error_message: None,
        };

        // Add to queue
        {
            let mut queue = self.queue.lock().unwrap();
            queue.push_back(job.clone());
        }

        // Track job
        {
            let mut jobs = self.jobs.write().unwrap();
            jobs.insert(job_id.clone(), job.clone());
        }

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.pending += 1;
        }

        info!("📥 Enqueued transform execution: {} (job: {})", job.transform_id, job_id);
        Ok(job_id)
    }

    /// Gets the current queue status.
    pub fn get_queue_status(&self) -> QueueStatus {
        let stats = self.stats.read().unwrap();
        stats.clone()
    }

    /// Retries a failed job.
    pub fn retry_failed(&self, job_id: JobId) -> TransformResult<()> {
        let mut jobs = self.jobs.write().unwrap();
        
        if let Some(job) = jobs.get_mut(&job_id) {
            if job.status == JobStatus::Failed {
                job.status = JobStatus::Retrying;
                job.retry_count += 1;
                job.error_message = None;

                // Re-enqueue the job
                let mut queue = self.queue.lock().unwrap();
                queue.push_back(job.clone());

                // Update stats
                let mut stats = self.stats.write().unwrap();
                stats.pending += 1;
                stats.failed = stats.failed.saturating_sub(1);

                info!("🔄 Retrying failed job: {} (attempt {})", job_id, job.retry_count);
                Ok(())
            } else {
                Err(TransformError::queue(
                    "Job is not in failed state".to_string(),
                    "retry_failed"
                ))
            }
        } else {
            Err(TransformError::not_found(job_id.to_string(), "retry_failed"))
        }
    }

    /// Starts the queue processor thread.
    fn start_queue_processor(&self) {
        let queue = Arc::clone(&self.queue);
        let jobs = Arc::clone(&self.jobs);
        let stats = Arc::clone(&self.stats);
        let executor = Arc::clone(&self.executor);

        thread::spawn(move || {
            info!("🏃 Queue processor started");
            
            loop {
                // Get next job from queue
                let job_opt = {
                    let mut queue = queue.lock().unwrap();
                    queue.pop_front()
                };

                if let Some(mut job) = job_opt {
                    // Update job status
                    job.status = JobStatus::Running;
                    job.started_at = Some(SystemTime::now());

                    // Update stats
                    {
                        let mut stats = stats.write().unwrap();
                        stats.pending = stats.pending.saturating_sub(1);
                        stats.running += 1;
                    }

                    // Update job tracking
                    {
                        let mut jobs = jobs.write().unwrap();
                        jobs.insert(job.id.clone(), job.clone());
                    }

                    info!("🔥 Processing job: {} for transform: {}", job.id, job.transform_id);

                    // Execute the transform
                    let result = executor.execute_transform(job.transform_id.clone(), job.input.clone());

                    // Update job with result
                    job.completed_at = Some(SystemTime::now());
                    
                    match result {
                        Ok(_) => {
                            job.status = JobStatus::Completed;
                            
                            // Update stats
                            {
                                let mut stats = stats.write().unwrap();
                                stats.running = stats.running.saturating_sub(1);
                                stats.completed += 1;
                            }
                            
                            info!("✅ Job completed successfully: {}", job.id);
                        }
                        Err(error) => {
                            job.status = JobStatus::Failed;
                            job.error_message = Some(error.to_string());
                            
                            // Update stats
                            {
                                let mut stats = stats.write().unwrap();
                                stats.running = stats.running.saturating_sub(1);
                                stats.failed += 1;
                            }
                            
                            error!("❌ Job failed: {} - {}", job.id, error);
                        }
                    }

                    // Update job tracking
                    {
                        let mut jobs = jobs.write().unwrap();
                        jobs.insert(job.id.clone(), job);
                    }
                } else {
                    // No jobs in queue, sleep for a bit
                    thread::sleep(Duration::from_millis(100));
                }
            }
        });
    }
}

/// Transform registration manager.
pub struct TransformRegistration {
    /// Database operations
    db_ops: Arc<DbOperations>,
    /// State store reference
    state_store: Arc<StateStoreInner>,
}

impl TransformRegistration {
    /// Creates a new transform registration manager.
    pub(crate) fn new(
        db_ops: Arc<DbOperations>,
        state_store: Arc<StateStoreInner>,
    ) -> TransformResult<Self> {
        Ok(Self { db_ops, state_store })
    }

    /// Registers a new transform.
    pub fn register_transform(&self, definition: TransformDefinition) -> TransformResult<TransformId> {
        let transform_id = definition.id.clone();
        
        // Check if transform already exists
        if let Ok(Some(_)) = self.db_ops.get_transform(&transform_id) {
            return Err(TransformError::registration(
                format!("Transform {} already exists", transform_id)
            ));
        }

        // Validate the transform
        self.validate_transform(&definition)?;

        // Store the transform
        self.db_ops
            .store_transform(&transform_id, &definition.transform)
            .map_err(|e| TransformError::registration(format!("Failed to store transform: {}", e)))?;

        // Create initial state
        let state = TransformState::new(transform_id.clone());
        self.state_store
            .save_state(&state)
            .map_err(|e| TransformError::registration(format!("Failed to create initial state: {}", e)))?;

        info!("✨ Registered new transform: {}", transform_id);
        Ok(transform_id)
    }

    /// Lists all registered transforms.
    pub fn list_transforms(&self, filter: Option<&str>) -> Vec<TransformMetadata> {
        let mut transforms = Vec::new();

        match self.db_ops.list_transforms() {
            Ok(transform_ids) => {
                for transform_id in transform_ids {
                    // Apply filter if specified
                    if let Some(filter_str) = filter {
                        if !transform_id.contains(filter_str) {
                            continue;
                        }
                    }

                    if let Ok(Some(transform)) = self.db_ops.get_transform(&transform_id) {
                        let state = self.state_store
                            .load_state(&transform_id)
                            .ok()
                            .flatten()
                            .unwrap_or_else(|| TransformState::new(transform_id.clone()));

                        let metadata = TransformMetadata {
                            id: transform_id.clone(),
                            name: transform_id.clone(), // TODO: Extract from transform metadata
                            description: "Transform description".to_string(), // TODO: Extract from metadata
                            inputs: transform.get_inputs().to_vec(),
                            output: transform.get_output().to_string(),
                            created_at: state.created_at,
                            updated_at: state.updated_at,
                            status: state.status,
                            metadata: state.metadata,
                        };

                        transforms.push(metadata);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to list transforms: {}", e);
            }
        }

        transforms
    }

    /// Updates an existing transform.
    pub fn update_transform(&self, transform_id: TransformId, update: TransformUpdate) -> TransformResult<()> {
        // Load existing transform
        let mut transform = self.db_ops
            .get_transform(&transform_id)
            .map_err(|e| TransformError::database(e.to_string(), "get_transform"))?
            .ok_or_else(|| TransformError::not_found(transform_id.clone(), "update_transform"))?;

        // Apply updates
        if let Some(new_transform) = update.transform {
            transform = new_transform;
        }

        if let Some(new_inputs) = update.inputs {
            transform.set_inputs(new_inputs);
        }

        // Validate updated transform
        let definition = TransformDefinition {
            id: transform_id.clone(),
            transform: transform.clone(),
            inputs: transform.get_inputs().to_vec(),
            metadata: update.metadata.unwrap_or_default(),
        };
        self.validate_transform(&definition)?;

        // Store updated transform
        self.db_ops
            .store_transform(&transform_id, &transform)
            .map_err(|e| TransformError::registration(format!("Failed to update transform: {}", e)))?;

        // Update state if status changed
        if let Some(new_status) = update.status {
            if let Ok(mut state) = self.state_store.load_state(&transform_id).map(|opt| opt.unwrap_or_else(|| TransformState::new(transform_id.clone()))) {
                state.set_status(new_status);
                if let Err(e) = self.state_store.save_state(&state) {
                    warn!("Failed to update transform state: {}", e);
                }
            }
        }

        info!("🔄 Updated transform: {}", transform_id);
        Ok(())
    }

    /// Removes a transform from the system.
    pub fn remove_transform(&self, transform_id: TransformId) -> TransformResult<()> {
        // Check if transform exists
        if self.db_ops.get_transform(&transform_id)
            .map_err(|e| TransformError::database(e.to_string(), "get_transform"))?
            .is_none() {
            return Err(TransformError::not_found(transform_id.clone(), "remove_transform"));
        }

        // Remove transform from database
        if let Err(e) = self.db_ops.transforms_tree.remove(transform_id.as_bytes()) {
            return Err(TransformError::database(
                format!("Failed to remove transform: {}", e),
                "remove_transform"
            ));
        }

        // Remove state
        let state_key = format!("state_{}", transform_id);
        let history_key = format!("history_{}", transform_id);
        
        let _ = self.db_ops.transforms_tree.remove(state_key.as_bytes());
        let _ = self.db_ops.transforms_tree.remove(history_key.as_bytes());

        info!("🗑️ Removed transform: {}", transform_id);
        Ok(())
    }

    /// Validates a transform definition.
    fn validate_transform(&self, definition: &TransformDefinition) -> TransformResult<()> {
        // Basic validation
        if definition.id.is_empty() {
            return Err(TransformError::validation("Transform ID cannot be empty"));
        }

        if definition.transform.logic.is_empty() {
            return Err(TransformError::validation("Transform logic cannot be empty"));
        }

        if definition.transform.output.is_empty() {
            return Err(TransformError::validation("Transform output cannot be empty"));
        }

        // TODO: Add more sophisticated validation
        // - Parse and validate transform logic
        // - Check input/output schema compatibility
        // - Validate permissions and security constraints

        Ok(())
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
    fn test_transform_executor_creation() {
        let db_ops = create_test_db_ops();
        let state_store = Arc::new(StateStoreInner::new(Arc::clone(&db_ops)));
        let executor = TransformExecutor::new(db_ops, state_store);
        assert!(executor.is_ok());
    }

    #[test]
    fn test_transform_registration() {
        let db_ops = create_test_db_ops();
        let state_store = Arc::new(StateStoreInner::new(Arc::clone(&db_ops)));
        let registration = TransformRegistration::new(db_ops, state_store).unwrap();

        let definition = TransformDefinition {
            id: "test_transform".to_string(),
            transform: Transform::new("return input + 1".to_string(), "test.output".to_string()),
            inputs: vec!["test.input".to_string()],
            metadata: HashMap::new(),
        };

        let result = registration.register_transform(definition);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_transform");
    }

    #[test]
    fn test_transform_registration_duplicate() {
        let db_ops = create_test_db_ops();
        let state_store = Arc::new(StateStoreInner::new(Arc::clone(&db_ops)));
        let registration = TransformRegistration::new(db_ops, state_store).unwrap();

        let definition = TransformDefinition {
            id: "test_transform".to_string(),
            transform: Transform::new("return input + 1".to_string(), "test.output".to_string()),
            inputs: vec!["test.input".to_string()],
            metadata: HashMap::new(),
        };

        // First registration should succeed
        let result1 = registration.register_transform(definition.clone());
        assert!(result1.is_ok());

        // Second registration should fail
        let result2 = registration.register_transform(definition);
        assert!(result2.is_err());
    }

    #[test]
    fn test_list_transforms() {
        let db_ops = create_test_db_ops();
        let state_store = Arc::new(StateStoreInner::new(Arc::clone(&db_ops)));
        let registration = TransformRegistration::new(db_ops, state_store).unwrap();

        // Register a transform
        let definition = TransformDefinition {
            id: "test_transform".to_string(),
            transform: Transform::new("return input + 1".to_string(), "test.output".to_string()),
            inputs: vec!["test.input".to_string()],
            metadata: HashMap::new(),
        };
        registration.register_transform(definition).unwrap();

        // List transforms
        let transforms = registration.list_transforms(None);
        assert_eq!(transforms.len(), 1);
        assert_eq!(transforms[0].id, "test_transform");
    }

    #[test]
    fn test_queue_status() {
        let status = QueueStatus::default();
        assert_eq!(status.pending, 0);
        assert_eq!(status.running, 0);
        assert_eq!(status.completed, 0);
        assert_eq!(status.failed, 0);
    }
}