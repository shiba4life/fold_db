//! CLI integration with unified configuration system
//!
//! This module provides seamless integration between the DataFold CLI
//! and the unified configuration system for signature authentication.

use crate::config::unified_config::{UnifiedConfigManager, UnifiedConfigError, EnvironmentConfig};
use crate::cli::auth::{CliAuthProfile, CliSigningConfig, SignatureComponent};
use crate::cli::config::{CliConfigManager, CliConfigError, SignatureSettings};
use crate::cli::signing_config::{EnhancedSigningConfig, SigningMode, AutoSigningConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Errors that can occur during unified CLI integration
#[derive(Debug, thiserror::Error)]
pub enum UnifiedIntegrationError {
    #[error("Unified config error: {0}")]
    UnifiedConfig(#[from] UnifiedConfigError),
    
    #[error("CLI config error: {0}")]
    CliConfig(#[from] CliConfigError),
    
    #[error("Environment not found: {0}")]
    EnvironmentNotFound(String),
    
    #[error("Profile not configured for environment: {0}")]
    ProfileNotConfigured(String),
    
    #[error("Mandatory authentication not configured")]
    MandatoryAuthNotConfigured,
    
    #[error("Invalid signature configuration: {0}")]
    InvalidSignatureConfig(String),
}

pub type UnifiedIntegrationResult<T> = Result<T, UnifiedIntegrationError>;

/// Unified CLI configuration that combines CLI config with unified config
pub struct UnifiedCliConfig {
    /// Unified configuration manager
    pub unified_manager: UnifiedConfigManager,
    /// CLI configuration manager
    pub cli_manager: CliConfigManager,
    /// Current environment
    pub current_environment: String,
}

impl UnifiedCliConfig {
    /// Create a new unified CLI configuration
    pub fn new() -> UnifiedIntegrationResult<Self> {
        let unified_manager = UnifiedConfigManager::load_default()?;
        let cli_manager = CliConfigManager::load_with_migration()?;
        let current_environment = unified_manager.current_environment().to_string();
        
        Ok(Self {
            unified_manager,
            cli_manager,
            current_environment,
        })
    }
    
    /// Create from specific config paths
    pub fn from_paths<P1, P2>(unified_path: P1, cli_path: P2) -> UnifiedIntegrationResult<Self>
    where
        P1: AsRef<std::path::Path>,
        P2: AsRef<std::path::Path>,
    {
        let unified_manager = UnifiedConfigManager::load_from_file(unified_path)?;
        let cli_manager = CliConfigManager::with_path(cli_path)?;
        let current_environment = unified_manager.current_environment().to_string();
        
        Ok(Self {
            unified_manager,
            cli_manager,
            current_environment,
        })
    }
    
    /// Switch to a different environment
    pub fn switch_environment(&mut self, environment: &str) -> UnifiedIntegrationResult<()> {
        self.unified_manager.set_environment(environment.to_string())?;
        self.current_environment = environment.to_string();
        Ok(())
    }
    
    /// Get current environment configuration
    pub fn current_environment_config(&self) -> UnifiedIntegrationResult<&EnvironmentConfig> {
        self.unified_manager.current_environment_config()
            .map_err(UnifiedIntegrationError::UnifiedConfig)
    }
    
    /// Get signing configuration for current environment
    pub fn get_signing_config(&self) -> UnifiedIntegrationResult<CliSigningConfig> {
        let env_config = self.current_environment_config()?;
        let signing_config = &env_config.signing;
        
        // Convert unified config to CLI signing config
        let required_components = signing_config.required_components
            .iter()
            .map(|s| match s.as_str() {
                "@method" => SignatureComponent::Method,
                "@target-uri" => SignatureComponent::TargetUri,
                "@authority" => SignatureComponent::Authority,
                "@scheme" => SignatureComponent::Scheme,
                "@path" => SignatureComponent::Path,
                "@query" => SignatureComponent::Query,
                header => SignatureComponent::Header(header.to_string()),
            })
            .collect();
        
        Ok(CliSigningConfig {
            required_components,
            include_content_digest: signing_config.include_content_digest,
            include_timestamp: signing_config.include_timestamp,
            include_nonce: signing_config.include_nonce,
            max_body_size: (signing_config.max_body_size_mb * 1024 * 1024) as usize,
        })
    }
    
    /// Get enhanced signing configuration with unified config integration
    pub fn get_enhanced_signing_config(&self) -> UnifiedIntegrationResult<EnhancedSigningConfig> {
        let env_config = self.current_environment_config()?;
        let base_config = self.get_signing_config()?;
        
        // For mandatory authentication, all signing should be automatic
        let auto_signing = AutoSigningConfig {
            enabled: true,
            default_mode: SigningMode::Auto, // Mandatory signing means always auto
            command_overrides: HashMap::new(),
            env_override: None, // No environment override for mandatory auth
            prompt_on_first_sign: env_config.authentication.prompt_on_first_sign,
            default_profile: self.cli_manager.config().default_profile.clone(),
        };
        
        Ok(EnhancedSigningConfig {
            base: base_config,
            auto_signing,
            debug: crate::cli::signing_config::SigningDebugConfig {
                enabled: env_config.signing.debug.enabled,
                log_canonical_strings: env_config.signing.debug.log_canonical_strings,
                log_components: env_config.signing.debug.log_components,
                log_timing: env_config.signing.debug.log_timing,
                debug_output_dir: None,
            },
            performance: crate::cli::signing_config::SigningPerformanceConfig {
                max_signing_time_ms: env_config.signing.timeout_ms,
                cache_keys: env_config.performance.cache_keys,
                max_concurrent_signs: env_config.performance.max_concurrent_signs,
            },
        })
    }
    
    /// Get CLI authentication profile for current environment
    pub fn get_auth_profile(&self) -> UnifiedIntegrationResult<Option<CliAuthProfile>> {
        if let Some(profile_name) = &self.cli_manager.config().default_profile {
            Ok(self.cli_manager.get_profile(profile_name).cloned())
        } else {
            Ok(None)
        }
    }
    
    /// Ensure mandatory authentication is properly configured
    pub fn validate_mandatory_auth(&self) -> UnifiedIntegrationResult<()> {
        // Check if we have a default profile configured
        let profile = self.get_auth_profile()?
            .ok_or(UnifiedIntegrationError::MandatoryAuthNotConfigured)?;
        
        // Validate profile has required fields
        if profile.client_id.is_empty() {
            return Err(UnifiedIntegrationError::InvalidSignatureConfig(
                "Client ID is required for mandatory authentication".to_string()
            ));
        }
        
        if profile.key_id.is_empty() {
            return Err(UnifiedIntegrationError::InvalidSignatureConfig(
                "Key ID is required for mandatory authentication".to_string()
            ));
        }
        
        if profile.server_url.is_empty() {
            return Err(UnifiedIntegrationError::InvalidSignatureConfig(
                "Server URL is required for mandatory authentication".to_string()
            ));
        }
        
        // Ensure current environment has valid signing configuration
        let env_config = self.current_environment_config()?;
        
        if env_config.signing.required_components.is_empty() {
            return Err(UnifiedIntegrationError::InvalidSignatureConfig(
                "At least one signature component is required".to_string()
            ));
        }
        
        if env_config.signing.timeout_ms == 0 {
            return Err(UnifiedIntegrationError::InvalidSignatureConfig(
                "Signing timeout must be greater than 0".to_string()
            ));
        }
        
        Ok(())
    }
    
    /// Get signature settings for CLI configuration
    pub fn get_signature_settings(&self) -> UnifiedIntegrationResult<SignatureSettings> {
        let env_config = self.current_environment_config()?;
        
        Ok(SignatureSettings {
            include_timestamp: env_config.signing.include_timestamp,
            include_nonce: env_config.signing.include_nonce,
            default_components: env_config.signing.required_components.clone(),
            max_body_size_mb: env_config.signing.max_body_size_mb,
            verify_responses: env_config.verification.strict_timing,
        })
    }
    
    /// List all available environments
    pub fn list_environments(&self) -> Vec<String> {
        self.unified_manager.list_environments().into_iter().cloned().collect()
    }
    
    /// Get environment summary for display
    pub fn get_environment_summary(&self) -> UnifiedIntegrationResult<EnvironmentSummary> {
        let env_config = self.current_environment_config()?;
        let profile = self.get_auth_profile()?;
        
        Ok(EnvironmentSummary {
            name: self.current_environment.clone(),
            signing_policy: env_config.signing.policy.clone(),
            mandatory_auth: true, // Always true for T11.4
            profile_configured: profile.is_some(),
            client_id: profile.as_ref().map(|p| p.client_id.clone()),
            server_url: profile.as_ref().map(|p| p.server_url.clone()),
            components_count: env_config.signing.required_components.len(),
            include_timestamp: env_config.signing.include_timestamp,
            include_nonce: env_config.signing.include_nonce,
            timeout_ms: env_config.signing.timeout_ms,
        })
    }
    
    /// Save CLI configuration changes
    pub fn save_cli_config(&mut self) -> UnifiedIntegrationResult<()> {
        self.cli_manager.save().map_err(UnifiedIntegrationError::CliConfig)
    }
    
    /// Access CLI configuration manager
    pub fn cli_config_mut(&mut self) -> &mut CliConfigManager {
        &mut self.cli_manager
    }
    
    /// Access unified configuration manager
    pub fn unified_config(&self) -> &UnifiedConfigManager {
        &self.unified_manager
    }
}

/// Summary of current environment configuration for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentSummary {
    /// Environment name
    pub name: String,
    /// Signing policy (minimal, standard, strict)
    pub signing_policy: String,
    /// Whether mandatory authentication is enabled
    pub mandatory_auth: bool,
    /// Whether authentication profile is configured
    pub profile_configured: bool,
    /// Client ID if configured
    pub client_id: Option<String>,
    /// Server URL if configured
    pub server_url: Option<String>,
    /// Number of required signature components
    pub components_count: usize,
    /// Whether timestamp is included in signatures
    pub include_timestamp: bool,
    /// Whether nonce is included in signatures
    pub include_nonce: bool,
    /// Signing timeout in milliseconds
    pub timeout_ms: u64,
}

/// Utility functions for CLI integration
pub mod integration_utils {
    use super::*;
    
    /// Initialize unified CLI configuration for a command
    pub fn init_for_command(_command: &str) -> UnifiedIntegrationResult<UnifiedCliConfig> {
        let config = UnifiedCliConfig::new()?;
        
        // Validate that mandatory authentication is properly configured
        config.validate_mandatory_auth().map_err(|e| {
            eprintln!("❌ Mandatory authentication not properly configured: {}", e);
            eprintln!("💡 Run 'datafold auth init' to set up authentication");
            e
        })?;
        
        Ok(config)
    }
    
    /// Create authenticated HTTP client from unified configuration
    pub async fn create_authenticated_client(
        config: &UnifiedCliConfig,
        keypair: crate::crypto::ed25519::MasterKeyPair,
    ) -> UnifiedIntegrationResult<crate::cli::http_client::AuthenticatedHttpClient> {
        let profile = config.get_auth_profile()?
            .ok_or(UnifiedIntegrationError::MandatoryAuthNotConfigured)?;
        
        let signing_config = config.get_signing_config()?;
        let env_config = config.current_environment_config()?;
        
        // Build HTTP client with unified configuration
        let builder = crate::cli::http_client::HttpClientBuilder::new()
            .timeout_secs(env_config.performance.default_timeout_secs)
            .retry_config(crate::cli::http_client::RetryConfig {
                max_retries: env_config.performance.default_max_retries,
                initial_delay_ms: 100,
                max_delay_ms: 5000,
                retry_server_errors: true,
                retry_network_errors: true,
            });
        
        // Create authenticated client
        builder.build_authenticated(keypair, profile, Some(signing_config))
            .map_err(|e| UnifiedIntegrationError::InvalidSignatureConfig(format!("Failed to create authenticated client: {}", e)))
    }
    
    /// Display environment status for CLI commands
    pub fn display_environment_status(config: &UnifiedCliConfig) -> UnifiedIntegrationResult<()> {
        let summary = config.get_environment_summary()?;
        
        println!("🔐 DataFold Authentication Status");
        println!("  Environment: {}", summary.name);
        println!("  Policy: {}", summary.signing_policy);
        println!("  Mandatory Auth: ✅ {}", if summary.mandatory_auth { "Enabled" } else { "Disabled" });
        
        if summary.profile_configured {
            println!("  Profile: ✅ Configured");
            if let Some(client_id) = &summary.client_id {
                println!("    Client ID: {}", client_id);
            }
            if let Some(server_url) = &summary.server_url {
                println!("    Server: {}", server_url);
            }
        } else {
            println!("  Profile: ❌ Not configured");
        }
        
        println!("  Signature Components: {} required", summary.components_count);
        println!("  Timestamp: {}", if summary.include_timestamp { "✅" } else { "❌" });
        println!("  Nonce: {}", if summary.include_nonce { "✅" } else { "❌" });
        println!("  Timeout: {}ms", summary.timeout_ms);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_environment_summary() {
        let summary = EnvironmentSummary {
            name: "test".to_string(),
            signing_policy: "standard".to_string(),
            mandatory_auth: true,
            profile_configured: true,
            client_id: Some("test-client".to_string()),
            server_url: Some("https://api.example.com".to_string()),
            components_count: 2,
            include_timestamp: true,
            include_nonce: true,
            timeout_ms: 5000,
        };
        
        assert_eq!(summary.name, "test");
        assert!(summary.mandatory_auth);
        assert!(summary.profile_configured);
        assert_eq!(summary.components_count, 2);
    }
    
    #[test]
    fn test_unified_integration_error_types() {
        let error = UnifiedIntegrationError::MandatoryAuthNotConfigured;
        assert!(format!("{}", error).contains("Mandatory authentication not configured"));
        
        let error = UnifiedIntegrationError::EnvironmentNotFound("prod".to_string());
        assert!(format!("{}", error).contains("Environment not found: prod"));
    }
}