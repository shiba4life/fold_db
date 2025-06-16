//! Cryptographic configuration for DataFold database encryption
//!
//! This module provides configuration structures for master key encryption,
//! key derivation parameters, and crypto initialization settings.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::any::Any;
use std::collections::HashMap;
use std::path::Path;

use crate::config::error::ConfigError;
use crate::config::traits::base::{
    ConfigChangeType, ConfigMetadata, ReportingConfig, ValidationRule, ValidationRuleType,
    ValidationSeverity,
};
use crate::config::traits::integration::{
    AccessPattern, AccessType, ConfigComparison, ConfigSnapshot, ConfigTelemetry, DebugLevel,
    MonitoringSession,
};
use crate::config::traits::network::{
    ComplianceCheck, ComplianceCheckResult, ComplianceStatus, SecurityComplianceReport,
    SecurityConfig as SecurityConfigTrait, SecuritySeverity, SecurityStrengthAssessment,
    SecurityVulnerability,
};
use crate::config::traits::{
    BaseConfig, ConfigLifecycle, ConfigValidation, ObservableConfig, TraitConfigError,
    TraitConfigResult, ValidatableConfig, ValidationContext,
};
use crate::config::value::ConfigValue;
use crate::crypto::{argon2::Argon2Params, error::CryptoResult, CryptoError};
use crate::security_types::SecurityLevel;
use serde::{Deserialize, Serialize};

/// Top-level cryptographic configuration for database encryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    /// Whether database encryption is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Master key configuration for database encryption
    #[serde(default)]
    pub master_key: MasterKeyConfig,

    /// Key derivation configuration (when using passphrase-based keys)
    #[serde(default)]
    pub key_derivation: KeyDerivationConfig,

    // Enhanced trait support fields
    /// Configuration metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,
    /// Reporting configuration
    #[serde(default)]
    pub reporting_config: ReportingConfig,
    /// Validation rules
    #[serde(skip)]
    pub validation_rules: Vec<ValidationRule>,
    /// Security monitoring session
    #[serde(skip)]
    pub monitoring_session: Option<MonitoringSession>,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            master_key: MasterKeyConfig::default(),
            key_derivation: KeyDerivationConfig::default(),
            metadata: ConfigMetadata::default(),
            reporting_config: ReportingConfig::default(),
            validation_rules: Self::default_validation_rules(),
            monitoring_session: None,
        }
    }
}

fn default_enabled() -> bool {
    false
}

#[async_trait]
impl BaseConfig for CryptoConfig {
    type Error = ConfigError;
    type Event = ConfigChangeType;
    type TransformTarget = CryptoConfig;

    async fn load(path: &Path) -> Result<Self, Self::Error> {
        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::io(format!("Failed to read crypto config file: {}", e)))?;

