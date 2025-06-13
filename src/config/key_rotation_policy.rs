//! Key rotation policy configuration and management
//!
//! This module provides comprehensive policy management for key rotation operations
//! including policy definition, validation, enforcement, and testing capabilities.

use crate::crypto::key_rotation::RotationReason;
use chrono::{DateTime, Utc, Duration as ChronoDuration, Timelike, Weekday, Datelike};
use log::{info, warn, error, debug};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Policy configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    /// Policy metadata
    pub metadata: PolicyMetadata,
    /// Rotation policies
    pub rotation_policies: Vec<RotationPolicy>,
    /// Rate limiting policies
    pub rate_limits: RateLimitConfig,
    /// Notification policies
    pub notifications: NotificationConfig,
    /// Emergency procedures
    pub emergency_procedures: EmergencyConfig,
    /// Global settings
    pub global_settings: GlobalSettings,
}

/// Policy metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyMetadata {
    /// Policy configuration name
    pub name: String,
    /// Version of the policy configuration
    pub version: String,
    /// Description
    pub description: Option<String>,
    /// Author/creator
    pub author: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// Individual rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    /// Unique policy identifier
    pub id: String,
    /// Human-readable policy name
    pub name: String,
    /// Policy description
    pub description: Option<String>,
    /// Policy status (active, inactive, draft)
    pub status: PolicyStatus,
    /// Policy priority (higher numbers = higher priority)
    pub priority: u32,
    /// Target selector for this policy
    pub target_selector: TargetSelector,
    /// Rotation schedule configuration
    pub schedule: RotationSchedule,
    /// Policy conditions
    pub conditions: PolicyConditions,
    /// Actions to take when policy triggers
    pub actions: PolicyActions,
    /// Exceptions to this policy
    pub exceptions: Vec<PolicyException>,
    /// Metadata for this policy
    pub metadata: HashMap<String, String>,
}

/// Policy status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PolicyStatus {
    /// Policy is active and enforced
    Active,
    /// Policy is inactive (not enforced)
    Inactive,
    /// Policy is in draft mode (validation only)
    Draft,
    /// Policy is deprecated (will be removed)
    Deprecated,
}

/// Target selector for policy application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSelector {
    /// Target type (user, key, role, group, all)
    pub target_type: String,
    /// Selection criteria
    pub criteria: SelectionCriteria,
    /// Include patterns
    pub include_patterns: Vec<String>,
    /// Exclude patterns
    pub exclude_patterns: Vec<String>,
}

/// Selection criteria for targets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// User IDs to include
    pub user_ids: Option<Vec<String>>,
    /// Roles to include
    pub roles: Option<Vec<String>>,
    /// Groups to include
    pub groups: Option<Vec<String>>,
    /// Key types to include
    pub key_types: Option<Vec<String>>,
    /// Key age criteria
    pub key_age: Option<KeyAgeCriteria>,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Key age criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAgeCriteria {
    /// Minimum age in days
    pub min_age_days: Option<u64>,
    /// Maximum age in days
    pub max_age_days: Option<u64>,
    /// Last rotation timeframe
    pub last_rotation_days: Option<u64>,
}

/// Rotation schedule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationSchedule {
    /// Schedule type
    pub schedule_type: ScheduleType,
    /// Schedule interval
    pub interval: ScheduleInterval,
    /// Time windows when rotation is allowed
    pub time_windows: Vec<TimeWindow>,
    /// Timezone for schedule evaluation
    pub timezone: String,
    /// Grace period for rotation completion
    pub grace_period_hours: u64,
}

/// Schedule type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScheduleType {
    /// Fixed interval scheduling
    Interval,
    /// Cron-based scheduling
    Cron,
    /// Event-based scheduling
    Event,
    /// Manual scheduling only
    Manual,
}

/// Schedule interval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleInterval {
    /// Interval value
    pub value: u64,
    /// Interval unit
    pub unit: IntervalUnit,
    /// Jitter to add for load distribution
    pub jitter_percent: Option<f64>,
}

