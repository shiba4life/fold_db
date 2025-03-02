use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::error::{FoldDbError, FoldDbResult};
use crate::datafold_node::app::manifest::ApiRequirements;

/// Manages API access for apps
pub struct ApiManager {
    /// Map of available APIs
    available_apis: HashMap<String, ApiDefinition>,
    
    /// Map of app API contexts
    app_contexts: HashMap<String, ApiContext>,
}

/// Represents an API definition
#[derive(Debug, Clone)]
pub struct ApiDefinition {
    /// API name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// API description
    pub description: String,
    
    /// Whether the API is enabled
    pub enabled: bool,
}

/// Represents an API context for an app
#[derive(Debug, Clone)]
pub struct ApiContext {
    /// App name
    pub app_name: String,
    
    /// Map of API proxies
    pub apis: HashMap<String, ApiProxy>,
}

/// Represents an API proxy for an app
#[derive(Debug, Clone)]
pub struct ApiProxy {
    /// API name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// Whether the API is required
    pub required: bool,
}

impl ApiManager {
    /// Creates a new API manager
    pub fn new() -> Self {
        Self {
            available_apis: HashMap::new(),
            app_contexts: HashMap::new(),
        }
    }
    
    /// Registers an API
    pub fn register_api(&mut self, name: &str, version: &str, description: &str) -> FoldDbResult<()> {
        // Check if API already exists
        if self.available_apis.contains_key(name) {
            return Err(FoldDbError::Config(format!("API with name '{}' already registered", name)));
        }
        
        // Register API
        self.available_apis.insert(name.to_string(), ApiDefinition {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            enabled: true,
        });
        
        Ok(())
    }
    
    /// Unregisters an API
    pub fn unregister_api(&mut self, name: &str) -> FoldDbResult<()> {
        // Check if API exists
        if !self.available_apis.contains_key(name) {
            return Err(FoldDbError::Config(format!("API with name '{}' not registered", name)));
        }
        
        // Check if any app requires this API
        for (app_name, context) in &self.app_contexts {
            if let Some(proxy) = context.apis.get(name) {
                if proxy.required {
                    return Err(FoldDbError::Config(format!("Cannot unregister API '{}' because it is required by app '{}'", name, app_name)));
                }
            }
        }
        
        // Unregister API
        self.available_apis.remove(name);
        
        // Remove API from all app contexts
        for context in self.app_contexts.values_mut() {
            context.apis.remove(name);
        }
        
        Ok(())
    }
    
    /// Enables an API
    pub fn enable_api(&mut self, name: &str) -> FoldDbResult<()> {
        // Check if API exists
        let api = self.available_apis.get_mut(name)
            .ok_or_else(|| FoldDbError::Config(format!("API with name '{}' not registered", name)))?;
        
        // Enable API
        api.enabled = true;
        
        Ok(())
    }
    
    /// Disables an API
    pub fn disable_api(&mut self, name: &str) -> FoldDbResult<()> {
        // Check if API exists
        let api = self.available_apis.get_mut(name)
            .ok_or_else(|| FoldDbError::Config(format!("API with name '{}' not registered", name)))?;
        
        // Check if any app requires this API
        for (app_name, context) in &self.app_contexts {
            if let Some(proxy) = context.apis.get(name) {
                if proxy.required {
                    return Err(FoldDbError::Config(format!("Cannot disable API '{}' because it is required by app '{}'", name, app_name)));
                }
            }
        }
        
        // Disable API
        api.enabled = false;
        
        Ok(())
    }
    
    /// Checks if an API is available
    pub fn is_api_available(&self, name: &str) -> bool {
        if let Some(api) = self.available_apis.get(name) {
            api.enabled
        } else {
            false
        }
    }
    
    /// Gets an API proxy
    pub fn get_api_proxy(&self, name: &str) -> FoldDbResult<ApiProxy> {
        // Check if API exists and is enabled
        let api = self.available_apis.get(name)
            .ok_or_else(|| FoldDbError::Config(format!("API with name '{}' not registered", name)))?;
        
        if !api.enabled {
            return Err(FoldDbError::Config(format!("API with name '{}' is disabled", name)));
        }
        
        // Create API proxy
        Ok(ApiProxy {
            name: api.name.clone(),
            version: api.version.clone(),
            required: false,
        })
    }
    
    /// Creates an API context for an app
    pub fn create_app_api_context(&mut self, app_name: &str, apis: &ApiRequirements) -> FoldDbResult<ApiContext> {
        // Create context
        let mut context = ApiContext {
            app_name: app_name.to_string(),
            apis: HashMap::new(),
        };
        
        // Add required APIs
        for api_name in &apis.required {
            if !self.is_api_available(api_name) {
                return Err(FoldDbError::Config(format!("Required API '{}' is not available", api_name)));
            }
            
            let mut proxy = self.get_api_proxy(api_name)?;
            proxy.required = true;
            context.apis.insert(api_name.clone(), proxy);
        }
        
        // Add optional APIs if available
        for api_name in &apis.optional {
            if self.is_api_available(api_name) {
                let proxy = self.get_api_proxy(api_name)?;
                context.apis.insert(api_name.clone(), proxy);
            }
        }
        
        // Store context
        self.app_contexts.insert(app_name.to_string(), context.clone());
        
        Ok(context)
    }
    
    /// Gets an API context for an app
    pub fn get_app_api_context(&self, app_name: &str) -> FoldDbResult<&ApiContext> {
        // Check if context exists
        self.app_contexts.get(app_name)
            .ok_or_else(|| FoldDbError::Config(format!("App '{}' has no API context", app_name)))
    }
    
    /// Removes an API context for an app
    pub fn remove_app_api_context(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Check if context exists
        if !self.app_contexts.contains_key(app_name) {
            return Err(FoldDbError::Config(format!("App '{}' has no API context", app_name)));
        }
        
        // Remove context
        self.app_contexts.remove(app_name);
        
        Ok(())
    }
    
    /// Gets a list of available APIs
    pub fn list_available_apis(&self) -> Vec<ApiInfo> {
        self.available_apis.values()
            .map(|api| ApiInfo {
                name: api.name.clone(),
                version: api.version.clone(),
                description: api.description.clone(),
                enabled: api.enabled,
            })
            .collect()
    }
}

/// Represents API information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiInfo {
    /// API name
    pub name: String,
    
    /// API version
    pub version: String,
    
    /// API description
    pub description: String,
    
    /// Whether the API is enabled
    pub enabled: bool,
}

impl ApiContext {
    /// Creates a new API context
    pub fn new(app_name: &str) -> Self {
        Self {
            app_name: app_name.to_string(),
            apis: HashMap::new(),
        }
    }
    
    /// Adds an API to the context
    pub fn add_api(&mut self, name: &str, proxy: ApiProxy) {
        self.apis.insert(name.to_string(), proxy);
    }
    
    /// Removes an API from the context
    pub fn remove_api(&mut self, name: &str) -> bool {
        self.apis.remove(name).is_some()
    }
    
    /// Checks if the context has an API
    pub fn has_api(&self, name: &str) -> bool {
        self.apis.contains_key(name)
    }
    
    /// Gets an API from the context
    pub fn get_api(&self, name: &str) -> Option<&ApiProxy> {
        self.apis.get(name)
    }
    
    /// Gets a list of APIs in the context
    pub fn list_apis(&self) -> Vec<String> {
        self.apis.keys().cloned().collect()
    }
}
