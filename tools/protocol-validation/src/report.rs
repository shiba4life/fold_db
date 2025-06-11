//! Report generation for validation results

use crate::{ValidationResult, OutputConfig};
use anyhow::Result;
use serde_json;
use std::fs;

/// Report generator for validation results
pub struct ReportGenerator {
    config: OutputConfig,
}

impl ReportGenerator {
    pub fn new(config: OutputConfig) -> Self {
        Self { config }
    }

    /// Generate all configured reports
    pub async fn generate_all_reports(&self, result: &ValidationResult) -> Result<()> {
        // Ensure output directory exists
        fs::create_dir_all(&self.config.output_directory)?;

        if self.config.generate_json_report {
            self.generate_json_report(result).await?;
        }

        if self.config.generate_html_report {
            self.generate_html_report(result).await?;
        }

        if self.config.generate_junit_xml {
            self.generate_junit_report(result).await?;
        }

        Ok(())
    }

    /// Generate JSON report
    async fn generate_json_report(&self, result: &ValidationResult) -> Result<()> {
        let json = serde_json::to_string_pretty(result)?;
        let path = format!("{}/validation-report.json", self.config.output_directory);
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Generate HTML report
    async fn generate_html_report(&self, result: &ValidationResult) -> Result<()> {
        let html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>DataFold Protocol Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ background: #f5f5f5; padding: 20px; border-radius: 5px; }}
        .success {{ color: green; }}
        .warning {{ color: orange; }}
        .error {{ color: red; }}
        .category {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>DataFold Protocol Validation Report</h1>
        <p><strong>Status:</strong> <span class="{}">{:?}</span></p>
        <p><strong>Duration:</strong> {}ms</p>
        <p><strong>Success Rate:</strong> {:.1}%</p>
    </div>
    <div class="summary">
        <h2>Summary</h2>
        <p>Total Tests: {}</p>
        <p>Passed: {}</p>
        <p>Failed: {}</p>
        <p>Warnings: {}</p>
    </div>
</body>
</html>"#,
            match result.overall_status {
                crate::ValidationStatus::Passed => "success",
                crate::ValidationStatus::Warning => "warning",
                _ => "error",
            },
            result.overall_status,
            result.duration_ms,
            result.summary.success_rate,
            result.summary.total_tests,
            result.summary.total_passed,
            result.summary.total_failed,
            result.summary.warnings,
        );

        let path = format!("{}/validation-report.html", self.config.output_directory);
        tokio::fs::write(path, html).await?;
        Ok(())
    }

    /// Generate JUnit XML report
    async fn generate_junit_report(&self, result: &ValidationResult) -> Result<()> {
        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="DataFold Protocol Validation" tests="{}" failures="{}" time="{}">
    <testsuite name="validation" tests="{}" failures="{}" time="{}">
    </testsuite>
</testsuites>"#,
            result.summary.total_tests,
            result.summary.total_failed,
            result.duration_ms as f64 / 1000.0,
            result.summary.total_tests,
            result.summary.total_failed,
            result.duration_ms as f64 / 1000.0,
        );

        let path = format!("{}/validation-report.xml", self.config.output_directory);
        tokio::fs::write(path, xml).await?;
        Ok(())
    }
}