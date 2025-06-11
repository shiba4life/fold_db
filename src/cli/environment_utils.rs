//! Environment management utilities for unified configuration
//!
//! Provides utilities for switching between development, staging, and production
//! environments using the unified configuration system.

use crate::config::unified_config::{UnifiedConfigManager, UnifiedConfigError};
use std::collections::HashMap;

/// Result type for environment operations
pub type EnvironmentResult<T> = Result<T, UnifiedConfigError>;

/// Environment switching and management utilities
pub struct EnvironmentManager {
    manager: UnifiedConfigManager,
}

impl EnvironmentManager {
    /// Create a new environment manager
    pub fn new() -> EnvironmentResult<Self> {
        let manager = UnifiedConfigManager::load_default()?;
        Ok(Self { manager })
    }

    /// Create environment manager from specific config file
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> EnvironmentResult<Self> {
        let manager = UnifiedConfigManager::load_from_file(path)?;
        Ok(Self { manager })
    }

    /// List all available environments
    pub fn list_environments(&self) -> Vec<String> {
        self.manager.list_environments().into_iter().cloned().collect()
    }

    /// Get current environment
    pub fn current_environment(&self) -> String {
        self.manager.current_environment().to_string()
    }

    /// Switch to a different environment
    pub fn switch_environment(&mut self, environment: &str) -> EnvironmentResult<()> {
        self.manager.set_environment(environment.to_string())?;
        println!("Switched to environment: {}", environment);
        Ok(())
    }

    /// Show environment configuration summary
    pub fn show_environment_info(&self, environment: Option<&str>) -> EnvironmentResult<()> {
        let env = environment.map(|s| s.to_string()).unwrap_or_else(|| self.current_environment());
        
        // Create temporary manager for the specified environment
        let mut temp_manager = self.manager.clone();
        temp_manager.set_environment(env.clone())?;
        
        let env_config = temp_manager.current_environment_config()?;
        
        println!("Environment: {}", env);
        println!("  Signing Policy: {}", env_config.signing.policy);
        println!("  Signing Timeout: {}ms", env_config.signing.timeout_ms);
        println!("  Content Digest: {}", env_config.signing.include_content_digest);
        println!("  Strict Timing: {}", env_config.verification.strict_timing);
        println!("  Log Level: {}", env_config.logging.level);
        println!("  Cache Keys: {}", env_config.performance.cache_keys);
        println!("  Max Concurrent Signs: {}", env_config.performance.max_concurrent_signs);
        
        Ok(())
    }

    /// Compare environments
    pub fn compare_environments(&self, env1: &str, env2: &str) -> EnvironmentResult<()> {
        let mut temp_manager1 = self.manager.clone();
        let mut temp_manager2 = self.manager.clone();
        
        temp_manager1.set_environment(env1.to_string())?;
        temp_manager2.set_environment(env2.to_string())?;
        
        let config1 = temp_manager1.current_environment_config()?;
        let config2 = temp_manager2.current_environment_config()?;
        
        println!("Comparing {} vs {}:", env1, env2);
        
        // Compare signing configurations
        if config1.signing.policy != config2.signing.policy {
            println!("  Signing Policy: {} vs {}", config1.signing.policy, config2.signing.policy);
        }
        
        if config1.signing.timeout_ms != config2.signing.timeout_ms {
            println!("  Signing Timeout: {}ms vs {}ms", config1.signing.timeout_ms, config2.signing.timeout_ms);
        }
        
        if config1.verification.strict_timing != config2.verification.strict_timing {
            println!("  Strict Timing: {} vs {}", config1.verification.strict_timing, config2.verification.strict_timing);
        }
        
        if config1.logging.level != config2.logging.level {
            println!("  Log Level: {} vs {}", config1.logging.level, config2.logging.level);
        }
        
        if config1.performance.cache_keys != config2.performance.cache_keys {
            println!("  Cache Keys: {} vs {}", config1.performance.cache_keys, config2.performance.cache_keys);
        }
        
        Ok(())
    }

    /// Validate all environments
    pub fn validate_all_environments(&self) -> EnvironmentResult<()> {
        let environments = self.list_environments();
        let mut validation_errors = Vec::new();
        
        for env in &environments {
            let mut temp_manager = self.manager.clone();
            if let Err(e) = temp_manager.set_environment(env.clone()) {
                validation_errors.push(format!("Environment '{}': {}", env, e));
            }
        }
        
        if validation_errors.is_empty() {
            println!("All {} environments are valid", environments.len());
        } else {
            println!("Validation errors found:");
            for error in validation_errors {
                println!("  {}", error);
            }
        }
        
        Ok(())
    }

    /// Get environment configuration as key-value pairs for scripting
    pub fn get_environment_vars(&self, environment: Option<&str>) -> EnvironmentResult<HashMap<String, String>> {
        let env = environment.map(|s| s.to_string()).unwrap_or_else(|| self.current_environment());
        let mut vars = HashMap::new();
        
        vars.insert("DATAFOLD_ENVIRONMENT".to_string(), env.to_string());
        
        let mut temp_manager = self.manager.clone();
        temp_manager.set_environment(env.clone())?;
        
        let env_config = temp_manager.current_environment_config()?;
        
        vars.insert("DATAFOLD_SIGNING_POLICY".to_string(), env_config.signing.policy.clone());
        vars.insert("DATAFOLD_SIGNING_TIMEOUT_MS".to_string(), env_config.signing.timeout_ms.to_string());
        vars.insert("DATAFOLD_CONTENT_DIGEST".to_string(), env_config.signing.include_content_digest.to_string());
        vars.insert("DATAFOLD_STRICT_TIMING".to_string(), env_config.verification.strict_timing.to_string());
        vars.insert("DATAFOLD_LOG_LEVEL".to_string(), env_config.logging.level.clone());
        vars.insert("DATAFOLD_CACHE_KEYS".to_string(), env_config.performance.cache_keys.to_string());
        
        Ok(vars)
    }

    /// Get the underlying unified config manager
    pub fn unified_manager(&self) -> &UnifiedConfigManager {
        &self.manager
    }
}

/// CLI command functions for environment management
pub mod commands {
    use super::*;

    /// List all available environments
    pub fn list_environments() -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        let environments = manager.list_environments();
        let current = manager.current_environment();
        
        println!("Available environments:");
        for env in environments {
            if env == current {
                println!("  {} (current)", env);
            } else {
                println!("  {}", env);
            }
        }
        
        Ok(())
    }

    /// Switch to a different environment
    pub fn switch_environment(environment: &str) -> EnvironmentResult<()> {
        let mut manager = EnvironmentManager::new()?;
        manager.switch_environment(environment)
    }

    /// Show current environment information
    pub fn show_current_environment() -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        manager.show_environment_info(None)
    }

    /// Show specific environment information
    pub fn show_environment(environment: &str) -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        manager.show_environment_info(Some(environment))
    }

    /// Compare two environments
    pub fn compare_environments(env1: &str, env2: &str) -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        manager.compare_environments(env1, env2)
    }

    /// Validate all environments
    pub fn validate_environments() -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        manager.validate_all_environments()
    }

    /// Export environment variables for scripting
    pub fn export_environment_vars(environment: Option<&str>) -> EnvironmentResult<()> {
        let manager = EnvironmentManager::new()?;
        let vars = manager.get_environment_vars(environment)?;
        
        for (key, value) in vars {
            println!("export {}={}", key, value);
        }
        
        Ok(())
    }
}