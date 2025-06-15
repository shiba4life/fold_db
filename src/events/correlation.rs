//! Cross-Platform Event Correlation
//!
//! This module provides correlation capabilities for security events across different
//! DataFold platforms (Rust CLI, JavaScript SDK, Python SDK). It enables tracing
//! related events and building comprehensive security incident timelines.

use super::event_types::{PlatformSource, SecurityEvent, SecurityEventCategory};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::Duration;
use uuid::Uuid;

/// Configuration for event correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationConfig {
    /// Time window for correlating events
    pub correlation_window: Duration,
    /// Maximum number of events to keep in correlation cache
    pub max_cached_events: usize,
    /// Maximum number of correlation groups to maintain
    pub max_correlation_groups: usize,
    /// Enable automatic correlation discovery
    pub auto_discovery: bool,
    /// Correlation strategies to use
    pub strategies: Vec<CorrelationStrategy>,
}

impl Default for CorrelationConfig {
    fn default() -> Self {
        Self {
            correlation_window: Duration::from_secs(3600), // 1 hour
            max_cached_events: 10000,
            max_correlation_groups: 1000,
            auto_discovery: true,
            strategies: vec![
                CorrelationStrategy::TraceId,
                CorrelationStrategy::CorrelationId,
                CorrelationStrategy::SessionId,
                CorrelationStrategy::Actor,
                CorrelationStrategy::TemporalProximity,
            ],
        }
    }
}

/// Strategies for correlating events
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorrelationStrategy {
    /// Correlate by trace ID
    TraceId,
    /// Correlate by correlation ID
    CorrelationId,
    /// Correlate by session ID
    SessionId,
    /// Correlate by actor/user
    Actor,
    /// Correlate by temporal proximity and event patterns
    TemporalProximity,
    /// Correlate by operation sequence
    OperationSequence,
    /// Correlate by component interaction
    ComponentInteraction,
}

/// A group of correlated events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationGroup {
    /// Unique identifier for this correlation group
    pub group_id: Uuid,
    /// Primary correlation key (trace_id, correlation_id, etc.)
    pub primary_key: String,
    /// Strategy used to create this correlation
    pub strategy: CorrelationStrategy,
    /// Events in this correlation group
    pub events: Vec<SecurityEvent>,
    /// When this correlation group was created
    pub created_at: DateTime<Utc>,
    /// Last time an event was added to this group
    pub last_updated: DateTime<Utc>,
    /// Platforms involved in this correlation
    pub platforms: HashSet<PlatformSource>,
    /// Event categories involved
    pub categories: HashSet<SecurityEventCategory>,
    /// Timeline span of events in this group
    pub time_span: Option<Duration>,
}

impl CorrelationGroup {
    /// Create a new correlation group
    pub fn new(
        primary_key: String,
        strategy: CorrelationStrategy,
        initial_event: SecurityEvent,
    ) -> Self {
        let mut platforms = HashSet::new();
        platforms.insert(initial_event.platform().clone());

        let mut categories = HashSet::new();
        categories.insert(initial_event.category());

        let now = Utc::now();

        Self {
            group_id: Uuid::new_v4(),
            primary_key,
            strategy,
            events: vec![initial_event],
            created_at: now,
            last_updated: now,
            platforms,
            categories,
            time_span: None,
        }
    }

    /// Add an event to this correlation group
    pub fn add_event(&mut self, event: SecurityEvent) {
        self.platforms.insert(event.platform().clone());
        self.categories.insert(event.category());
        self.events.push(event);
        self.last_updated = Utc::now();
        self.update_time_span();
    }

    /// Update the time span based on events in the group
    fn update_time_span(&mut self) {
        if self.events.len() < 2 {
            self.time_span = None;
            return;
        }

        let mut earliest = &self.events[0].base_event().timestamp;
        let mut latest = &self.events[0].base_event().timestamp;

        for event in &self.events {
            let timestamp = &event.base_event().timestamp;
            if timestamp < earliest {
                earliest = timestamp;
            }
            if timestamp > latest {
                latest = timestamp;
            }
        }

        self.time_span = Some(Duration::from_millis(
            (latest.timestamp_millis() - earliest.timestamp_millis()) as u64,
        ));
    }

    /// Check if this group involves multiple platforms
    pub fn is_cross_platform(&self) -> bool {
        self.platforms.len() > 1
    }

    /// Get events sorted by timestamp
    pub fn events_by_timestamp(&self) -> Vec<&SecurityEvent> {
        let mut events: Vec<&SecurityEvent> = self.events.iter().collect();
        events.sort_by_key(|e| e.base_event().timestamp);
        events
    }