        let mut config: CryptoConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::validation(format!("Failed to parse crypto config: {}", e))
        })?;

        // Set metadata
        config.metadata.source = Some(path.to_string_lossy().to_string());
        config.metadata.accessed_at = Utc::now();

        // Validate after loading - convert CryptoResult to ConfigError
        config
            .validate_crypto()
            .map_err(|e| ConfigError::validation(e.to_string()))?;

        Ok(config)
    }

    fn validate(&self) -> Result<(), Self::Error> {
        // Convert CryptoResult to ConfigError
        self.validate_crypto()
            .map_err(|e| ConfigError::validation(e.to_string()))
    }

    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::validation(format!("Failed to serialize crypto config: {}", e))
        })?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| ConfigError::io(format!("Failed to write crypto config file: {}", e)))
    }

    fn metadata(&self) -> HashMap<String, String> {
        let mut metadata = HashMap::new();
        metadata.insert("enabled".to_string(), self.enabled.to_string());
        metadata.insert(
            "master_key_type".to_string(),
            format!("{:?}", self.master_key),
        );
        metadata.insert(
            "key_derivation_type".to_string(),
            format!("{:?}", self.key_derivation),
        );
        metadata
    }

    fn merge(&self, other: &Self) -> Self {
        let mut result = self.clone();

        // Prefer the other config's values for enabled status
        result.enabled = other.enabled;

        // Merge master key config
        if matches!(
            other.master_key,
            MasterKeyConfig::Random | MasterKeyConfig::Passphrase { .. }
        ) {
            result.master_key = other.master_key.clone();
        }

        // Merge key derivation config
        result.key_derivation = other.key_derivation.clone();

        result
    }

    fn report_event(&self, event: Self::Event) {
        if self.reporting_config.report_changes {
            // Integration with unified reporting system would go here
            log::info!("Crypto config event: {:?}", event);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl ConfigLifecycle for CryptoConfig {
    async fn save(&self, path: &Path) -> Result<(), Self::Error> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::validation(format!("Failed to serialize crypto config: {}", e))
        })?;

        tokio::fs::write(path, content)
            .await
            .map_err(|e| ConfigError::io(format!("Failed to write crypto config file: {}", e)))?;

        Ok(())
    }

    async fn backup(&self, backup_path: &Path) -> Result<(), Self::Error> {
        self.save(backup_path).await
    }

    async fn merge(&mut self, other: Self) -> Result<(), Self::Error> {
        // Prefer the other config's values for enabled status
        self.enabled = other.enabled;

        // Merge master key config
        if matches!(
            other.master_key,
            MasterKeyConfig::Random | MasterKeyConfig::Passphrase { .. }
        ) {
            self.master_key = other.master_key;
        }

        // Merge key derivation config
        self.key_derivation = other.key_derivation;

        Ok(())
    }

    async fn reload(&mut self, path: &Path) -> Result<(), Self::Error> {
        let new_config = Self::load(path).await?;
        *self = new_config;
        Ok(())
    }

    async fn has_changed(&self, path: &Path) -> Result<bool, Self::Error> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| ConfigError::io(format!("Failed to read file metadata: {}", e)))?;

        let modified = metadata
            .modified()
            .map_err(|e| ConfigError::io(format!("Failed to get modification time: {}", e)))?;

        let modified_utc = DateTime::<Utc>::from(modified);
        Ok(modified_utc > self.metadata.updated_at)
    }

    fn get_metadata(&self) -> ConfigMetadata {
        self.metadata.clone()
    }

    fn set_metadata(&mut self, metadata: ConfigMetadata) {
        self.metadata = metadata;
    }
}

impl ConfigValidation for CryptoConfig {
    fn validate_with_context(&self) -> Result<(), ValidationContext> {
        let context = ValidationContext::new("CryptoConfig", "crypto_validation".to_string());

        self.validate().map_err(|_e| context)?;
        Ok(())
    }

    fn validate_field(&self, field_path: &str) -> Result<(), Self::Error> {
        match field_path {
            "enabled" => Ok(()), // Boolean field, always valid
            "master_key" => self
                .master_key
                .validate()
                .map_err(|e| ConfigError::validation(e.to_string())),
            "key_derivation" => self
                .key_derivation
                .validate()
                .map_err(|e| ConfigError::validation(e.to_string())),
            _ => Err(ConfigError::validation(format!(
                "Unknown field: {}",
                field_path
            ))),
        }
    }

    fn validation_rules(&self) -> Vec<ValidationRule> {
        self.validation_rules.clone()
    }

    fn add_validation_rule(&mut self, rule: ValidationRule) {
        self.validation_rules.push(rule);
    }
}

impl SecurityConfigTrait for CryptoConfig {
    fn validate_crypto_parameters(&self) -> TraitConfigResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Validate master key strength
        match &self.master_key {
            MasterKeyConfig::Passphrase { passphrase } => {
                if passphrase.len() < 12 {
                    return Err(TraitConfigError::trait_validation(
                        "Passphrase should be at least 12 characters for strong security",
                        Some(
                            ValidationContext::new(
                                "SecurityConfig",
                                "passphrase_strength".to_string(),
                            )
                            .with_path("master_key.passphrase")
                            .with_expected(">=12 characters")
                            .with_actual(&format!("{} characters", passphrase.len())),
                        ),
                    ));
                }

                // Check for common weak patterns
                if passphrase.to_lowercase().contains("password")
                    || passphrase.to_lowercase().contains("123456")
                    || passphrase == "admin"
                {
                    return Err(TraitConfigError::trait_validation(
                        "Passphrase contains common weak patterns",
                        Some(
                            ValidationContext::new(
                                "SecurityConfig",
                                "passphrase_patterns".to_string(),
                            )
                            .with_path("master_key.passphrase"),
                        ),
                    ));
                }
            }
            MasterKeyConfig::External { key_source } => {
                if key_source.is_empty() {
                    return Err(TraitConfigError::trait_validation(
                        "External key source cannot be empty",
                        Some(
                            ValidationContext::new(
                                "SecurityConfig",
                                "external_key_source".to_string(),
                            )
                            .with_path("master_key.key_source"),
                        ),
                    ));
                }
            }
            MasterKeyConfig::Random => {
                // Random keys are always secure
            }
        }