/// Interval unit enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntervalUnit {
    /// Hours
    Hours,
    /// Days
    Days,
    /// Weeks
    Weeks,
    /// Months
    Months,
}

/// Time window for allowed operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeWindow {
    /// Days of week (0=Sunday, 6=Saturday)
    pub days_of_week: Vec<u8>,
    /// Start hour (24-hour format)
    pub start_hour: u8,
    /// End hour (24-hour format)
    pub end_hour: u8,
    /// Time window description
    pub description: Option<String>,
}

/// Policy conditions that must be met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConditions {
    /// Minimum conditions that must be met
    pub minimum_conditions: Vec<Condition>,
    /// Maximum conditions that cannot be exceeded
    pub maximum_conditions: Vec<Condition>,
    /// Custom condition expressions
    pub custom_expressions: Vec<String>,
}

/// Individual condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    /// Condition type
    pub condition_type: String,
    /// Condition value
    pub value: ConditionValue,
    /// Condition operator
    pub operator: ConditionOperator,
    /// Condition description
    pub description: Option<String>,
}

/// Condition value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConditionValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Array of strings
    StringArray(Vec<String>),
    /// Array of numbers
    NumberArray(Vec<f64>),
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than or equal
    LessThanOrEqual,
    /// Contains (for strings/arrays)
    Contains,
    /// Does not contain
    NotContains,
    /// Matches regex pattern
    Matches,
    /// Does not match regex pattern
    NotMatches,
    /// In list
    In,
    /// Not in list
    NotIn,
}

/// Policy actions to execute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyActions {
    /// Primary action to take
    pub primary_action: PolicyAction,
    /// Secondary actions
    pub secondary_actions: Vec<PolicyAction>,
    /// Pre-rotation actions
    pub pre_rotation_actions: Vec<PolicyAction>,
    /// Post-rotation actions
    pub post_rotation_actions: Vec<PolicyAction>,
    /// Failure actions
    pub failure_actions: Vec<PolicyAction>,
}

/// Individual policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyAction {
    /// Action type
    pub action_type: ActionType,
    /// Action parameters
    pub parameters: HashMap<String, String>,
    /// Action timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Whether action is required for success
    pub required: bool,
}

/// Action type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    /// Rotate key immediately
    RotateKey,
    /// Schedule key rotation
    ScheduleRotation,
    /// Send notification
    SendNotification,
    /// Log event
    LogEvent,
    /// Execute webhook
    ExecuteWebhook,
    /// Create audit entry
    CreateAuditEntry,
    /// Invalidate sessions
    InvalidateSessions,
    /// Update permissions
    UpdatePermissions,
    /// Custom action
    Custom(String),
}

/// Policy exception configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyException {
    /// Exception identifier
    pub id: String,
    /// Exception reason
    pub reason: String,
    /// Exception criteria
    pub criteria: SelectionCriteria,
    /// Exception expiration
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether exception is active
    pub active: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Per-user rate limits
    pub per_user_limits: RateLimit,
    /// Per-role rate limits
    pub per_role_limits: RateLimit,
    /// Global rate limits
    pub global_limits: RateLimit,
    /// Bulk operation limits
    pub bulk_operation_limits: BulkRateLimit,
    /// Emergency operation limits
    pub emergency_limits: EmergencyRateLimit,
}

/// Rate limit specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    /// Maximum operations per time window
    pub max_operations: u64,
    /// Time window in seconds
    pub time_window_seconds: u64,
    /// Burst allowance
    pub burst_allowance: Option<u64>,
}

/// Bulk operation rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkRateLimit {
    /// Maximum targets per operation
    pub max_targets_per_operation: u64,
    /// Maximum concurrent operations
    pub max_concurrent_operations: u64,
    /// Cooldown period between operations
    pub cooldown_seconds: u64,
}

