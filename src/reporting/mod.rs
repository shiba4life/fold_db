//! # Unified Reporting Module
//!
//! This module provides unified reporting structures and interfaces for all security modules,
//! compliance, performance monitoring, and validation systems. It serves as the foundation
//! for consistent reporting across the entire DataFold platform.
//!
//! ## Modules
//!
//! - [`types`] - Core reporting types, formats, and structures

pub mod types;

// Re-export commonly used types for convenience
pub use types::{
    AuditTrailSummary, ComplianceSummary, ExecutiveSummary, PerformanceSummary,
    ResolutionStatistics, SecurityIncidentSummary, ThreatSummary, TimeRange, UnifiedReport,
    UnifiedReportConfig, UnifiedReportFormat, UnifiedReportMetadata, UnifiedSummarySection,
};