        // Validate key derivation parameters for security
        if let Some(preset) = &self.key_derivation.preset {
            match preset {
                SecurityLevel::Low => {
                    log::warn!("Using low security level for key derivation");
                }
                SecurityLevel::Standard => {
                    // Acceptable for most use cases
                }
                SecurityLevel::High => {
                    // Recommended for sensitive data
                }
            }
        }

        Ok(())
    }

    fn validate_security_policy(&self) -> TraitConfigResult<()> {
        // Ensure encryption is enabled for production environments
        if !self.enabled {
            log::warn!("Encryption is disabled - data will be stored in plaintext");
        }

        // Validate that secure key derivation parameters are used
        if self.enabled && self.requires_passphrase() {
            let params = self.key_derivation.to_argon2_params().map_err(|e| {
                TraitConfigError::trait_validation(
                    format!("Invalid key derivation parameters: {}", e),
                    Some(ValidationContext::new(
                        "SecurityConfig",
                        "key_derivation_params".to_string(),
                    )),
                )
            })?;

            // Check minimum security requirements
            if params.memory_cost < 32768 {
                // 32MB minimum
                return Err(TraitConfigError::trait_validation(
                    "Key derivation memory cost too low for secure operation",
                    Some(
                        ValidationContext::new("SecurityConfig", "memory_cost_minimum".to_string())
                            .with_path("key_derivation.memory_cost")
                            .with_expected(">=32768 KB")
                            .with_actual(&format!("{} KB", params.memory_cost)),
                    ),
                ));
            }
        }

        Ok(())
    }

    fn check_security_vulnerabilities(&self) -> TraitConfigResult<Vec<SecurityVulnerability>> {
        let mut vulnerabilities = Vec::new();

        // Check for disabled encryption
        if !self.enabled {
            vulnerabilities.push(SecurityVulnerability {
                id: "CRYPTO_001".to_string(),
                severity: SecuritySeverity::High,
                description: "Database encryption is disabled".to_string(),
                affected_fields: vec!["enabled".to_string()],
                remediation: vec!["Enable encryption by setting 'enabled = true'".to_string()],
                cvss_score: Some(7.5),
            });
        }

        // Check for weak passphrase
        if let MasterKeyConfig::Passphrase { passphrase } = &self.master_key {
            if passphrase.len() < 8 {
                vulnerabilities.push(SecurityVulnerability {
                    id: "CRYPTO_002".to_string(),
                    severity: SecuritySeverity::Critical,
                    description: "Weak passphrase detected".to_string(),
                    affected_fields: vec!["master_key.passphrase".to_string()],
                    remediation: vec![
                        "Use a passphrase with at least 12 characters".to_string(),
                        "Include uppercase, lowercase, numbers, and symbols".to_string(),
                    ],
                    cvss_score: Some(9.1),
                });
            }
        }

        // Check for weak key derivation parameters
        if self.enabled && self.key_derivation.memory_cost < 32768 {
            vulnerabilities.push(SecurityVulnerability {
                id: "CRYPTO_003".to_string(),
                severity: SecuritySeverity::Medium,
                description: "Weak key derivation parameters".to_string(),
                affected_fields: vec!["key_derivation.memory_cost".to_string()],
                remediation: vec!["Increase memory cost to at least 32768 KB".to_string()],
                cvss_score: Some(5.3),
            });
        }

        Ok(vulnerabilities)
    }

    fn assess_security_strength(&self) -> TraitConfigResult<SecurityStrengthAssessment> {
        let mut assessment = SecurityStrengthAssessment::new();
        let mut total_score = 0u32;
        let mut max_score = 0u32;

        // Encryption enabled (30 points)
        max_score += 30;
        if self.enabled {
            total_score += 30;
            assessment
                .strengths
                .push("Database encryption is enabled".to_string());
        } else {
            assessment
                .weaknesses
                .push("Database encryption is disabled".to_string());
            assessment
                .recommendations
                .push("Enable database encryption".to_string());
        }

        // Key strength (40 points)
        max_score += 40;
        match &self.master_key {
            MasterKeyConfig::Random => {
                total_score += 40;
                assessment
                    .strengths
                    .push("Using cryptographically secure random key generation".to_string());
            }
            MasterKeyConfig::Passphrase { passphrase } => {
                let passphrase_score = if passphrase.len() >= 16 {
                    40
                } else if passphrase.len() >= 12 {
                    30
                } else if passphrase.len() >= 8 {
                    20
                } else {
                    0
                };
                total_score += passphrase_score;

                if passphrase_score >= 30 {
                    assessment
                        .strengths
                        .push("Strong passphrase length".to_string());
                } else {
                    assessment
                        .weaknesses
                        .push("Weak passphrase length".to_string());
                    assessment
                        .recommendations
                        .push("Use a longer passphrase (16+ characters)".to_string());
                }
            }
            MasterKeyConfig::External { .. } => {
                total_score += 35; // Assume external key management is good
                assessment
                    .strengths
                    .push("Using external key management".to_string());
            }
        }

        // Key derivation strength (30 points)
        max_score += 30;
        if let Some(preset) = &self.key_derivation.preset {
            let kdf_score = match preset {
                SecurityLevel::High => 30,
                SecurityLevel::Standard => 20,
                SecurityLevel::Low => 10,
            };
            total_score += kdf_score;

            if kdf_score >= 20 {
                assessment.strengths.push(format!(
                    "Using {:?} security level for key derivation",
                    preset
                ));
            } else {
                assessment
                    .weaknesses
                    .push("Low security level for key derivation".to_string());
                assessment
                    .recommendations
                    .push("Use Standard or High security level".to_string());
            }
        } else {
            // Custom parameters - evaluate based on memory cost
            let kdf_score = if self.key_derivation.memory_cost >= 131072 {
                30
            } else if self.key_derivation.memory_cost >= 65536 {
                20
            } else if self.key_derivation.memory_cost >= 32768 {
                15
            } else {
                5
            };
            total_score += kdf_score;
        }

        assessment.overall_score = ((total_score as f64 / max_score as f64) * 100.0) as u8;
        assessment
            .component_scores
            .insert("encryption".to_string(), if self.enabled { 100 } else { 0 });
        assessment.component_scores.insert(
            "key_strength".to_string(),
            ((total_score.min(40) as f64 / 40.0) * 100.0) as u8,
        );

        Ok(assessment)
    }

    fn validate_key_management(&self) -> TraitConfigResult<()> {
        // Validate key rotation policies would go here
        // For now, just check that external key sources are accessible
        if let MasterKeyConfig::External { key_source } = &self.master_key {
            // In a real implementation, this would check if the external source is accessible
            log::info!("Using external key source: {}", key_source);
        }

        Ok(())
    }

    fn generate_compliance_report(&self) -> TraitConfigResult<SecurityComplianceReport> {
        let mut report = SecurityComplianceReport {
            framework: "DataFold Security Policy".to_string(),
            compliance_status: ComplianceStatus::Compliant,
            compliance_score: 100,
            checks: Vec::new(),
            non_compliant_items: Vec::new(),
            recommendations: Vec::new(),
            generated_at: Utc::now(),
        };

        // Check encryption requirement
        let encryption_check = ComplianceCheck {
            check_id: "SEC-001".to_string(),
            description: "Database encryption must be enabled".to_string(),
            result: if self.enabled {
                ComplianceCheckResult::Pass
            } else {
                ComplianceCheckResult::Fail
            },
            evidence: Some(format!("Encryption enabled: {}", self.enabled)),
            remediation: if self.enabled {
                None
            } else {
                Some("Enable database encryption".to_string())
            },
        };

        if encryption_check.result == ComplianceCheckResult::Fail {
            report.compliance_status = ComplianceStatus::NonCompliant;
            report.compliance_score = 0;
            report
                .non_compliant_items
                .push("Database encryption disabled".to_string());
            report
                .recommendations
                .push("Enable database encryption immediately".to_string());
        }

        report.checks.push(encryption_check);

        // Check key strength
        let key_strength_check = ComplianceCheck {
            check_id: "SEC-002".to_string(),
            description: "Cryptographic keys must meet minimum strength requirements".to_string(),
            result: match &self.master_key {
                MasterKeyConfig::Random => ComplianceCheckResult::Pass,
                MasterKeyConfig::Passphrase { passphrase } => {
                    if passphrase.len() >= 12 {
                        ComplianceCheckResult::Pass
                    } else {
                        ComplianceCheckResult::Fail
                    }
                }
                MasterKeyConfig::External { .. } => ComplianceCheckResult::Pass,
            },
            evidence: Some("Key strength assessed".to_string()),
            remediation: None,
        };

        report.checks.push(key_strength_check);

        Ok(report)
    }
}

