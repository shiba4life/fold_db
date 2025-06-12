//! Enhanced signing configuration for automatic signature injection
//!
//! This module provides comprehensive configuration management for automatic
//! HTTP request signing in the DataFold CLI.

use crate::cli::auth::CliSigningConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Global signing behavior configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSigningConfig {
    /// Whether to enable automatic signing by default
    pub enabled: bool,
    /// Default signing mode (auto, manual, disabled)
    pub default_mode: SigningMode,
    /// Per-command signing overrides
    pub command_overrides: HashMap<String, SigningMode>,
    /// Environment variable to check for signing preference
    pub env_override: Option<String>,
    /// Whether to prompt user for confirmation on first sign
    pub prompt_on_first_sign: bool,
    /// Default profile to use for signing
    pub default_profile: Option<String>,
}

/// Signing mode options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SigningMode {
    /// Automatically sign all requests
    Auto,
    /// Only sign when explicitly requested with --sign
    Manual,
}


impl FromStr for SigningMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "auto" | "automatic" | "always" => Ok(SigningMode::Auto),
            "manual" | "explicit" | "on-demand" => Ok(SigningMode::Manual),
            _ => Err(format!("Invalid signing mode: {}. Valid options: auto, manual", s)),
        }
    }
}

impl Default for AutoSigningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_mode: SigningMode::Manual,
            command_overrides: HashMap::new(),
            env_override: Some("DATAFOLD_AUTO_SIGN".to_string()),
            prompt_on_first_sign: true,
            default_profile: None,
        }
    }
}

/// Enhanced signing configuration with automatic injection support
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct EnhancedSigningConfig {
    /// Base signing configuration
    #[serde(flatten)]
    pub base: CliSigningConfig,
    /// Automatic signing behavior
    pub auto_signing: AutoSigningConfig,
    /// Debug configuration
    pub debug: SigningDebugConfig,
    /// Performance settings
    pub performance: SigningPerformanceConfig,
}

/// Debug configuration for signature troubleshooting
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SigningDebugConfig {
    /// Whether to enable debug logging for signatures
    pub enabled: bool,
    /// Whether to log canonical signature strings
    pub log_canonical_strings: bool,
    /// Whether to log signature components
    pub log_components: bool,
    /// Whether to log timing information
    pub log_timing: bool,
    /// Directory to save debug signature files
    pub debug_output_dir: Option<String>,
}

/// Performance configuration for signing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPerformanceConfig {
    /// Maximum time to spend on signing (milliseconds)
    pub max_signing_time_ms: u64,
    /// Whether to cache derived keys for the session
    pub cache_keys: bool,
    /// Maximum number of concurrent signing operations
    pub max_concurrent_signs: usize,
}



impl Default for SigningPerformanceConfig {
    fn default() -> Self {
        Self {
            max_signing_time_ms: 5000, // 5 seconds max
            cache_keys: true,
            max_concurrent_signs: 10,
        }
    }
}


impl SigningMode {

    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            SigningMode::Auto => "auto",
            SigningMode::Manual => "manual",
        }
    }

    /// Check if this mode should sign requests automatically
    pub fn should_auto_sign(&self) -> bool {
        matches!(self, SigningMode::Auto)
    }

    /// Check if this mode allows signing when explicitly requested
    pub fn allows_explicit_sign(&self) -> bool {
        matches!(self, SigningMode::Auto | SigningMode::Manual)
    }
}

impl AutoSigningConfig {
    /// Get the effective signing mode for a specific command
    pub fn get_command_mode(&self, command: &str) -> SigningMode {
        // Check environment variable override first
        if let Some(env_var) = &self.env_override {
            if let Ok(env_value) = std::env::var(env_var) {
                if let Ok(mode) = env_value.parse::<SigningMode>() {
                    return mode;
                }
            }
        }

        // Check command-specific override
        if let Some(mode) = self.command_overrides.get(command) {
            return mode.clone();
        }

        // Fall back to default mode
        self.default_mode.clone()
    }

    /// Set signing mode for a specific command
    pub fn set_command_mode(&mut self, command: String, mode: SigningMode) {
        self.command_overrides.insert(command, mode);
    }

    /// Remove command-specific override
    pub fn remove_command_override(&mut self, command: &str) {
        self.command_overrides.remove(command);
    }

    /// Check if automatic signing is effectively enabled
    pub fn is_effective_auto_signing(&self, command: &str) -> bool {
        self.enabled && self.get_command_mode(command).should_auto_sign()
    }
}