/// Emergency operation rate limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyRateLimit {
    /// Maximum emergency operations per hour
    pub max_operations_per_hour: u64,
    /// System-wide emergency cooldown
    pub system_wide_cooldown_hours: u64,
    /// Required approvals for system-wide
    pub required_approvals: u64,
}

/// Notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    /// Notification templates
    pub templates: HashMap<String, NotificationTemplate>,
    /// Escalation rules
    pub escalation_rules: Vec<EscalationRule>,
}

/// Notification channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    /// Channel identifier
    pub id: String,
    /// Channel type (email, slack, webhook, etc.)
    pub channel_type: String,
    /// Channel configuration
    pub config: HashMap<String, String>,
    /// Whether channel is enabled
    pub enabled: bool,
}

/// Notification template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationTemplate {
    /// Template subject
    pub subject: String,
    /// Template body
    pub body: String,
    /// Template format (text, html, markdown)
    pub format: String,
}

/// Escalation rule for notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationRule {
    /// Rule identifier
    pub id: String,
    /// Trigger conditions
    pub trigger_conditions: Vec<Condition>,
    /// Escalation steps
    pub escalation_steps: Vec<EscalationStep>,
}

/// Escalation step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStep {
    /// Delay before this step
    pub delay_minutes: u64,
    /// Notification channels for this step
    pub channels: Vec<String>,
    /// Recipients for this step
    pub recipients: Vec<String>,
}

/// Emergency procedures configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyConfig {
    /// Emergency contact information
    pub emergency_contacts: Vec<EmergencyContact>,
    /// Emergency procedures
    pub procedures: HashMap<String, EmergencyProcedure>,
    /// Approval workflows
    pub approval_workflows: Vec<ApprovalWorkflow>,
}

/// Emergency contact information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyContact {
    /// Contact name
    pub name: String,
    /// Contact role
    pub role: String,
    /// Contact methods
    pub contact_methods: HashMap<String, String>,
    /// Availability schedule
    pub availability: Option<String>,
}

/// Emergency procedure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmergencyProcedure {
    /// Procedure name
    pub name: String,
    /// Procedure description
    pub description: String,
    /// Required steps
    pub required_steps: Vec<String>,
    /// Automated actions
    pub automated_actions: Vec<PolicyAction>,
    /// Manual verification required
    pub requires_manual_verification: bool,
}

/// Approval workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    /// Workflow identifier
    pub id: String,
    /// Workflow name
    pub name: String,
    /// Required approvers
    pub required_approvers: Vec<String>,
    /// Approval threshold
    pub approval_threshold: u32,
    /// Timeout for approvals
    pub timeout_hours: u64,
}

/// Global policy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalSettings {
    /// Default rotation reason
    pub default_rotation_reason: RotationReason,
    /// Default grace period
    pub default_grace_period_hours: u64,
    /// Enable policy enforcement
    pub enforce_policies: bool,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Default timezone
    pub default_timezone: String,
    /// Backup retention days
    pub backup_retention_days: u64,
}

/// Policy validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyValidationResult {
    /// Whether the policy is valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
    /// Policy conflicts detected
    pub conflicts: Option<Vec<String>>,
    /// Validation metadata
    pub metadata: HashMap<String, String>,
}

/// Key rotation policy engine
pub struct KeyRotationPolicyEngine {
    /// Current policy configuration
    config: Option<PolicyConfig>,
    /// Policy validation rules
    validation_rules: Vec<Box<dyn PolicyValidator + Send + Sync>>,
    /// Policy cache for performance
    policy_cache: HashMap<String, Vec<RotationPolicy>>,
    /// Last configuration update time
    last_update: DateTime<Utc>,
}

/// Policy validator trait
pub trait PolicyValidator {
    /// Validate a policy configuration
    fn validate(&self, config: &PolicyConfig) -> PolicyValidationResult;
    
    /// Get validator name
    fn name(&self) -> &str;
}

impl KeyRotationPolicyEngine {
    /// Create a new policy engine
    pub fn new() -> Self {
        Self {
            config: None,
            validation_rules: Self::create_default_validators(),
            policy_cache: HashMap::new(),
            last_update: Utc::now(),
        }
    }