    /// Get summary of this correlation group
    pub fn summary(&self) -> CorrelationSummary {
        CorrelationSummary {
            group_id: self.group_id,
            primary_key: self.primary_key.clone(),
            strategy: self.strategy.clone(),
            event_count: self.events.len(),
            platform_count: self.platforms.len(),
            category_count: self.categories.len(),
            time_span: self.time_span,
            is_cross_platform: self.is_cross_platform(),
            severity_levels: self
                .events
                .iter()
                .map(|e| e.severity())
                .collect::<HashSet<_>>()
                .len(),
        }
    }
}

/// Summary information about a correlation group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationSummary {
    /// Group identifier
    pub group_id: Uuid,
    /// Primary correlation key
    pub primary_key: String,
    /// Correlation strategy used
    pub strategy: CorrelationStrategy,
    /// Number of events in the group
    pub event_count: usize,
    /// Number of different platforms involved
    pub platform_count: usize,
    /// Number of different event categories
    pub category_count: usize,
    /// Time span of events
    pub time_span: Option<Duration>,
    /// Whether this involves multiple platforms
    pub is_cross_platform: bool,
    /// Number of different severity levels
    pub severity_levels: usize,
}

/// Manages event correlation across platforms
pub struct CorrelationManager {
    /// Configuration for correlation
    config: CorrelationConfig,
    /// Active correlation groups by primary key
    correlation_groups: HashMap<String, CorrelationGroup>,
    /// Index mapping event IDs to correlation group IDs
    event_to_group: HashMap<Uuid, Uuid>,
    /// Recent events cache for auto-discovery
    recent_events: VecDeque<(DateTime<Utc>, SecurityEvent)>,
    /// Correlation statistics
    statistics: CorrelationStatistics,
}

/// Statistics about correlation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationStatistics {
    /// Total correlations created
    pub total_correlations: u64,
    /// Correlations by strategy
    pub correlations_by_strategy: HashMap<CorrelationStrategy, u64>,
    /// Cross-platform correlations
    pub cross_platform_correlations: u64,
    /// Events processed for correlation
    pub events_processed: u64,
    /// Auto-discovered correlations
    pub auto_discovered: u64,
    /// Average events per correlation
    pub avg_events_per_correlation: f64,
    /// Cache hit rate for correlation lookups
    pub cache_hit_rate: f64,
}

impl Default for CorrelationStatistics {
    fn default() -> Self {
        Self {
            total_correlations: 0,
            correlations_by_strategy: HashMap::new(),
            cross_platform_correlations: 0,
            events_processed: 0,
            auto_discovered: 0,
            avg_events_per_correlation: 0.0,
            cache_hit_rate: 0.0,
        }
    }
}

