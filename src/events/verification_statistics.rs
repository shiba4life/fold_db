//! Statistics Management for the Verification Event Bus
//!
//! This module contains the background statistics updater task logic.

use super::verification_bus_types::EventBusStatistics;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::interval;

/// Start the background statistics updater
pub async fn start_statistics_updater(
    statistics: Arc<RwLock<EventBusStatistics>>,
    start_time: DateTime<Utc>,
) -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    let handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60)); // Update every minute

        loop {
            interval.tick().await;

            let mut stats = statistics.write().await;
            stats.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;
        }
    });

    Ok(handle)
}

/// Update uptime statistics manually (useful for immediate updates)
pub async fn update_uptime(
    statistics: &Arc<RwLock<EventBusStatistics>>,
    start_time: DateTime<Utc>,
) {
    let mut stats = statistics.write().await;
    stats.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;
}

/// Clear all statistics while preserving essential data
pub async fn clear_statistics(statistics: &Arc<RwLock<EventBusStatistics>>) {
    let mut stats = statistics.write().await;
    stats.clear();
}

/// Get a snapshot of current statistics with updated uptime
pub async fn get_current_statistics(
    statistics: &Arc<RwLock<EventBusStatistics>>,
    start_time: DateTime<Utc>,
) -> EventBusStatistics {
    let mut stats = statistics.read().await.clone();
    stats.uptime_seconds = (Utc::now() - start_time).num_seconds() as u64;
    stats
}