impl CryptoConfig {
    /// Create a new crypto configuration with encryption disabled
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            ..Default::default()
        }
    }

    /// Create a new crypto configuration with passphrase-based encryption
    pub fn with_passphrase(passphrase: String) -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase { passphrase },
            key_derivation: KeyDerivationConfig::for_security_level(SecurityLevel::Standard),
            ..Default::default()
        }
    }

    /// Create a new crypto configuration with random key generation
    pub fn with_random_key() -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Random,
            key_derivation: KeyDerivationConfig::default(),
            ..Default::default()
        }
    }

    /// Create a new crypto configuration with enhanced security parameters
    pub fn with_enhanced_security(passphrase: String) -> Self {
        Self {
            enabled: true,
            master_key: MasterKeyConfig::Passphrase { passphrase },
            key_derivation: KeyDerivationConfig::sensitive(),
            ..Default::default()
        }
    }

    /// Validate the crypto configuration (internal method for compatibility)
    pub fn validate_crypto(&self) -> CryptoResult<()> {
        if !self.enabled {
            return Ok(());
        }

        // Validate master key configuration
        self.master_key.validate()?;

        // Validate key derivation parameters
        self.key_derivation.validate()?;

        Ok(())
    }

    /// Check if the configuration requires a passphrase
    pub fn requires_passphrase(&self) -> bool {
        self.enabled && matches!(self.master_key, MasterKeyConfig::Passphrase { .. })
    }

    /// Check if the configuration uses random key generation
    pub fn uses_random_key(&self) -> bool {
        self.enabled && matches!(self.master_key, MasterKeyConfig::Random)
    }

    /// Default validation rules for crypto configuration
    fn default_validation_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                name: "passphrase_strength".to_string(),
                description: "Passphrase must meet minimum strength requirements".to_string(),
                field_path: "master_key.passphrase".to_string(),
                rule_type: ValidationRuleType::StringLength {
                    min: Some(8),
                    max: None,
                },
                severity: ValidationSeverity::Error,
            },
            ValidationRule {
                name: "key_derivation_memory".to_string(),
                description: "Key derivation memory cost should be sufficient".to_string(),
                field_path: "key_derivation.memory_cost".to_string(),
                rule_type: ValidationRuleType::NumericRange {
                    min: Some(32768.0),
                    max: None,
                },
                severity: ValidationSeverity::Warning,
            },
            ValidationRule {
                name: "external_key_source".to_string(),
                description: "External key source must be specified".to_string(),
                field_path: "master_key.key_source".to_string(),
                rule_type: ValidationRuleType::Required,
                severity: ValidationSeverity::Error,
            },
        ]
    }
}