    /// Create policy engine with default configuration
    pub fn new_with_defaults() -> Self {
        let mut engine = Self::new();
        engine.config = Some(Self::create_default_config());
        engine.rebuild_cache();
        engine
    }

    /// Load policy configuration from file
    pub async fn load_from_file(&mut self, file_path: &PathBuf) -> Result<(), String> {
        info!("Loading policy configuration from: {:?}", file_path);

        let content = fs::read_to_string(file_path)
            .map_err(|e| format!("Failed to read policy file: {}", e))?;

        let config: PolicyConfig = if file_path.extension().and_then(|s| s.to_str()) == Some("json") {
            serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse JSON policy config: {}", e))?
        } else {
            serde_yaml::from_str(&content)
                .map_err(|e| format!("Failed to parse YAML policy config: {}", e))?
        };

        self.set_config(config).await
    }

    /// Set policy configuration
    pub async fn set_config(&mut self, config: PolicyConfig) -> Result<(), String> {
        // Validate configuration
        let validation_result = self.validate_config(&config).await;
        if !validation_result.is_valid {
            return Err(format!("Policy configuration validation failed: {:?}", validation_result.errors));
        }

        self.config = Some(config);
        self.last_update = Utc::now();
        self.rebuild_cache();

        info!("Policy configuration updated successfully");
        Ok(())
    }

    /// Validate policy configuration
    pub async fn validate_config(&self, config: &PolicyConfig) -> PolicyValidationResult {
        let mut result = PolicyValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            conflicts: Some(Vec::new()),
            metadata: HashMap::new(),
        };

        // Run all validation rules
        for validator in &self.validation_rules {
            let validation = validator.validate(config);
            result.errors.extend(validation.errors);
            result.warnings.extend(validation.warnings);

            if let Some(conflicts) = validation.conflicts {
                result.conflicts.as_mut().unwrap().extend(conflicts);
            }
        }

        // Additional validation logic
        self.validate_policy_consistency(config, &mut result);
        self.validate_time_windows(config, &mut result);
        self.validate_rate_limits(config, &mut result);

