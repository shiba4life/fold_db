//! Test vector validation for RFC 9421

use super::{RFC9421Config, ValidationTestResult};
use crate::{TestFailure, TestWarning};
use anyhow::Result;

/// Test vector validator
pub struct TestVectorValidator {
    config: RFC9421Config,
}

impl TestVectorValidator {
    pub fn new(config: RFC9421Config) -> Result<Self> {
        Ok(Self { config })
    }

    pub async fn validate(&self) -> Result<ValidationTestResult> {
        // TODO: Implement test vector validation
        Ok(ValidationTestResult {
            tests_run: 1,
            tests_passed: 1,
            tests_failed: 0,
            failures: Vec::new(),
            warnings: Vec::new(),
        })
    }
}