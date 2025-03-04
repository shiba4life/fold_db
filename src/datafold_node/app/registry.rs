use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{FoldDbError, FoldDbResult};
use crate::datafold_node::app::manifest::AppManifest;
use crate::datafold_node::app::resource::{AppResourceManager, ResourceAllocation};
use crate::datafold_node::app::window::AppWindow;
use crate::datafold_node::app::api::ApiManager;

/// Registry for managing apps
#[derive(Clone)]
pub struct AppRegistry {
    /// Map of registered apps by name
    apps: HashMap<String, RegisteredApp>,
    
    /// Resource manager for app resource allocation
    resource_manager: Arc<Mutex<AppResourceManager>>,
    
    /// API manager for app API access
    api_manager: Arc<Mutex<ApiManager>>,
}

/// Represents a registered app
#[derive(Clone)]
pub struct RegisteredApp {
    /// App manifest
    pub manifest: AppManifest,
    
    /// App window (if open)
    pub window: Option<AppWindow>,
    
    /// Resource allocation
    pub resources: ResourceAllocation,
    
    /// Whether the app is running
    pub running: bool,
}

impl AppRegistry {
    /// Creates a new app registry
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            resource_manager: Arc::new(Mutex::new(AppResourceManager::new())),
            api_manager: Arc::new(Mutex::new(ApiManager::new())),
        }
    }
    
    /// Registers an app with the registry
    pub fn register_app(&mut self, manifest: AppManifest) -> FoldDbResult<()> {
        // Validate manifest
        manifest.validate()
            .map_err(|e| FoldDbError::Config(format!("Invalid app manifest: {}", e)))?;
        
        // Check if app with same name already exists
        if self.apps.contains_key(&manifest.name) {
            return Err(FoldDbError::Config(format!("App with name '{}' already registered", manifest.name)));
        }
        
        // Create default resource allocation
        let resources = ResourceAllocation {
            memory: 50 * 1024 * 1024, // 50 MB
            cpu: 25.0,                // 25% of one CPU core
            storage: 5 * 1024 * 1024, // 5 MB
            bandwidth: 512 * 1024,    // 512 KB/s
        };
        
        // Register app
        self.apps.insert(manifest.name.clone(), RegisteredApp {
            manifest,
            window: None,
            resources,
            running: false,
        });
        
        Ok(())
    }
    
    /// Unregisters an app from the registry
    pub fn unregister_app(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Check if app exists
        if !self.apps.contains_key(app_name) {
            return Err(FoldDbError::Config(format!("App with name '{}' not registered", app_name)));
        }
        
        // Stop app if running
        if let Some(app) = self.apps.get_mut(app_name) {
            if app.running {
                self.stop_app(app_name)?;
            }
        }
        
        // Remove app
        self.apps.remove(app_name);
        
        Ok(())
    }
    
    /// Starts an app
    pub fn start_app(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Check if app exists
        let app = self.apps.get_mut(app_name)
            .ok_or_else(|| FoldDbError::Config(format!("App with name '{}' not registered", app_name)))?;
        
        // Check if app is already running
        if app.running {
            return Err(FoldDbError::Config(format!("App '{}' is already running", app_name)));
        }
        
        // Allocate resources
        let resource_manager = self.resource_manager.lock()
            .map_err(|_| FoldDbError::Config("Failed to lock resource manager".to_string()))?;
        
        // Create app window
        let mut window = AppWindow::new(&app.manifest.window, &app.manifest.entry)?;
        
        // Open the window
        window.open()?;
        
        // Open the app in a browser
        window.open_in_browser()?;
        
        // Store the window
        app.window = Some(window);
        
        // Set app as running
        app.running = true;
        
        Ok(())
    }
    
    /// Stops an app
    pub fn stop_app(&mut self, app_name: &str) -> FoldDbResult<()> {
        // Check if app exists
        let app = self.apps.get_mut(app_name)
            .ok_or_else(|| FoldDbError::Config(format!("App with name '{}' not registered", app_name)))?;
        
        // Check if app is running
        if !app.running {
            return Err(FoldDbError::Config(format!("App '{}' is not running", app_name)));
        }
        
        // Close window
        if let Some(window) = app.window.take() {
            window.close()?;
        }
        
        // Release resources
        let resource_manager = self.resource_manager.lock()
            .map_err(|_| FoldDbError::Config("Failed to lock resource manager".to_string()))?;
        
        // Set app as not running
        app.running = false;
        
        Ok(())
    }
    
    /// Gets a list of registered apps
    pub fn list_apps(&self) -> Vec<String> {
        self.apps.keys().cloned().collect()
    }
    
    /// Gets app information
    pub fn get_app_info(&self, app_name: &str) -> FoldDbResult<AppInfo> {
        // Check if app exists
        let app = self.apps.get(app_name)
            .ok_or_else(|| FoldDbError::Config(format!("App with name '{}' not registered", app_name)))?;
        
        Ok(AppInfo {
            name: app.manifest.name.clone(),
            version: app.manifest.version.clone(),
            description: app.manifest.description.clone(),
            running: app.running,
        })
    }
}

/// Represents app information
pub struct AppInfo {
    /// App name
    pub name: String,
    
    /// App version
    pub version: String,
    
    /// App description
    pub description: String,
    
    /// Whether the app is running
    pub running: bool,
}