impl EnhancedSigningConfig {
    /// Create configuration for a specific command context
    pub fn for_command(&self, command: &str) -> CommandSigningContext {
        let mode = self.auto_signing.get_command_mode(command);
        
        CommandSigningContext {
            mode: mode.clone(),
            should_auto_sign: self.auto_signing.is_effective_auto_signing(command),
            allows_explicit: mode.allows_explicit_sign(),
            base_config: self.base.clone(),
            debug_enabled: self.debug.enabled,
            profile: self.auto_signing.default_profile.clone(),
        }
    }

    /// Update from command-line arguments
    pub fn apply_cli_overrides(
        &mut self,
        sign_flag: Option<bool>,
        profile: Option<String>,
        debug: Option<bool>,
    ) {
        if let Some(sign) = sign_flag {
            if sign {
                self.auto_signing.default_mode = SigningMode::Auto;
            } else {
                self.auto_signing.default_mode = SigningMode::Manual;
            }
        }

        if let Some(prof) = profile {
            self.auto_signing.default_profile = Some(prof);
        }

        if let Some(debug_enabled) = debug {
            self.debug.enabled = debug_enabled;
        }
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate performance settings
        if self.performance.max_signing_time_ms == 0 {
            return Err("max_signing_time_ms must be greater than 0".to_string());
        }

        if self.performance.max_concurrent_signs == 0 {
            return Err("max_concurrent_signs must be greater than 0".to_string());
        }

        // Validate debug output directory if specified
        if let Some(debug_dir) = &self.debug.debug_output_dir {
            if !std::path::Path::new(debug_dir).exists() {
                return Err(format!("Debug output directory does not exist: {}", debug_dir));
            }
        }

        Ok(())
    }
}

/// Context for command-specific signing behavior
#[derive(Debug, Clone)]
pub struct CommandSigningContext {
    /// Effective signing mode for this command
    pub mode: SigningMode,
    /// Whether to automatically sign requests
    pub should_auto_sign: bool,
    /// Whether explicit signing is allowed
    pub allows_explicit: bool,
    /// Base signing configuration
    pub base_config: CliSigningConfig,
    /// Whether debug logging is enabled
    pub debug_enabled: bool,
    /// Profile to use for signing
    pub profile: Option<String>,
}

impl CommandSigningContext {
    /// Determine if a request should be signed given CLI flags
    pub fn should_sign_request(&self, explicit_sign: Option<bool>) -> bool {
        match explicit_sign {
            Some(true) => self.allows_explicit,
            Some(false) => false,
            None => self.should_auto_sign,
        }
    }

    /// Get signing configuration for request
    pub fn get_signing_config(&self) -> &CliSigningConfig {
        &self.base_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_mode_parsing() {
        assert_eq!("auto".parse::<SigningMode>().unwrap(), SigningMode::Auto);
        assert_eq!("MANUAL".parse::<SigningMode>().unwrap(), SigningMode::Manual);
        assert!("disabled".parse::<SigningMode>().is_err());
        assert!("invalid".parse::<SigningMode>().is_err());
    }

    #[test]
    fn test_signing_mode_behavior() {
        assert!(SigningMode::Auto.should_auto_sign());
        assert!(!SigningMode::Manual.should_auto_sign());

        assert!(SigningMode::Auto.allows_explicit_sign());
        assert!(SigningMode::Manual.allows_explicit_sign());
    }

    #[test]
    fn test_auto_signing_config() {
        let mut config = AutoSigningConfig::default();
        
        // Test command-specific overrides
        config.set_command_mode("query".to_string(), SigningMode::Auto);
        assert_eq!(config.get_command_mode("query"), SigningMode::Auto);
        assert_eq!(config.get_command_mode("other"), SigningMode::Manual);

        // Test effective auto signing
        config.enabled = true;
        assert!(config.is_effective_auto_signing("query"));
        assert!(!config.is_effective_auto_signing("other"));
    }

    #[test]
    fn test_command_signing_context() {
        let config = EnhancedSigningConfig::default();
        let context = config.for_command("test");

        // Test request signing decision
        assert!(!context.should_sign_request(None)); // Default manual mode - no auto signing
        assert!(context.should_sign_request(Some(true))); // Explicit true and manual mode allows it
        assert!(!context.should_sign_request(Some(false))); // Explicit false
    }

    #[test]
    fn test_cli_overrides() {
        let mut config = EnhancedSigningConfig::default();
        
        config.apply_cli_overrides(Some(true), Some("prod".to_string()), Some(true));
        
        assert_eq!(config.auto_signing.default_mode, SigningMode::Auto);
        assert_eq!(config.auto_signing.default_profile, Some("prod".to_string()));
        assert!(config.debug.enabled);
    }
}