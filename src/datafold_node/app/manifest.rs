use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an app manifest that defines app metadata and requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// Unique name of the app
    pub name: String,
    
    /// Version of the app
    pub version: String,
    
    /// Description of the app
    pub description: String,
    
    /// Entry point for the app (HTML file path)
    pub entry: String,
    
    /// Schemas used by the app
    pub schemas: Vec<String>,
    
    /// Window configuration
    pub window: WindowConfig,
    
    /// App permissions
    pub permissions: AppPermissions,
    
    /// API requirements
    pub apis: ApiRequirements,
    
    /// Payment configuration (optional)
    pub payments: Option<PaymentConfig>,
}

/// Represents window configuration for an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    /// Default window size
    pub default_size: Size,
    
    /// Minimum window size
    pub min_size: Size,
    
    /// Window title
    pub title: String,
    
    /// Window icon (optional)
    pub icon: Option<String>,
    
    /// Whether the window is resizable
    pub resizable: bool,
}

/// Represents a size with width and height
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    /// Width in pixels
    pub width: u32,
    
    /// Height in pixels
    pub height: u32,
}

/// Represents app permissions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermissions {
    /// Required permissions
    pub required: Vec<String>,
    
    /// Optional permissions
    #[serde(default)]
    pub optional: Vec<String>,
}

/// Represents API requirements for an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiRequirements {
    /// Required APIs
    pub required: Vec<String>,
    
    /// Optional APIs
    #[serde(default)]
    pub optional: Vec<String>,
}

/// Represents payment configuration for an app
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentConfig {
    /// Payment provider
    pub provider: String,
    
    /// Payment address
    pub address: String,
    
    /// Payment rates
    pub rates: HashMap<String, f64>,
}

impl AppManifest {
    /// Validates the manifest
    pub fn validate(&self) -> Result<(), String> {
        // Check for empty name
        if self.name.is_empty() {
            return Err("App name cannot be empty".to_string());
        }
        
        // Check for valid version format (semver)
        if !is_valid_semver(&self.version) {
            return Err(format!("Invalid version format: {}", self.version));
        }
        
        // Check for empty entry point
        if self.entry.is_empty() {
            return Err("Entry point cannot be empty".to_string());
        }
        
        // Validate window config
        if self.window.default_size.width < self.window.min_size.width ||
           self.window.default_size.height < self.window.min_size.height {
            return Err("Default size cannot be smaller than minimum size".to_string());
        }
        
        Ok(())
    }
}

/// Checks if a string is a valid semantic version
fn is_valid_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    
    // Basic check for major.minor.patch format
    if parts.len() != 3 {
        return false;
    }
    
    // Check that each part is a valid number
    for part in parts {
        if part.parse::<u32>().is_err() {
            return false;
        }
    }
    
    true
}