/// Configuration for master key generation and management
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MasterKeyConfig {
    /// Generate a random master key pair (highest security, no password recovery)
    Random,

    /// Derive master key from user passphrase (allows password recovery)
    Passphrase {
        /// The passphrase to use for key derivation
        /// Note: In production, this should be provided at runtime, not stored
        passphrase: String,
    },

    /// Use an existing key pair from external source (advanced use case)
    External {
        /// Path to the external key file or key identifier
        key_source: String,
    },
}

impl Default for MasterKeyConfig {
    fn default() -> Self {
        Self::Random
    }
}

impl MasterKeyConfig {
    /// Validate the master key configuration
    pub fn validate(&self) -> CryptoResult<()> {
        match self {
            Self::Random => Ok(()),
            Self::Passphrase { passphrase } => {
                if passphrase.is_empty() {
                    return Err(CryptoError::InvalidKey {
                        message: "Passphrase cannot be empty".to_string(),
                    });
                }

                if passphrase.len() < 6 {
                    return Err(CryptoError::InvalidKey {
                        message: "Passphrase must be at least 6 characters".to_string(),
                    });
                }

                Ok(())
            }
            Self::External { key_source } => {
                if key_source.is_empty() {
                    return Err(CryptoError::InvalidKey {
                        message: "External key source cannot be empty".to_string(),
                    });
                }

                Ok(())
            }
        }
    }
}