        result.is_valid = result.errors.is_empty();
        result
    }

    /// Check if bulk operation is allowed
    pub async fn is_bulk_operation_allowed(&self, reason: &RotationReason) -> bool {
        if let Some(config) = &self.config {
            // Check global settings
            if !config.global_settings.enforce_policies {
                return true;
            }

            // Check if there are any policies that allow bulk operations for this reason
            for policy in &config.rotation_policies {
                if policy.status == PolicyStatus::Active && self.policy_allows_bulk_operation(policy, reason) {
                    return true;
                }
            }
        }

        // Default to allowing bulk operations if no specific restrictions
        true
    }

    /// Check bulk rate limit
    pub async fn check_bulk_rate_limit(&self, target_count: usize) -> bool {
        if let Some(config) = &self.config {
            let bulk_limits = &config.rate_limits.bulk_operation_limits;
            return target_count <= bulk_limits.max_targets_per_operation as usize;
        }
        true
    }

    /// Check if emergency operation is allowed
    pub async fn is_emergency_operation_allowed(&self, _reason: &RotationReason) -> bool {
        // Emergency operations are generally allowed unless explicitly restricted
        true
    }

    /// Validate rotation eligibility for a target
    pub async fn validate_rotation_eligibility(&self, target: &str, reason: &RotationReason) -> Result<(), String> {
        if let Some(config) = &self.config {
            // Find applicable policies for this target
            let applicable_policies = self.find_applicable_policies(target, reason).await;
            
            for policy in applicable_policies {
                // Check policy conditions
                if !self.evaluate_policy_conditions(&policy, target, reason).await {
                    return Err(format!("Policy '{}' conditions not met for target '{}'", policy.name, target));
                }
            }
        }
        Ok(())
    }

    /// List policies with optional filters
    pub async fn list_policies(&self, filters: &HashMap<String, String>) -> Result<Vec<serde_json::Value>, String> {
        if let Some(config) = &self.config {
            let mut policies = Vec::new();
            
            for policy in &config.rotation_policies {
                // Apply filters
                if let Some(policy_type) = filters.get("type") {
                    if policy.target_selector.target_type != *policy_type {
                        continue;
                    }
                }
                
                if let Some(include_inactive) = filters.get("include_inactive") {
                    if include_inactive != "true" && policy.status != PolicyStatus::Active {
                        continue;
                    }
                }
                
                // Convert to JSON for response
                let policy_json = serde_json::json!({
                    "id": policy.id,
                    "name": policy.name,
                    "description": policy.description,
                    "status": format!("{:?}", policy.status),
                    "type": policy.target_selector.target_type,
                    "priority": policy.priority,
                    "schedule_type": format!("{:?}", policy.schedule.schedule_type)
                });
                
                policies.push(policy_json);
            }
            
            Ok(policies)
        } else {
            Ok(Vec::new())
        }
    }

    /// Test policies with given configuration and scenarios
    pub async fn test_policies(&self, _config: PolicyConfig, scenarios: &str) -> Result<serde_json::Value, String> {
        // Parse test scenarios
        let test_scenarios: serde_json::Value = serde_json::from_str(scenarios)
            .map_err(|e| format!("Failed to parse test scenarios: {}", e))?;
        
        let mut test_results = Vec::new();
        
        if let Some(scenarios_array) = test_scenarios.get("scenarios").and_then(|s| s.as_array()) {
            for scenario in scenarios_array {
                let scenario_name = scenario.get("name").and_then(|n| n.as_str()).unwrap_or("unknown");
                let user_id = scenario.get("user_id").and_then(|u| u.as_str()).unwrap_or("test_user");
                let reason_str = scenario.get("reason").and_then(|r| r.as_str()).unwrap_or("Scheduled");
                
                // Convert reason string to RotationReason
                let reason = match reason_str {
                    "Scheduled" => RotationReason::Scheduled,
                    "Maintenance" => RotationReason::Maintenance,
                    "Compromise" => RotationReason::Compromise,
                    _ => RotationReason::Scheduled,
                };
                
                // Test rotation eligibility
                let eligibility_result = self.validate_rotation_eligibility(user_id, &reason).await;
                
                let test_result = serde_json::json!({
                    "scenario": scenario_name,
                    "user_id": user_id,
                    "reason": reason_str,
                    "eligible": eligibility_result.is_ok(),
                    "error": eligibility_result.err(),
                    "timestamp": Utc::now()
                });
                
                test_results.push(test_result);
            }
        }
        
        Ok(serde_json::json!({
            "test_results": test_results,
            "total_scenarios": test_results.len(),
            "passed": test_results.iter().filter(|r| r.get("eligible").and_then(|e| e.as_bool()).unwrap_or(false)).count()
        }))
    }

    /// Export policies in specified format
    pub async fn export_policies(&self, format: &str, include_metadata: bool) -> Result<String, String> {
        if let Some(config) = &self.config {
            let export_data = if include_metadata {
                serde_json::json!({
                    "metadata": config.metadata,
                    "policies": config.rotation_policies,
                    "rate_limits": config.rate_limits,
                    "global_settings": config.global_settings,
                    "exported_at": Utc::now()
                })
            } else {
                serde_json::json!({
                    "policies": config.rotation_policies,
                    "exported_at": Utc::now()
                })
            };
            
            match format {
                "json" => {
                    serde_json::to_string_pretty(&export_data)
                        .map_err(|e| format!("Failed to serialize to JSON: {}", e))
                }
                "yaml" => {
                    serde_yaml::to_string(&export_data)
                        .map_err(|e| format!("Failed to serialize to YAML: {}", e))
                }
                _ => Err(format!("Unsupported export format: {}", format))
            }
        } else {
            Err("No policy configuration loaded".to_string())
        }
    }

    /// Apply new configuration
    pub async fn apply_config(&mut self, config: PolicyConfig, force: bool) -> Result<usize, String> {
        if !force {
            // Validate configuration first
            let validation_result = self.validate_config(&config).await;
            if !validation_result.is_valid {
                return Err(format!("Configuration validation failed: {:?}", validation_result.errors));
            }
        }
        
        let policies_count = config.rotation_policies.len();
        self.config = Some(config);
        self.last_update = Utc::now();
        self.rebuild_cache();
        
        Ok(policies_count)
    }

    /// Backup current policies
    pub async fn backup_current_policies(&self) -> Result<String, String> {
        if let Some(config) = &self.config {
            let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
            let backup_filename = format!("policy_backup_{}.yaml", timestamp);
            let backup_path = PathBuf::from("backups").join(&backup_filename);
            
            // Create backups directory if it doesn't exist
            if let Some(parent) = backup_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("Failed to create backup directory: {}", e))?;
            }
            
            let backup_content = serde_yaml::to_string(config)
                .map_err(|e| format!("Failed to serialize backup: {}", e))?;
            
            std::fs::write(&backup_path, backup_content)
                .map_err(|e| format!("Failed to write backup file: {}", e))?;
            
            Ok(backup_path.to_string_lossy().to_string())
        } else {
            Err("No configuration to backup".to_string())
        }
    }

    // Private helper methods

    /// Create default validators
    fn create_default_validators() -> Vec<Box<dyn PolicyValidator + Send + Sync>> {
        vec![
            Box::new(BasicPolicyValidator),
            Box::new(ScheduleValidator),
            Box::new(RateLimitValidator),
        ]
    }

    /// Create default configuration
    fn create_default_config() -> PolicyConfig {
        PolicyConfig {
            metadata: PolicyMetadata {
                name: "Default Policy Configuration".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Default key rotation policies".to_string()),
                author: Some("DataFold System".to_string()),
                created_at: Utc::now(),
                modified_at: Utc::now(),
                tags: vec!["default".to_string()],
            },
            rotation_policies: vec![],
            rate_limits: RateLimitConfig {
                per_user_limits: RateLimit {
                    max_operations: 10,
                    time_window_seconds: 3600,
                    burst_allowance: Some(5),
                },
                per_role_limits: RateLimit {
                    max_operations: 100,
                    time_window_seconds: 3600,
                    burst_allowance: Some(20),
                },
                global_limits: RateLimit {
                    max_operations: 1000,
                    time_window_seconds: 3600,
                    burst_allowance: Some(100),
                },
                bulk_operation_limits: BulkRateLimit {
                    max_targets_per_operation: 1000,
                    max_concurrent_operations: 10,
                    cooldown_seconds: 300,
                },
                emergency_limits: EmergencyRateLimit {
                    max_operations_per_hour: 5,
                    system_wide_cooldown_hours: 24,
                    required_approvals: 2,
                },
            },
            notifications: NotificationConfig {
                channels: vec![],
                templates: HashMap::new(),
                escalation_rules: vec![],
            },
            emergency_procedures: EmergencyConfig {
                emergency_contacts: vec![],
                procedures: HashMap::new(),
                approval_workflows: vec![],
            },
            global_settings: GlobalSettings {
                default_rotation_reason: RotationReason::Scheduled,
                default_grace_period_hours: 24,
                enforce_policies: true,
                enable_audit_logging: true,
                enable_performance_monitoring: true,
                default_timezone: "UTC".to_string(),
                backup_retention_days: 90,
            },
        }
    }

    /// Rebuild policy cache
    fn rebuild_cache(&mut self) {
        self.policy_cache.clear();
        
        if let Some(config) = &self.config {
            for policy in &config.rotation_policies {
                let target_type = &policy.target_selector.target_type;
                self.policy_cache
                    .entry(target_type.clone())
                    .or_insert_with(Vec::new)
                    .push(policy.clone());
            }
        }
    }

    /// Validate policy consistency
    fn validate_policy_consistency(&self, config: &PolicyConfig, result: &mut PolicyValidationResult) {
        // Check for policy ID uniqueness
        let mut policy_ids = std::collections::HashSet::new();
        for policy in &config.rotation_policies {
            if !policy_ids.insert(&policy.id) {
                result.errors.push(format!("Duplicate policy ID: {}", policy.id));
            }
        }
    }

    /// Validate time windows
    fn validate_time_windows(&self, config: &PolicyConfig, result: &mut PolicyValidationResult) {
        for policy in &config.rotation_policies {
            for window in &policy.schedule.time_windows {
                if window.start_hour >= 24 || window.end_hour >= 24 {
                    result.errors.push(format!("Invalid time window in policy '{}': hours must be 0-23", policy.name));
                }
                
                for &day in &window.days_of_week {
                    if day > 6 {
                        result.errors.push(format!("Invalid day of week in policy '{}': must be 0-6", policy.name));
                    }
                }
            }
        }
    }

    /// Validate rate limits
    fn validate_rate_limits(&self, config: &PolicyConfig, result: &mut PolicyValidationResult) {
        let limits = &config.rate_limits;
        
        if limits.bulk_operation_limits.max_targets_per_operation == 0 {
            result.errors.push("Bulk operation max targets cannot be zero".to_string());
        }
        
        if limits.emergency_limits.max_operations_per_hour == 0 {
            result.warnings.push("Emergency operations per hour limit is zero - this may prevent emergency responses".to_string());
        }
    }

    /// Check if policy allows bulk operation
    fn policy_allows_bulk_operation(&self, _policy: &RotationPolicy, _reason: &RotationReason) -> bool {
        // This would implement more sophisticated policy evaluation
        // For now, return true as a placeholder
        true
    }

    /// Find applicable policies for target
    async fn find_applicable_policies(&self, _target: &str, _reason: &RotationReason) -> Vec<RotationPolicy> {
        // This would implement policy matching logic
        // For now, return empty list as placeholder
        Vec::new()
    }

    /// Evaluate policy conditions
    async fn evaluate_policy_conditions(&self, _policy: &RotationPolicy, _target: &str, _reason: &RotationReason) -> bool {
        // This would implement condition evaluation logic
        // For now, return true as placeholder
        true
    }
}