impl CorrelationManager {
    /// Create a new correlation manager
    pub fn new(correlation_window: Duration) -> Self {
        Self {
            config: CorrelationConfig {
                correlation_window,
                ..Default::default()
            },
            correlation_groups: HashMap::new(),
            event_to_group: HashMap::new(),
            recent_events: VecDeque::new(),
            statistics: CorrelationStatistics::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CorrelationConfig) -> Self {
        Self {
            config,
            correlation_groups: HashMap::new(),
            event_to_group: HashMap::new(),
            recent_events: VecDeque::new(),
            statistics: CorrelationStatistics::default(),
        }
    }

    /// Add an event and perform correlation
    pub async fn add_event(&mut self, event: &SecurityEvent) {
        self.statistics.events_processed += 1;

        // Clean up old events and correlations
        self.cleanup_old_data().await;

        // Add to recent events cache
        self.recent_events.push_back((Utc::now(), event.clone()));

        // Limit recent events cache size
        while self.recent_events.len() > self.config.max_cached_events {
            self.recent_events.pop_front();
        }

        // Try to correlate using configured strategies
        let mut correlated = false;
        let strategies = self.config.strategies.clone(); // Clone to avoid borrow issues

        for strategy in strategies {
            if self.correlate_event(event, strategy).await {
                correlated = true;
                break; // Use first successful correlation strategy
            }
        }

        // If no explicit correlation found and auto-discovery is enabled
        if !correlated && self.config.auto_discovery {
            self.auto_discover_correlations(event).await;
        }
    }

    /// Get correlated events for a specific event
    pub async fn get_correlated_events(&self, event_id: Uuid) -> Vec<SecurityEvent> {
        if let Some(group_id) = self.event_to_group.get(&event_id) {
            if let Some(group) = self
                .correlation_groups
                .values()
                .find(|g| g.group_id == *group_id)
            {
                return group.events.clone();
            }
        }
        Vec::new()
    }

    /// Get correlation group by group ID
    pub async fn get_correlation_group(&self, group_id: Uuid) -> Option<&CorrelationGroup> {
        self.correlation_groups
            .values()
            .find(|g| g.group_id == group_id)
    }

    /// Get all correlation groups
    pub async fn get_all_correlations(&self) -> Vec<&CorrelationGroup> {
        self.correlation_groups.values().collect()
    }

    /// Get cross-platform correlations only
    pub async fn get_cross_platform_correlations(&self) -> Vec<&CorrelationGroup> {
        self.correlation_groups
            .values()
            .filter(|g| g.is_cross_platform())
            .collect()
    }

    /// Get correlation statistics
    pub async fn get_statistics(&self) -> CorrelationStatistics {
        let mut stats = self.statistics.clone();

        // Update calculated statistics
        if stats.total_correlations > 0 {
            let total_events: usize = self
                .correlation_groups
                .values()
                .map(|g| g.events.len())
                .sum();
            stats.avg_events_per_correlation =
                total_events as f64 / stats.total_correlations as f64;
        }

        stats
    }

    /// Search correlations by criteria
    pub async fn search_correlations(
        &self,
        criteria: &CorrelationSearchCriteria,
    ) -> Vec<&CorrelationGroup> {
        self.correlation_groups
            .values()
            .filter(|group| criteria.matches(group))
            .collect()
    }

    // Private methods

    /// Attempt to correlate an event using a specific strategy
    async fn correlate_event(
        &mut self,
        event: &SecurityEvent,
        strategy: CorrelationStrategy,
    ) -> bool {
        let base_event = event.base_event();

        let correlation_key = match strategy {
            CorrelationStrategy::TraceId => base_event.trace_id.clone(),
            CorrelationStrategy::CorrelationId => {
                base_event.correlation_id.map(|id| id.to_string())
            }
            CorrelationStrategy::SessionId => base_event.session_id.clone(),
            CorrelationStrategy::Actor => base_event.actor.clone(),
            _ => None, // Other strategies handled separately
        };

        if let Some(key) = correlation_key {
            let group_key = format!("{}:{}", strategy_to_prefix(&strategy), key);

            if let Some(group) = self.correlation_groups.get_mut(&group_key) {
                // Check if group will become cross-platform
                let was_cross_platform = group.is_cross_platform();

                // Add to existing group
                group.add_event(event.clone());
                self.event_to_group
                    .insert(base_event.event_id, group.group_id);

                // Update cross-platform counter if group became cross-platform
                if !was_cross_platform && group.is_cross_platform() {
                    self.statistics.cross_platform_correlations += 1;
                }

                return true;
            } else {
                // Create new group
                let new_group = CorrelationGroup::new(key, strategy.clone(), event.clone());
                let group_id = new_group.group_id;

                self.event_to_group.insert(base_event.event_id, group_id);
                self.correlation_groups.insert(group_key, new_group);

                // Update statistics
                self.statistics.total_correlations += 1;
                *self
                    .statistics
                    .correlations_by_strategy
                    .entry(strategy)
                    .or_insert(0) += 1;

                return true;
            }
        }

        false
    }

    /// Auto-discover correlations based on patterns
    async fn auto_discover_correlations(&mut self, event: &SecurityEvent) {
        // Look for temporal proximity patterns
        if self
            .config
            .strategies
            .contains(&CorrelationStrategy::TemporalProximity)
        {
            self.discover_temporal_correlations(event).await;
        }

        // Look for operation sequence patterns
        if self
            .config
            .strategies
            .contains(&CorrelationStrategy::OperationSequence)
        {
            self.discover_operation_sequences(event).await;
        }

        // Look for component interaction patterns
        if self
            .config
            .strategies
            .contains(&CorrelationStrategy::ComponentInteraction)
        {
            self.discover_component_interactions(event).await;
        }
    }

    /// Discover temporal correlations (events close in time)
    async fn discover_temporal_correlations(&mut self, event: &SecurityEvent) {
        let base_event = event.base_event();
        let time_window = ChronoDuration::minutes(5); // 5-minute window for temporal correlation

        // Look for recent events from different platforms
        let candidates: Vec<&SecurityEvent> = self
            .recent_events
            .iter()
            .filter_map(|(_, cached_event)| {
                let cached_base = cached_event.base_event();

                // Different platform but similar operation or component
                if cached_event.platform() != event.platform()
                    && (cached_base.operation == base_event.operation
                        || cached_base.component == base_event.component)
                    && (base_event.timestamp - cached_base.timestamp) < time_window
                {
                    Some(cached_event)
                } else {
                    None
                }
            })
            .collect();

        if !candidates.is_empty() {
            let correlation_key = format!(
                "temporal_{}_{}",
                base_event.operation,
                base_event.timestamp.timestamp() / 300 // 5-minute buckets
            );

            if let Some(group) = self.correlation_groups.get_mut(&correlation_key) {
                group.add_event(event.clone());
                self.event_to_group
                    .insert(base_event.event_id, group.group_id);
            } else {
                let mut new_group = CorrelationGroup::new(
                    correlation_key.clone(),
                    CorrelationStrategy::TemporalProximity,
                    event.clone(),
                );

                let group_id = new_group.group_id; // Capture before any potential moves

                // Add related events
                for candidate in candidates {
                    new_group.add_event(candidate.clone());
                    self.event_to_group
                        .insert(candidate.base_event().event_id, group_id);
                }

                let is_cross_platform = new_group.is_cross_platform();
                self.event_to_group.insert(base_event.event_id, group_id);
                self.correlation_groups.insert(correlation_key, new_group);

                self.statistics.auto_discovered += 1;
                self.statistics.total_correlations += 1;
                *self
                    .statistics
                    .correlations_by_strategy
                    .entry(CorrelationStrategy::TemporalProximity)
                    .or_insert(0) += 1;

                if is_cross_platform {
                    self.statistics.cross_platform_correlations += 1;
                }
            }
        }
    }

    /// Discover operation sequence patterns
    async fn discover_operation_sequences(&mut self, _event: &SecurityEvent) {
        // Placeholder for operation sequence discovery
        // This would analyze patterns like login -> authenticate -> access_resource
    }

    /// Discover component interaction patterns
    async fn discover_component_interactions(&mut self, _event: &SecurityEvent) {
        // Placeholder for component interaction discovery
        // This would analyze how different components interact across platforms
    }

    /// Clean up old correlation data
    async fn cleanup_old_data(&mut self) {
        let cutoff_time =
            Utc::now() - ChronoDuration::from_std(self.config.correlation_window).unwrap();

        // Clean up recent events cache
        while let Some((timestamp, _)) = self.recent_events.front() {
            if *timestamp < cutoff_time {
                self.recent_events.pop_front();
            } else {
                break;
            }
        }

        // Clean up old correlation groups
        let mut groups_to_remove = Vec::new();

        for (key, group) in &self.correlation_groups {
            if group.last_updated < cutoff_time {
                groups_to_remove.push(key.clone());

                // Remove event mappings
                for event in &group.events {
                    self.event_to_group.remove(&event.base_event().event_id);
                }
            }
        }

        for key in groups_to_remove {
            self.correlation_groups.remove(&key);
        }

        // Limit correlation groups if too many
        if self.correlation_groups.len() > self.config.max_correlation_groups {
            let excess = self.correlation_groups.len() - self.config.max_correlation_groups;

            // Remove oldest groups
            let mut groups_by_age: Vec<_> = self
                .correlation_groups
                .iter()
                .map(|(key, group)| (key.clone(), group.last_updated, group.events.clone()))
                .collect();
            groups_by_age.sort_by_key(|(_, last_updated, _)| *last_updated);

            // Collect keys to remove
            let keys_to_remove: Vec<String> = groups_by_age
                .iter()
                .take(excess)
                .map(|(key, _, _)| key.clone())
                .collect();

            // Remove event mappings and groups
            for key in keys_to_remove {
                if let Some(group) = self.correlation_groups.remove(&key) {
                    for event in &group.events {
                        self.event_to_group.remove(&event.base_event().event_id);
                    }
                }
            }
        }
    }
}

/// Search criteria for finding correlations
#[derive(Debug, Clone)]
pub struct CorrelationSearchCriteria {
    /// Filter by strategy
    pub strategy: Option<CorrelationStrategy>,
    /// Filter by platform
    pub platform: Option<PlatformSource>,
    /// Filter by event category
    pub category: Option<SecurityEventCategory>,
    /// Filter by cross-platform status
    pub cross_platform_only: bool,
    /// Minimum number of events in group
    pub min_events: Option<usize>,
    /// Time range filter
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

impl CorrelationSearchCriteria {
    /// Check if a correlation group matches the criteria
    pub fn matches(&self, group: &CorrelationGroup) -> bool {
        if let Some(strategy) = &self.strategy {
            if &group.strategy != strategy {
                return false;
            }
        }

        if let Some(platform) = &self.platform {
            if !group.platforms.contains(platform) {
                return false;
            }
        }

        if let Some(category) = &self.category {
            if !group.categories.contains(category) {
                return false;
            }
        }

        if self.cross_platform_only && !group.is_cross_platform() {
            return false;
        }

        if let Some(min_events) = self.min_events {
            if group.events.len() < min_events {
                return false;
            }
        }

        if let Some((start, end)) = &self.time_range {
            if group.created_at < *start || group.last_updated > *end {
                return false;
            }
        }

        true
    }
}

/// Convert correlation strategy to prefix for grouping
fn strategy_to_prefix(strategy: &CorrelationStrategy) -> &'static str {
    match strategy {
        CorrelationStrategy::TraceId => "trace",
        CorrelationStrategy::CorrelationId => "corr",
        CorrelationStrategy::SessionId => "sess",
        CorrelationStrategy::Actor => "actor",
        CorrelationStrategy::TemporalProximity => "temporal",
        CorrelationStrategy::OperationSequence => "sequence",
        CorrelationStrategy::ComponentInteraction => "component",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::event_types::{
        CreateVerificationEvent, PlatformSource, SecurityEvent,
        SecurityEventCategory, VerificationEvent,
    };
    use crate::security_types::Severity;

    #[tokio::test]
    async fn test_correlation_manager_creation() {
        let manager = CorrelationManager::new(Duration::from_secs(3600));
        assert_eq!(manager.config.correlation_window, Duration::from_secs(3600));
    }

    #[tokio::test]
    async fn test_trace_id_correlation() {
        let mut manager = CorrelationManager::new(Duration::from_secs(3600));

        let trace_id = "test-trace-123".to_string();

        let mut event1 = VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "auth".to_string(),
            "login".to_string(),
        );
        event1.trace_id = Some(trace_id.clone());

        let mut event2 = VerificationEvent::create_base_event(
            SecurityEventCategory::Authorization,
            Severity::Info,
            PlatformSource::JavaScriptSdk,
            "authz".to_string(),
            "check_permission".to_string(),
        );
        event2.trace_id = Some(trace_id);

        manager
            .add_event(&SecurityEvent::Generic(event1.clone()))
            .await;
        manager
            .add_event(&SecurityEvent::Generic(event2.clone()))
            .await;

        let correlations = manager.get_all_correlations().await;
        assert_eq!(correlations.len(), 1);
        assert_eq!(correlations[0].events.len(), 2);
        assert!(correlations[0].is_cross_platform());

        let stats = manager.get_statistics().await;
        assert_eq!(stats.total_correlations, 1);
        assert_eq!(stats.cross_platform_correlations, 1);
    }