/// Configuration for key derivation parameters (Argon2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    /// Memory cost in KB (minimum 8 KB, recommended 64 MB+)
    #[serde(default = "default_memory_cost")]
    pub memory_cost: u32,

    /// Time cost (iterations, minimum 1, recommended 3+)
    #[serde(default = "default_time_cost")]
    pub time_cost: u32,

    /// Parallelism degree (threads, minimum 1, recommended 4)
    #[serde(default = "default_parallelism")]
    pub parallelism: u32,

    /// Security level preset (overrides individual parameters if set)
    #[serde(default)]
    pub preset: Option<SecurityLevel>,
}

fn default_memory_cost() -> u32 {
    65536 // 64 MB
}

fn default_time_cost() -> u32 {
    3
}

fn default_parallelism() -> u32 {
    4
}

impl Default for KeyDerivationConfig {
    fn default() -> Self {
        Self {
            memory_cost: default_memory_cost(),
            time_cost: default_time_cost(),
            parallelism: default_parallelism(),
            preset: None,
        }
    }
}

impl KeyDerivationConfig {
    /// Create configuration optimized for interactive use (faster)
    pub fn interactive() -> Self {
        Self {
            memory_cost: 32768, // 32 MB
            time_cost: 2,
            parallelism: 2,
            preset: Some(SecurityLevel::Low),
        }
    }

    /// Create configuration optimized for sensitive operations (slower, more secure)
    pub fn sensitive() -> Self {
        Self {
            memory_cost: 131_072, // 128 MB
            time_cost: 4,
            parallelism: 8,
            preset: Some(SecurityLevel::High),
        }
    }

    /// Create configuration for a specific security level
    pub fn for_security_level(level: SecurityLevel) -> Self {
        match level {
            SecurityLevel::Low => Self::interactive(),
            SecurityLevel::Standard => Self {
                preset: Some(SecurityLevel::Standard),
                ..Default::default()
            },
            SecurityLevel::High => Self::sensitive(),
        }
    }

    /// Create configuration with custom parameters
    pub fn custom(memory_cost: u32, time_cost: u32, parallelism: u32) -> CryptoResult<Self> {
        let config = Self {
            memory_cost,
            time_cost,
            parallelism,
            preset: None,
        };

        config.validate()?;
        Ok(config)
    }

    /// Validate the key derivation configuration
    pub fn validate(&self) -> CryptoResult<()> {
        // Use preset parameters if specified
        let (memory_cost, time_cost, parallelism) = if let Some(preset) = &self.preset {
            match preset {
                SecurityLevel::Low => (32768, 2, 2),
                SecurityLevel::Standard => (65536, 3, 4),
                SecurityLevel::High => (131_072, 4, 8),
            }
        } else {
            (self.memory_cost, self.time_cost, self.parallelism)
        };

        // Validate using Argon2Params validation
        Argon2Params::new(memory_cost, time_cost, parallelism)?;

        Ok(())
    }

    /// Convert to Argon2Params for use with crypto module
    pub fn to_argon2_params(&self) -> CryptoResult<Argon2Params> {
        let (memory_cost, time_cost, parallelism) = if let Some(preset) = &self.preset {
            match preset {
                SecurityLevel::Low => return Ok(Argon2Params::interactive()),
                SecurityLevel::Standard => return Ok(Argon2Params::default()),
                SecurityLevel::High => return Ok(Argon2Params::sensitive()),
            }
        } else {
            (self.memory_cost, self.time_cost, self.parallelism)
        };

        Argon2Params::new(memory_cost, time_cost, parallelism)
    }
}