/// Basic policy validator
struct BasicPolicyValidator;

impl PolicyValidator for BasicPolicyValidator {
    fn validate(&self, config: &PolicyConfig) -> PolicyValidationResult {
        let mut result = PolicyValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            conflicts: None,
            metadata: HashMap::new(),
        };

        // Validate metadata
        if config.metadata.name.is_empty() {
            result.errors.push("Policy configuration name cannot be empty".to_string());
        }

        if config.metadata.version.is_empty() {
            result.errors.push("Policy configuration version cannot be empty".to_string());
        }

        // Validate policies
        for policy in &config.rotation_policies {
            if policy.id.is_empty() {
                result.errors.push("Policy ID cannot be empty".to_string());
            }

            if policy.name.is_empty() {
                result.errors.push(format!("Policy '{}' name cannot be empty", policy.id));
            }

            if policy.target_selector.target_type.is_empty() {
                result.errors.push(format!("Policy '{}' target type cannot be empty", policy.id));
            }
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    fn name(&self) -> &str {
        "BasicPolicyValidator"
    }
}

/// Schedule validator
struct ScheduleValidator;

impl PolicyValidator for ScheduleValidator {
    fn validate(&self, config: &PolicyConfig) -> PolicyValidationResult {
        let mut result = PolicyValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            conflicts: None,
            metadata: HashMap::new(),
        };