    #[tokio::test]
    async fn test_correlation_search() {
        let mut manager = CorrelationManager::new(Duration::from_secs(3600));

        let mut event = VerificationEvent::create_base_event(
            SecurityEventCategory::Security,
            Severity::Critical,
            PlatformSource::DataFoldNode,
            "security".to_string(),
            "threat_detected".to_string(),
        );
        event.trace_id = Some("security-trace-456".to_string());

        manager.add_event(&SecurityEvent::Generic(event)).await;

        let criteria = CorrelationSearchCriteria {
            strategy: Some(CorrelationStrategy::TraceId),
            platform: None,
            category: Some(SecurityEventCategory::Security),
            cross_platform_only: false,
            min_events: None,
            time_range: None,
        };

        let results = manager.search_correlations(&criteria).await;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].strategy, CorrelationStrategy::TraceId);
    }

    #[test]
    fn test_correlation_group_operations() {
        let event1 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authentication,
            Severity::Info,
            PlatformSource::RustCli,
            "auth".to_string(),
            "login".to_string(),
        ));

        let mut group =
            CorrelationGroup::new("test-key".to_string(), CorrelationStrategy::TraceId, event1);

        assert_eq!(group.events.len(), 1);
        assert_eq!(group.platforms.len(), 1);
        assert!(!group.is_cross_platform());

        let event2 = SecurityEvent::Generic(VerificationEvent::create_base_event(
            SecurityEventCategory::Authorization,
            Severity::Info,
            PlatformSource::JavaScriptSdk,
            "authz".to_string(),
            "check_permission".to_string(),
        ));

        group.add_event(event2);

        assert_eq!(group.events.len(), 2);
        assert_eq!(group.platforms.len(), 2);
        assert!(group.is_cross_platform());

        let summary = group.summary();
        assert_eq!(summary.event_count, 2);
        assert_eq!(summary.platform_count, 2);
        assert!(summary.is_cross_platform);
    }
}