/// Legacy crypto config error type alias for backward compatibility
pub type CryptoConfigError = crate::config::error::ConfigError;

/// Result type for crypto configuration operations
pub type CryptoConfigResult<T> = Result<T, CryptoConfigError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_config_default() {
        let config = CryptoConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.master_key, MasterKeyConfig::Random));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_with_passphrase() {
        let config = CryptoConfig::with_passphrase("test-passphrase-123".to_string());
        assert!(config.enabled);
        assert!(config.requires_passphrase());
        assert!(!config.uses_random_key());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_with_random_key() {
        let config = CryptoConfig::with_random_key();
        assert!(config.enabled);
        assert!(!config.requires_passphrase());
        assert!(config.uses_random_key());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_crypto_config_enhanced_security() {
        let config = CryptoConfig::with_enhanced_security("strong-passphrase".to_string());
        assert!(config.enabled);
        assert!(config.requires_passphrase());
        assert_eq!(config.key_derivation.preset, Some(SecurityLevel::High));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_master_key_config_validation() {
        // Valid passphrase
        let config = MasterKeyConfig::Passphrase {
            passphrase: "valid-passphrase".to_string(),
        };
        assert!(config.validate().is_ok());

        // Empty passphrase should fail
        let config = MasterKeyConfig::Passphrase {
            passphrase: "".to_string(),
        };
        assert!(config.validate().is_err());

        // Very short passphrase should fail
        let config = MasterKeyConfig::Passphrase {
            passphrase: "tiny".to_string(),
        };
        assert!(config.validate().is_err());

        // Random key should always be valid
        let config = MasterKeyConfig::Random;
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_key_derivation_config_presets() {
        let interactive = KeyDerivationConfig::interactive();
        assert_eq!(interactive.preset, Some(SecurityLevel::Low));
        assert!(interactive.validate().is_ok());

        let sensitive = KeyDerivationConfig::sensitive();
        assert_eq!(sensitive.preset, Some(SecurityLevel::High));
        assert!(sensitive.validate().is_ok());

        let default = KeyDerivationConfig::default();
        assert_eq!(default.preset, None);
        assert!(default.validate().is_ok());
    }

    #[test]
    fn test_key_derivation_config_to_argon2_params() {
        let config = KeyDerivationConfig::interactive();
        let params = config
            .to_argon2_params()
            .expect("Should convert to Argon2Params");
        assert_eq!(params.memory_cost, 32768);
        assert_eq!(params.time_cost, 2);
        assert_eq!(params.parallelism, 2);

        let config = KeyDerivationConfig::sensitive();
        let params = config
            .to_argon2_params()
            .expect("Should convert to Argon2Params");
        assert_eq!(params.memory_cost, 131072);
        assert_eq!(params.time_cost, 4);
        assert_eq!(params.parallelism, 8);
    }

    #[test]
    fn test_key_derivation_config_custom() {
        let config = KeyDerivationConfig::custom(1024, 2, 2).expect("Should create custom config");
        assert_eq!(config.memory_cost, 1024);
        assert_eq!(config.time_cost, 2);
        assert_eq!(config.parallelism, 2);
        assert!(config.validate().is_ok());

        // Invalid parameters should fail
        assert!(KeyDerivationConfig::custom(7, 2, 2).is_err()); // Memory too low
        assert!(KeyDerivationConfig::custom(1024, 0, 2).is_err()); // Time too low
        assert!(KeyDerivationConfig::custom(1024, 2, 0).is_err()); // Parallelism too low
    }

    #[test]
    fn test_config_serialization() {
        let config = CryptoConfig::with_passphrase("test-passphrase".to_string());

        let json = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: CryptoConfig = serde_json::from_str(&json).expect("Should deserialize");

        assert_eq!(config.enabled, deserialized.enabled);
        assert!(matches!(
            deserialized.master_key,
            MasterKeyConfig::Passphrase { .. }
        ));
    }

    #[test]
    fn test_disabled_config_validation() {
        let config = CryptoConfig::disabled();
        assert!(!config.enabled);
        assert!(config.validate().is_ok());

        // Even with invalid passphrase, disabled config should pass validation
        let config = CryptoConfig {
            master_key: MasterKeyConfig::Passphrase {
                passphrase: "".to_string(), // Invalid but should be ignored when disabled
            },
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }
}