        for policy in &config.rotation_policies {
            let schedule = &policy.schedule;
            
            // Validate interval
            if schedule.interval.value == 0 {
                result.errors.push(format!("Policy '{}' schedule interval cannot be zero", policy.name));
            }

            // Validate grace period
            if schedule.grace_period_hours > 168 { // More than a week
                result.warnings.push(format!("Policy '{}' has very long grace period ({}h)", policy.name, schedule.grace_period_hours));
            }
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    fn name(&self) -> &str {
        "ScheduleValidator"
    }
}

/// Rate limit validator
struct RateLimitValidator;

impl PolicyValidator for RateLimitValidator {
    fn validate(&self, config: &PolicyConfig) -> PolicyValidationResult {
        let mut result = PolicyValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            conflicts: None,
            metadata: HashMap::new(),
        };

        let limits = &config.rate_limits;

        // Validate that per-user limits are less than global limits
        if limits.per_user_limits.max_operations > limits.global_limits.max_operations {
            result.warnings.push("Per-user rate limit exceeds global limit".to_string());
        }

        // Validate emergency limits
        if limits.emergency_limits.required_approvals == 0 {
            result.warnings.push("Emergency operations require no approvals - this may be insecure".to_string());
        }

        result.is_valid = result.errors.is_empty();
        result
    }

    fn name(&self) -> &str {
        "RateLimitValidator"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_config_serialization() {
        let config = PolicyConfig {
            metadata: PolicyMetadata {
                name: "Test Config".to_string(),
                version: "1.0.0".to_string(),
                description: None,
                author: None,
                created_at: Utc::now(),
                modified_at: Utc::now(),
                tags: vec![],
            },
            rotation_policies: vec![],
            rate_limits: RateLimitConfig {
                per_user_limits: RateLimit {
                    max_operations: 10,
                    time_window_seconds: 3600,
                    burst_allowance: None,
                },
                per_role_limits: RateLimit {
                    max_operations: 100,
                    time_window_seconds: 3600,
                    burst_allowance: None,
                },
                global_limits: RateLimit {
                    max_operations: 1000,
                    time_window_seconds: 3600,
                    burst_allowance: None,
                },
                bulk_operation_limits: BulkRateLimit {
                    max_targets_per_operation: 1000,
                    max_concurrent_operations: 10,
                    cooldown_seconds: 300,
                },
                emergency_limits: EmergencyRateLimit {
                    max_operations_per_hour: 5,
                    system_wide_cooldown_hours: 24,
                    required_approvals: 2,
                },
            },
            notifications: NotificationConfig {
                channels: vec![],
                templates: HashMap::new(),
                escalation_rules: vec![],
            },
            emergency_procedures: EmergencyConfig {
                emergency_contacts: vec![],
                procedures: HashMap::new(),
                approval_workflows: vec![],
            },
            global_settings: GlobalSettings {
                default_rotation_reason: RotationReason::Scheduled,
                default_grace_period_hours: 24,
                enforce_policies: true,
                enable_audit_logging: true,
                enable_performance_monitoring: true,
                default_timezone: "UTC".to_string(),
                backup_retention_days: 90,
            },
        };

        let json = serde_json::to_string(&config).expect("Should serialize to JSON");
        let deserialized: PolicyConfig = serde_json::from_str(&json).expect("Should deserialize from JSON");
        
        assert_eq!(config.metadata.name, deserialized.metadata.name);
        assert_eq!(config.metadata.version, deserialized.metadata.version);
    }

    #[tokio::test]
    async fn test_policy_engine_validation() {
        let engine = KeyRotationPolicyEngine::new_with_defaults();
        let config = KeyRotationPolicyEngine::create_default_config();
        
        let result = engine.validate_config(&config).await;
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_bulk_operation_validation() {
        let engine = KeyRotationPolicyEngine::new_with_defaults();
        
        let allowed = engine.is_bulk_operation_allowed(&RotationReason::Maintenance).await;
        assert!(allowed);
        
        let rate_limit_ok = engine.check_bulk_rate_limit(100).await;
        assert!(rate_limit_ok);
        
        let rate_limit_exceeded = engine.check_bulk_rate_limit(2000).await;
        assert!(!rate_limit_exceeded);
    }
}